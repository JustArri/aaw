use aaw::{
    APP_VERSION, MAX_BLOCK_SIZE, PACKAGE_NAME,
    app_state::{AppState, LastEngineMidiConfiguration, LastProject, Preferences},
    assets::Assets,
    audio::{self, params::DeviceCfgSelection},
    engine::{
        self, EngineContext,
        midi_app_controls::StoredMidiEngineControls,
        project::{CtrProjectStorage, ProjectAsset, StoredProject, StoredProjectVersioned},
    },
    engine_midi_configurations::{
        CtrEngineMidiConfiguration, CtrEngineMidiConfigurationStorage, EngineMidiConfigurations,
        StoredEngineMidiConfiguration,
    },
    files::{self, load_opt},
    midi::MidiRuntime,
    projects::Projects,
    rt_channels,
    server::api::audio::{AudioOutputStream, AudioOutputStreamInfo},
};
use anyhow::Context;
use clap::{ArgAction, Parser};
use cpal::traits::StreamTrait;
use std::{
    env,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

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

    #[arg(long, default_value_t = 256, help = "Project change broadcast channel capacity")]
    project_change_bc_capacity: usize,

    #[arg(long, default_value_t = 32, help = "MIDI to-relay channel capacity")]
    midi_to_relay_capacity: usize,

    #[arg(long, default_value_t = 32, help = "MIDI to-engine channel capacity")]
    midi_to_engine_capacity: usize,

    #[arg(long, default_value_t = 16, help = "Command channel capacity")]
    cmd_capacity: usize,

    #[arg(long, default_value_t = 16, help = "Audio recording channel capacity")]
    recording_queue_capacity: usize,

    #[arg(
        long,
        default_value_t = 32,
        help = "Minimum controller capacity (can be overridden by a project)"
    )]
    min_controllers: usize,

    #[arg(
        long,
        default_value_t = 32,
        help = "Minimum instrument capacity (can be overridden by a project)"
    )]
    min_instruments: usize,

    #[arg(long, help = "Create a migrated copy of specified project")]
    migrate_project: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    aaw::subscribe_tracing("aaw=info");
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

    let orig_project: StoredProjectVersioned =
        serde_json::from_str(&content).context("failed to parse file")?;

    info!(
        "old version: {}",
        orig_project.version().unwrap_or(StoredProjectVersioned::CURRENT_VERSION)
    );
    info!("new version: {}", StoredProjectVersioned::CURRENT_VERSION);

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

    let projects_dir = aaw_root_dir.join(aaw::PROJECTS_DIR);
    tokio::fs::create_dir_all(&projects_dir).await?;
    let engine_midi_configurations_dir = aaw_root_dir.join(aaw::ENGINE_MIDI_CONFIGURATIONS_DIR);
    tokio::fs::create_dir_all(&engine_midi_configurations_dir).await?;
    let assets_dir = aaw_root_dir.join(aaw::ASSETS_DIR);
    tokio::fs::create_dir_all(&assets_dir).await?;
    let recordings_dir = aaw_root_dir.join(aaw::RECORDINGS_DIR);
    tokio::fs::create_dir_all(&recordings_dir).await?;

    info!(addr = %args.http_addr, port = args.http_port, "starting HTTP server...");
    let bind_params = (args.http_addr, args.http_port);
    let listener =
        tokio::net::TcpListener::bind(bind_params).await.context("failed to bind TCP listener")?;
    let port = listener.local_addr()?.port();

    let (midi_bc_tx, _) = tokio::sync::broadcast::channel(args.midi_bc_capacity);
    let (midi_relay_tx, midi_relay_rx) =
        rt_channels::create_notified_mpsc(args.midi_to_relay_capacity);

    let stream_error = Arc::new(AtomicBool::new(true));

    let (project, project_storage) = match load_opt(&aaw_root_dir, aaw::CFG_LAST_PROJECT).await {
        Some(LastProject { id: Some(id) }) => {
            let path = Projects::project_path(&projects_dir, id);

            if let Some(proj) =
                load_opt::<StoredProject>("", (&path, aaw::CFG_LAST_PROJECT_SUBJECT)).await
            {
                let revision = proj.revision;
                (proj, Some(CtrProjectStorage { id, revision }))
            } else {
                files::save(&aaw_root_dir, aaw::CFG_LAST_PROJECT, &LastProject { id: None }).await;
                (StoredProject::default(), None)
            }
        }
        _ => (StoredProject::default(), None),
    };

    let preferences =
        load_opt::<Preferences>(&aaw_root_dir, aaw::CFG_PREFERENCES).await.unwrap_or_default();
    let mut engine_midi_configurations =
        EngineMidiConfigurations::new(engine_midi_configurations_dir)
            .await
            .context("failed to load engine MIDI configurations")?;
    let last_engine_midi_configuration = load_opt::<LastEngineMidiConfiguration>(
        &aaw_root_dir,
        aaw::CFG_LAST_ENGINE_MIDI_CONFIGURATION,
    )
    .await;
    let loaded_engine_midi_configuration = match last_engine_midi_configuration {
        Some(LastEngineMidiConfiguration { id: Some(id) }) => engine_midi_configurations
            .load(id)
            .await
            .ok()
            .map(|configuration| (configuration, Some(id))),
        _ => None,
    };
    let (engine_midi_configuration, engine_midi_configuration_id) =
        if let Some(configuration) = loaded_engine_midi_configuration {
            configuration
        } else {
            let midi_app_controls =
                load_opt::<StoredMidiEngineControls>(&aaw_root_dir, aaw::CFG_MIDI_APP_CONTROLS)
                    .await
                    .unwrap_or_default();
            let input_auto_connections =
                load_opt(&aaw_root_dir, aaw::CFG_MIDI_AC).await.unwrap_or_default();
            (
                StoredEngineMidiConfiguration {
                    midi_app_controls,
                    input_auto_connections,
                    ..Default::default()
                },
                None,
            )
        };
    let active_engine_midi_configuration = CtrEngineMidiConfiguration {
        name: engine_midi_configuration.name.clone(),
        storage: engine_midi_configuration_id.map(|id| CtrEngineMidiConfigurationStorage { id }),
    };
    let midi_app_controls = engine_midi_configuration.midi_app_controls;
    let midi_auto_connections = engine_midi_configuration.input_auto_connections;

    let (project_change_tx, _) = broadcast::channel(args.project_change_bc_capacity);
    let (midi_app_controls_change_tx, _) = broadcast::channel(args.project_change_bc_capacity);
    let (transport_position_tx, _) = broadcast::channel(args.project_change_bc_capacity);
    let (engine, engine_handle, engine_runtime) = engine::create_engine(
        ProjectAsset { project, storage: project_storage },
        EngineContext {
            assets_dir: &assets_dir,
            stream_error: Arc::clone(&stream_error),
            cmd_capacity: args.cmd_capacity,
            midi_capacity: args.midi_to_engine_capacity,
            recording_queue_capacity: args.recording_queue_capacity,
            min_instruments: args.min_instruments,
            min_controllers: args.min_controllers,
            project_change_tx,
            midi_app_controls_change_tx,
            midi_app_controls,
        },
    )
    .await
    .context("failed to create engine")?;

    let midi_event_senders: Arc<[_]> = Arc::new([midi_relay_tx]);

    let midi_runtime = Arc::new(Mutex::new(
        MidiRuntime::new(16, "aaw", Arc::clone(&midi_event_senders), midi_auto_connections)
            .context("failed to create midi runtime")?,
    ));

    let engine_metrics = Arc::new(engine::metrics::EngineMetrics::default());

    let output_stream = if let Some(info) =
        load_opt::<AudioOutputStreamInfo>(&aaw_root_dir, aaw::CFG_AUDIO_OUTPUT).await
    {
        try_start_stream(
            info,
            Arc::clone(&engine),
            Arc::clone(&stream_error),
            Arc::clone(&engine_metrics),
        )
        .await
        .inspect_err(|err| warn!("{err}"))
        .ok()
    } else {
        None
    };

    let output_stream = Arc::new(Mutex::new(output_stream));

    let stop = CancellationToken::new();
    let root_router = aaw::server::router(assets_dir.clone()).with_state(AppState {
        stop: stop.clone(),
        midi_tx: midi_bc_tx.clone(),
        transport_position_tx: transport_position_tx.clone(),
        midi_runtime: Arc::clone(&midi_runtime),
        midi_event_senders,
        engine,
        engine_handle: engine_handle.clone(),
        output_stream: Arc::clone(&output_stream),
        min_instruments: args.min_instruments,
        min_controllers: args.min_controllers,
        stream_error: Arc::clone(&stream_error),
        engine_metrics: Arc::clone(&engine_metrics),
        aaw_root_dir: aaw_root_dir.into(),
        projects: Arc::new(RwLock::new(
            Projects::new(projects_dir).await.context("failed to load projects")?,
        )),
        engine_midi_configurations: Arc::new(RwLock::new(engine_midi_configurations)),
        active_engine_midi_configuration: Arc::new(RwLock::new(active_engine_midi_configuration)),
        assets: Arc::new(RwLock::new(
            Assets::new(assets_dir).await.context("failed to load assets")?,
        )),
        recordings_dir: recordings_dir.into(),
        preferences: Arc::new(RwLock::new(preferences)),
    });

    let transport_position_handle = aaw::server::api::notifications::spawn_transport_position_task(
        stop.clone(),
        engine_handle.clone(),
        engine_metrics,
        transport_position_tx,
    );
    let relay_handle = aaw::spawn_midi_relay_task(
        stop.clone(),
        engine_handle.clone(),
        midi_relay_rx,
        midi_bc_tx.clone(),
    );
    let server_handle = aaw::spawn_web_server_task(stop.clone(), listener, root_router);
    let midi_rt_handle = aaw::spawn_midi_runtime_task(stop.clone(), Arc::clone(&midi_runtime));
    let engine_rt_handle = tokio::spawn(engine_runtime.into_future(stop.clone()));

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

    relay_handle.await?;
    transport_position_handle.await?;
    let _ = server_handle.await?;
    midi_rt_handle.await?;
    let _ = engine_rt_handle.await?;

    Ok(())
}

fn aaw_root_dir() -> anyhow::Result<PathBuf> {
    let path = env::var_os(aaw::AAW_ROOT_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or(env::current_dir().context("failed to get current working directory")?);
    let path = std::path::absolute(&path).with_context(|| {
        format!("failed to resolve path: {} ({})", path.display(), aaw::AAW_ROOT_DIR_ENV)
    })?;

    if !path.exists() {
        anyhow::bail!("directory doesn't exist: {} ({})", path.display(), aaw::AAW_ROOT_DIR_ENV);
    }

    if !path.is_dir() {
        anyhow::bail!("not a directory: {} ({})", path.display(), aaw::AAW_ROOT_DIR_ENV);
    }

    Ok(path)
}

async fn try_start_stream(
    stream_info: AudioOutputStreamInfo,
    engine: Arc<Mutex<engine::Engine>>,
    stream_error: Arc<AtomicBool>,
    engine_metrics: Arc<engine::metrics::EngineMetrics>,
) -> anyhow::Result<AudioOutputStream> {
    let device_config = audio::output::ResolvedConfig::find(DeviceCfgSelection {
        host: audio::params::HostSelection::Id(stream_info.host_id.as_str()),
        device: audio::params::DeviceSelection::Id(stream_info.output_id.as_str()),
        latency: if stream_info.buffer_size != 0 {
            audio::params::LatencySelection::CustomBufferSize(stream_info.buffer_size)
        } else {
            audio::params::LatencySelection::Automatic
        },
        sample_rate: audio::params::SampleRateSelection::Fixed(stream_info.sample_rate),
    });

    let resolved_config = device_config.context("failed to resolve audio config")?;
    let sample_format = resolved_config.sample_format().to_string();
    let stream_cfg = resolved_config.config();

    let mut stream_info = AudioOutputStreamInfo {
        host_id: stream_info.host_id,
        output_id: stream_info.output_id,
        buffer_size: match stream_cfg.buffer_size {
            cpal::BufferSize::Default => 0,
            cpal::BufferSize::Fixed(size) => size,
        },
        channel_count: stream_cfg.channels as u32,
        sample_rate: stream_cfg.sample_rate,
        max_block_size: MAX_BLOCK_SIZE,
        sample_format,
    };

    let stream = resolved_config
        .start_stream(Arc::clone(&engine), stream_error, engine_metrics)
        .await
        .context("failed to start audio stream")?;

    if stream_info.buffer_size == 0 {
        stream_info.buffer_size = stream.buffer_size().unwrap_or_default();
    }

    Ok(AudioOutputStream { stream, info: stream_info })
}
