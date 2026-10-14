use aaw::{
    audio::{self, params::DeviceCfgSelection},
    engine::{self},
    midi::MidiRuntime,
    rt_channels,
    server::{AppState, api::audio::AudioOutputStream},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

const AAW_ROOT_DIR_ENV: &str = "AAW_ROOT_DIR";
const CFG_AUDIO_OUTPUT: (&str, &str) = ("config/audio-output.json", "audio output config");
const CFG_LAST_PROJECT: (&str, &str) = ("config/last-project.json", "last project");
const CFG_MIDI_AC: (&str, &str) = ("config/midi-auto-connections.json", "midi auto-connections");

const HTTP_SERVER_ADDR: &str = "0.0.0.0";
const HTTP_SERVER_PORT: u16 = 8080;
const MIDI_BC_CAPACITY: usize = 256;
const MIDI_TO_RELAY_CAPACITY: usize = 32;
const MIDI_TO_ENGINE_CAPACITY: usize = 32;
const CMD_CAPACITY: usize = 16;
const MAX_SYNTH_SLOTS: usize = 32;
const MAX_SYNTHS_PER_GRAPH: usize = 32;

#[tokio::main]
async fn main() {
    aaw::subscribe_tracing("aaw=info,midi-event=info,midi-runtime=info");
    info!("** Arri's Audio Workstation **");

    if let Err(err) = run().await {
        error!("{err}");
    }
}

async fn run() -> anyhow::Result<()> {
    let aaw_root_dir = aaw_root_dir()?;
    info!("root directory: {}", aaw_root_dir.display());

    info!(addr = %HTTP_SERVER_ADDR, port = HTTP_SERVER_PORT, "starting HTTP server...");

    let bind_params = (HTTP_SERVER_ADDR, HTTP_SERVER_PORT);
    let listener = tokio::net::TcpListener::bind(bind_params).await?;
    let port = listener.local_addr()?.port();

    let (midi_bc_tx, _) = tokio::sync::broadcast::channel(MIDI_BC_CAPACITY);
    let (midi_relay_tx, midi_relay_rx) = rt_channels::create_notified_mpsc(MIDI_TO_RELAY_CAPACITY);

    let project = load_opt(&aaw_root_dir, CFG_LAST_PROJECT).await.unwrap_or_default();
    let (engine, engine_handle, engine_runtime) = engine::create_engine(
        project,
        CMD_CAPACITY,
        MIDI_TO_ENGINE_CAPACITY,
        MAX_SYNTH_SLOTS,
        MAX_SYNTHS_PER_GRAPH,
    )
    .await?;

    let engine = Arc::new(std::sync::Mutex::new(engine));

    let midi_auto_connections = load_opt(&aaw_root_dir, CFG_MIDI_AC).await.unwrap_or_default();
    let midi_event_senders: Arc<[_]> = Arc::new([engine_handle.midi_tx.clone(), midi_relay_tx]);

    let midi_runtime = Arc::new(Mutex::new(MidiRuntime::new(
        16,
        "aaw",
        Arc::clone(&midi_event_senders),
        midi_auto_connections,
    )?));

    let stop = CancellationToken::new();

    let output_stream = Arc::new(Mutex::new(
        load_opt::<AudioOutputStream>(&aaw_root_dir, CFG_AUDIO_OUTPUT).await.and_then(|info| {
            try_start_stream(info, Arc::clone(&engine)).inspect_err(|err| warn!("{err}")).ok()
        }),
    ));

    let root_router = aaw::server::router().with_state(AppState {
        midi_tx: midi_bc_tx.clone(),
        midi_runtime: Arc::clone(&midi_runtime),
        midi_event_senders,
        engine,
        output_stream: Arc::clone(&output_stream),
    });

    let relay_handle = aaw::spawn_midi_relay_task(stop.clone(), midi_relay_rx, midi_bc_tx);
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
        save(&aaw_root_dir, CFG_AUDIO_OUTPUT, &stream_info).await;
        drop(stream);
    }

    let project = engine_handle.save_project().await;
    save(&aaw_root_dir, CFG_LAST_PROJECT, &project).await;

    {
        let midi_runtime = midi_runtime.lock().await;
        let midi_ac = midi_runtime.input_auto_connections();
        save(&aaw_root_dir, CFG_MIDI_AC, midi_ac).await;
    }

    relay_handle.await?;
    let _ = server_handle.await?;
    midi_rt_handle.await?;
    engine_rt_handle.await?;

    Ok(())
}

fn aaw_root_dir() -> anyhow::Result<PathBuf> {
    let path = env::var_os(AAW_ROOT_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let path = std::path::absolute(path)?;

    if !path.exists() {
        anyhow::bail!("directory doesn't exist: {} ({AAW_ROOT_DIR_ENV})", path.display());
    }

    if !path.is_dir() {
        anyhow::bail!("not a directory: {} ({AAW_ROOT_DIR_ENV})", path.display());
    }

    Ok(path)
}

async fn save<T: Serialize>(root: &Path, (path, subject): (impl AsRef<Path>, &str), value: &T) {
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

    let full_path = root.join(&path);

    if inner(&full_path, subject, value).await.is_err() {
        error!(path = %full_path.display(), "failed to save {subject}");
    }
}

async fn load_opt<T: DeserializeOwned>(
    root: &Path,
    (path, subject): (impl AsRef<Path>, &str),
) -> Option<T> {
    let full_path = root.join(&path);

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
