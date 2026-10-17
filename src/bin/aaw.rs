use aaw::{
    APP_VERSION, PACKAGE_NAME,
    app_state::{AppState, LastProject},
    audio::{self, params::DeviceCfgSelection},
    engine::{self, stored::ProjectVersioned},
    files::{load_opt, save},
    midi::MidiRuntime,
    projects::Projects,
    rt_channels,
    server::api::audio::AudioOutputStream,
};
use anyhow::Context;
use clap::{ArgAction, Parser};
use std::{
    env,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

const AAW_ROOT_DIR_ENV: &str = "AAW_ROOT_DIR";
const CFG_AUDIO_OUTPUT: (&str, &str) = ("config/audio-output.json", "audio output config");
const CFG_LAST_PROJECT_SUBJECT: &str = "last project";
const CFG_LAST_PROJECT: (&str, &str) = ("config/last-project.json", CFG_LAST_PROJECT_SUBJECT);
const CFG_MIDI_AC: (&str, &str) = ("config/midi-auto-connections.json", "midi auto-connections");
const PROJECTS_DIR: &str = "projects";
const ASSETS_DIR: &str = "assets";

#[derive(Parser, Debug)]
#[command(
    name = PACKAGE_NAME,
    version = APP_VERSION,
    disable_help_flag = true
)]
struct Args {
    #[arg(long = "help", action = ArgAction::HelpLong, help = "Print help")]
    help: Option<bool>,

    #[arg(long, default_value = "0.0.0.0", help = "HTTP server address")]
    http_addr: String,

    #[arg(long, default_value_t = 80, help = "HTTP server port")]
    http_port: u16,

    #[arg(long, default_value_t = 256, help = "MIDI broadcast channel capacity")]
    midi_bc_capacity: usize,

    #[arg(long, default_value_t = 32, help = "MIDI to-relay channel capacity")]
    midi_to_relay_capacity: usize,

    #[arg(long, default_value_t = 32, help = "MIDI to-engine channel capacity")]
    midi_to_engine_capacity: usize,

    #[arg(long, default_value_t = 16, help = "Command channel capacity")]
    cmd_capacity: usize,

    #[arg(
        long,
        default_value_t = 32,
        help = "Minimum instrument capacity (can be overriden by a project)"
    )]
    min_instrument_slots: usize,

    #[arg(
        long,
        default_value_t = 32,
        help = "Minimum synths-per-graph capacity (can be overriden by a project)"
    )]
    min_synths_per_graph: usize,

    #[arg(long, help = "Create a migrated copy of specified project")]
    migrate_project: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    aaw::subscribe_tracing("aaw=info,midi-event=info,midi-runtime=info");
    info!("** Arri's Audio Workstation **");

    let result = match &args.migrate_project {
        Some(path) => migrate_project(path).await,
        None => run(args).await,
    };

    if let Err(err) = result {
        error!("{err:#}");
    }
}

async fn migrate_project(path: &Path) -> anyhow::Result<()> {
    let content = tokio::fs::read(&path).await.context("failed to load file")?;
    let content = String::from_utf8_lossy(&content);

    let orig_project: ProjectVersioned =
        serde_json::from_str(&content).context("failed to parse file")?;

    info!("old version: {}", orig_project.version().unwrap_or(ProjectVersioned::CURRENT_VERSION));
    info!("new version: {}", ProjectVersioned::CURRENT_VERSION);

    let migrated_project = orig_project.migrate();
    let value = serde_json::to_string_pretty(&migrated_project).context("failed to serialize:")?;
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let new_path = path.with_file_name(format!("{stem}.migrated.json"));

    tokio::fs::write(&new_path, value)
        .await
        .with_context(|| format!("failed to save file '{}'", new_path.display()))?;

    info!("file generated: {}", new_path.display());
    Ok(())
}

async fn run(args: Args) -> anyhow::Result<()> {
    let aaw_root_dir = aaw_root_dir()?;
    info!("root directory: {}", aaw_root_dir.display());

    let projects_dir = aaw_root_dir.join(PROJECTS_DIR);
    tokio::fs::create_dir_all(&projects_dir).await?;
    let assets_dir = aaw_root_dir.join(ASSETS_DIR);
    tokio::fs::create_dir_all(&assets_dir).await?;

    info!(addr = %args.http_addr, port = args.http_port, "starting HTTP server...");
    let bind_params = (args.http_addr, args.http_port);
    let listener =
        tokio::net::TcpListener::bind(bind_params).await.context("failed to bind TCP listener")?;
    let port = listener.local_addr()?.port();

    let (midi_bc_tx, _) = tokio::sync::broadcast::channel(args.midi_bc_capacity);
    let (midi_relay_tx, midi_relay_rx) =
        rt_channels::create_notified_mpsc(args.midi_to_relay_capacity);

    let stream_error = Arc::new(AtomicBool::new(true));

    let project: Option<LastProject> = load_opt(&aaw_root_dir, CFG_LAST_PROJECT).await;
    let (project, project_id) = match project {
        Some(LastProject::Cached(cached)) => (cached, None),
        Some(LastProject::Saved(id)) => {
            let path = Projects::project_path(&projects_dir, id);
            let proj = load_opt::<engine::stored::Project>("", (&path, CFG_LAST_PROJECT_SUBJECT))
                .await
                .unwrap_or_default();
            (proj, Some(id))
        }
        None => (engine::stored::Project::default(), None),
    };

    let (engine, engine_handle, engine_runtime) = engine::create_engine(
        project,
        Arc::clone(&stream_error),
        args.cmd_capacity,
        args.midi_to_engine_capacity,
        args.min_instrument_slots,
        args.min_synths_per_graph,
    )
    .await
    .context("failed to create engine")?;

    let midi_auto_connections = load_opt(&aaw_root_dir, CFG_MIDI_AC).await.unwrap_or_default();
    let midi_event_senders: Arc<[_]> = Arc::new([engine_handle.midi_tx.clone(), midi_relay_tx]);

    let midi_runtime = Arc::new(Mutex::new(
        MidiRuntime::new(16, "aaw", Arc::clone(&midi_event_senders), midi_auto_connections)
            .context("failed to create midi runtime")?,
    ));

    let stop = CancellationToken::new();

    let output_stream =
        if let Some(info) = load_opt::<AudioOutputStream>(&aaw_root_dir, CFG_AUDIO_OUTPUT).await {
            try_start_stream(info, Arc::clone(&engine), Arc::clone(&stream_error))
                .await
                .inspect_err(|err| warn!("{err}"))
                .ok()
        } else {
            None
        };

    let output_stream = Arc::new(Mutex::new(output_stream));
    let active_project_id = Arc::new(RwLock::new(project_id));

    let root_router = aaw::server::router(assets_dir).with_state(AppState {
        midi_tx: midi_bc_tx.clone(),
        midi_runtime: Arc::clone(&midi_runtime),
        midi_event_senders,
        engine,
        engine_handle: engine_handle.clone(),
        output_stream: Arc::clone(&output_stream),
        min_instrument_slots: args.min_instrument_slots,
        min_synths_per_graph: args.min_synths_per_graph,
        stream_error: Arc::clone(&stream_error),
        projects: Arc::new(RwLock::new(
            Projects::new(projects_dir).await.context("failed to load projects")?,
        )),
        active_project_id: Arc::clone(&active_project_id),
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

    let project = match *active_project_id.read().await {
        Some(id) => LastProject::Saved(id),
        None => LastProject::Cached(engine_handle.save_project().await),
    };
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
        .unwrap_or(env::current_dir().context("failed to get current working directory")?);
    let path = std::path::absolute(&path).with_context(|| {
        format!("failed to resolve path: {} ({AAW_ROOT_DIR_ENV})", path.display())
    })?;

    if !path.exists() {
        anyhow::bail!("directory doesn't exist: {} ({AAW_ROOT_DIR_ENV})", path.display());
    }

    if !path.is_dir() {
        anyhow::bail!("not a directory: {} ({AAW_ROOT_DIR_ENV})", path.display());
    }

    Ok(path)
}

async fn try_start_stream(
    stream_info: AudioOutputStream,
    engine: Arc<Mutex<engine::rt::Engine>>,
    stream_error: Arc<AtomicBool>,
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

    let resolved_config = device_config.context("failed to resolve audio config")?;
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

    let stream = resolved_config
        .start_stream(Arc::clone(&engine), stream_error)
        .await
        .context("failed to start audio stream")?;
    Ok((stream, stream_info))
}
