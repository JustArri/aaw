use crate::{
    DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE,
    engine::{
        instrument_graph::InstrumentGraphError,
        project::{ProjectAsset, ProjectContext},
    },
    rt_channels,
    server::api::{audio::AudioOutputStreamConfig, notifications::ProjectChange},
};
pub use ctr::*;
pub use rt::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32},
    },
};
use tokio::sync::{Mutex, RwLock, broadcast};

pub mod controller_graph;
mod ctr;
pub mod effects;
pub mod graph;
pub mod instrument_graph;
pub mod project;
mod rt;
pub mod user_presets;

pub const RETIREMENT_QUEUE_CAPACITY: usize = 16;

#[derive(Clone)]
pub struct EngineContext<'a> {
    pub assets_dir: &'a Path,
    pub stream_error: Arc<AtomicBool>,
    pub cmd_capacity: usize,
    pub midi_capacity: usize,
    pub recording_queue_capacity: usize,
    pub min_instruments: usize,
    pub min_controllers: usize,
    pub project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
}

impl<'a> From<&EngineContext<'a>> for ProjectContext<'a> {
    fn from(value: &EngineContext<'a>) -> Self {
        Self {
            assets_dir: value.assets_dir,
            min_instruments: value.min_instruments,
            min_controllers: value.min_controllers,
            stream_config: &AudioOutputStreamConfig {
                channel_count: DEFAULT_CHANNEL_COUNT,
                sample_rate: DEFAULT_SAMPLE_RATE,
                max_block_size: MAX_BLOCK_SIZE,
            },
        }
    }
}

pub async fn create_engine<'a>(
    project: ProjectAsset,
    ctx: EngineContext<'a>,
) -> Result<(Arc<Mutex<Engine>>, EngineHandle, Runtime), InstrumentGraphError> {
    let (cmd_tx, cmd_rx) = rt_channels::create_notified_mpsc(ctx.cmd_capacity);
    let (reply_tx, reply_rx) = rt_channels::create_spsc(ctx.cmd_capacity);
    let (midi_tx, midi_rx) = rt_channels::create_notified_mpsc(ctx.midi_capacity);
    let (recording_tx, recording_rx) = rt_channels::create_spsc(ctx.recording_queue_capacity);
    let (clap_parameter_tx, clap_parameter_rx) = rt_channels::create_spsc(ctx.cmd_capacity);
    let pending_replies = Arc::new(Mutex::new(HashMap::new()));
    let audio_rec_capture_enabled = Arc::new(AtomicBool::new(false));
    let audio_rec_state = Arc::new(RwLock::new(AudioRecordingState::default()));
    let audio_rec_params = Arc::new(Mutex::new(AudioRecordingParams {
        sample_rate: DEFAULT_SAMPLE_RATE,
        file_path: PathBuf::new(),
    }));

    let (ctr_project, rt_project, controller_graph) =
        project::prepare(project, (&ctx).into()).await?;
    let controller_graph = Arc::new(RwLock::new(controller_graph));

    let handle = EngineHandle {
        cmd_tx,
        pending_replies: Arc::clone(&pending_replies),
        midi_tx,
        next_cmd_id: Arc::new(AtomicU32::new(0)),
        project: Arc::new(RwLock::new(ctr_project)),
        project_change_tx: ctx.project_change_tx,
        audio_rec_state: Arc::clone(&audio_rec_state),
        audio_rec_params: Arc::clone(&audio_rec_params),
        controller_graph: Arc::clone(&controller_graph),
    };

    let (cmd_rx, cmd_notify) = cmd_rx.unwrap();
    let (midi_rx, midi_notify) = midi_rx.unwrap();
    let engine = Engine {
        project: rt_project,
        audio_blocks: None,
        midi_rx,
        cmd_rx,
        reply_tx,
        recording_tx,
        clap_parameter_tx,
        sample_rate: DEFAULT_SAMPLE_RATE as f32,
        audio_rec_capture_enabled: Arc::clone(&audio_rec_capture_enabled),
    };
    let engine = Arc::new(Mutex::new(engine));

    let runtime = Runtime {
        pending_replies,
        reply_rx,
        recording_rx,
        clap_parameter_rx,
        cmd_notify,
        midi_notify,
        stream_error: ctx.stream_error,
        engine: Arc::clone(&engine),
        audio_rec_state,
        audio_rec_params,
        project: Arc::clone(&handle.project),
        project_change_tx: handle.project_change_tx.clone(),
        audio_rec_capture_enabled,
        audio_rec_active_sample_rate: DEFAULT_SAMPLE_RATE,
        audio_rec_file: None,
        audio_rec_buffer: Vec::with_capacity(MAX_BLOCK_SIZE * 4),
        audio_rec_bytes: 0,
    };

    Ok((engine, handle, runtime))
}

async fn resolve_asset_path(assets_dir: &Path, relative: &Path) -> std::io::Result<PathBuf> {
    if relative.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid asset path"));
    }

    let root = tokio::fs::canonicalize(assets_dir).await?;
    let resolved = tokio::fs::canonicalize(root.join(relative)).await?;

    if !resolved.starts_with(&root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "asset path escapes the assets directory",
        ));
    }

    Ok(resolved)
}
