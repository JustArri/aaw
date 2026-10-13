use aaw::{
    audio::{self, params::DeviceCfgSelection},
    engine::{self},
    midi::MidiRuntime,
    rt_channels,
    server::{AppState, api::audio::AudioOutputStream},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

const CFG_AUDIO_OUTPUT: (&str, &str) = ("config/audio-output.json", "audio output config");
const CFG_LAST_PROJECT: (&str, &str) = ("config/last-project.json", "last project");
const CFG_MIDI_AC: (&str, &str) = ("config/midi-auto-connections.json", "midi auto-connections");

#[tokio::main]
async fn main() {
    aaw::subscribe_tracing("aaw=info,midi-event=info,midi-runtime=info");
    info!("** Arri's Audio Workstation **");

    if let Err(err) = run().await {
        error!("{err}");
    }
}

async fn run() -> anyhow::Result<()> {
    let http_server_addr = "0.0.0.0";
    let http_server_port = 8080;

    info!(addr = %http_server_addr, port = http_server_port, "starting HTTP server...");

    let bind_params = (http_server_addr, http_server_port);
    let listener = tokio::net::TcpListener::bind(bind_params).await?;
    let port = listener.local_addr()?.port();

    let (midi_bc_tx, _) = tokio::sync::broadcast::channel(256);
    let (midi_tx, midi_rx) = rt_channels::create_mpsc(32);

    let project = load_opt(CFG_LAST_PROJECT).await.unwrap_or_default();
    let (engine, engine_handle, engine_runtime) =
        engine::create_engine(project, 16, 32, 32, 32).await?;

    let engine = Arc::new(std::sync::Mutex::new(engine));

    let midi_auto_connections = load_opt(CFG_MIDI_AC).await.unwrap_or_default();

    let midi_runtime = Arc::new(Mutex::new(MidiRuntime::new(
        16,
        "aaw",
        vec![engine_handle.midi_tx.clone(), midi_tx],
        midi_auto_connections,
    )?));

    let stop = CancellationToken::new();

    let output_stream = Arc::new(Mutex::new(
        load_opt::<AudioOutputStream>(CFG_AUDIO_OUTPUT).await.and_then(|info| {
            try_start_stream(info, Arc::clone(&engine)).inspect_err(|err| warn!("{err}")).ok()
        }),
    ));

    let root_router = aaw::server::router().with_state(AppState {
        midi_tx: midi_bc_tx.clone(),
        midi_runtime: Arc::clone(&midi_runtime),
        engine,
        output_stream: Arc::clone(&output_stream),
    });

    let relay_handle = aaw::spawn_midi_relay_task(stop.clone(), midi_rx, midi_bc_tx);
    let server_handle = aaw::spawn_web_server_task(stop.clone(), listener, root_router);
    let midi_rt_handle = aaw::spawn_midi_runtime_task(stop.clone(), Arc::clone(&midi_runtime));
    let engine_rt_handle = aaw::spawn_engine_runtime_task(stop.clone(), engine_runtime);

    info!("http server started @ http://localhost:{port}");

    info!("press Ctrl+C to stop");
    tokio::select! {
        _ = stop.cancelled() => {}
        _ = tokio::signal::ctrl_c() => {
            println!();
            stop.cancel();
        }
    }

    info!("stopping...");

    if let Some((stream, stream_info)) = output_stream.lock().await.take() {
        save(CFG_AUDIO_OUTPUT, &stream_info).await;
        drop(stream);
    }

    let project = engine_handle.project().await;
    save(CFG_LAST_PROJECT, &project).await;

    {
        let midi_runtime = midi_runtime.lock().await;
        let midi_ac = midi_runtime.input_auto_connections();
        save(CFG_MIDI_AC, midi_ac).await;
    }

    relay_handle.await?;
    let _ = server_handle.await?;
    midi_rt_handle.await?;
    engine_rt_handle.await?;

    Ok(())
}

async fn save<T: Serialize>((path, subject): (impl AsRef<Path>, &str), value: &T) {
    async fn inner<T: Serialize>(full_path: &Path, subject: &str, value: &T) -> anyhow::Result<()> {
        info!(path = %full_path.display(), "saving {subject}...");
        if let Some(parent) = full_path.parent()
            && !parent.exists()
        {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(full_path, serde_json::to_string_pretty(value)?).await?;
        Ok(())
    }

    let Ok(full_path) = std::path::absolute(&path) else {
        error!(path = %path.as_ref().display(), "failed to save {subject}");
        return;
    };

    if inner(&full_path, subject, value).await.is_err() {
        error!(path = %path.as_ref().display(), "failed to save {subject}");
    }
}

async fn load_opt<T: DeserializeOwned>((path, subject): (impl AsRef<Path>, &str)) -> Option<T> {
    let Ok(full_path) = std::path::absolute(&path) else {
        error!(path = %path.as_ref().display(), "failed to load {subject}");
        return None;
    };

    info!(path = %full_path.display(), "loading {subject}...");
    let content = tokio::fs::read(&full_path).await.ok()?;
    let content = String::from_utf8_lossy(&content);

    match serde_json::from_str(&content) {
        Ok(value) => Some(value),
        Err(err) => {
            error!(path = %full_path.display(), %err, "failed to load {subject}");
            None
        }
    }
}

fn try_start_stream(
    stream_info: AudioOutputStream,
    engine: Arc<std::sync::Mutex<engine::rt::Engine>>,
) -> anyhow::Result<(cpal::Stream, AudioOutputStream)> {
    let device_config = audio::output::ResolvedConfig::find(DeviceCfgSelection {
        host: audio::params::HostSelection::Id(stream_info.host_id.as_str()),
        device: audio::params::DeviceSelection::Id(stream_info.output_id.as_str()),
        latency: if let Some(buffer_size) = stream_info.buffer_size {
            audio::params::LatencySelection::CustomBufferSize(buffer_size)
        } else {
            audio::params::LatencySelection::Automatic
        },
        sample_rate: audio::params::SampleRateSelection::Fixed(stream_info.sample_rate),
    });

    let resolved_config = device_config?;
    let sample_format = resolved_config.sample_format().to_string();
    let stream_cfg = resolved_config.config();

    let stream_info = AudioOutputStream {
        host_id: stream_info.host_id,
        output_id: stream_info.output_id,
        buffer_size: match stream_cfg.buffer_size {
            cpal::BufferSize::Default => None,
            cpal::BufferSize::Fixed(size) => Some(size),
        },
        channel_count: stream_cfg.channels as u32,
        sample_rate: stream_cfg.sample_rate,
        sample_format,
    };

    let stream = resolved_config.start_stream(Arc::clone(&engine))?;
    Ok((stream, stream_info))
}

// fn old_run() -> anyhow::Result<()> {
//     let (engine, engine_handle) = create_engine();

//     let (midi_tx, mut midi_rx) = rt_channels::create_mpsc(32);

//     let auto_connect_inputs = HashMap::from([
//         (
//             "129:0".into(),
//             AutoConnectInput {
//                 slot_id: 0,
//                 slot_id_fixed: false,
//             },
//         ),
//         (
//             "148:0".into(),
//             AutoConnectInput {
//                 slot_id: 0,
//                 slot_id_fixed: false,
//             },
//         ),
//     ]);
//     let mut midi_runtime = MidiRuntime::new(
//         16,
//         "aaw",
//         vec![engine_handle.midi_tx, midi_tx],
//         auto_connect_inputs,
//     )?;
//     midi_runtime.refresh_input_ports()?;

//     println!("--- MIDI Info ---");
//     for (i, info) in midi_runtime.available_input_ports().iter().enumerate() {
//         println!("[{i}] {info:?}");
//     }

//     println!("===================");
//     println!("--- Audio Hosts ---");
//     for (i, host) in audio::info::list_hosts().iter().enumerate() {
//         println!("[{i}] {}: {}", host.id, host.name);
//     }

//     println!("=====================");
//     println!("--- Audio Outputs ---");
//     for (i, info) in audio::info::list_output_devices("pipewire")?
//         .iter()
//         .enumerate()
//     {
//         println!("[{i:>2}] {} {}", info.host_id, info.id);
//     }

//     #[cfg(debug_assertions)]
//     const LATENCY_SEL: audio::params::LatencySelection =
//         audio::params::LatencySelection::CustomBufferSize(1024);

//     #[cfg(not(debug_assertions))]
//     const LATENCY_SEL: audio::params::LatencySelection =
//         audio::params::LatencySelection::CustomBufferSize(256);

//     let device_config = audio::output::ResolvedConfig::find(&DeviceCfgSelection {
//         host: audio::params::HostSelection::Id("pipewire".into()),
//         device: audio::params::DeviceSelection::Id(
//             "alsa_output.usb-HP__Inc_HyperX_Cloud_Alpha_Wireless_00000001-00.analog-stereo".into(),
//         ),
//         latency: LATENCY_SEL,
//         sample_rate: audio::params::SampleRateSelection::Automatic,
//     })?;

//     println!("=====================");
//     println!("--- Stream Info ---");
//     info!("Sample format: {}", device_config.sample_format());
//     let stream_cfg = device_config.config();
//     info!("Buffer size: {:?}", stream_cfg.buffer_size);
//     info!("Channel count: {}", stream_cfg.channels);
//     info!("Sample rate: {} Hz", stream_cfg.sample_rate);

//     let _stream = device_config.start_stream(Arc::clone(&engine));

//     println!("=====================");

//     spawn_midi_runtime_task(midi_runtime);

//     loop {
//         if let Ok(event) = midi_rx.pop() {
//             info!(target: "midi-event", "{event:?}");

//             if let MidiEventKind::ControlChange {
//                 controller,
//                 value: _,
//             } = event.kind
//                 && controller == ControlChange::AllNotesOff as u8
//             {
//                 info!("[midi panic]");
//                 break;
//             }
//         } else {
//             thread::sleep(Duration::from_millis(10));
//         }
//     }

//     Ok(())
// }
