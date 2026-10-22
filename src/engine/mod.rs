use crate::{
    DEFAULT_CHANNEL_COUNT, DEFAULT_MAX_BLOCK_SIZE, DEFAULT_SAMPLE_RATE,
    engine::{
        project::{ProjectAsset, ProjectContext},
        synth_graph::SynthGraphError,
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

mod ctr;
pub mod effects;
pub mod graph;
pub mod instrument_graph;
pub mod project;
mod rt;
pub mod synth_graph;
pub mod user_presets;

#[derive(Clone)]
pub struct EngineContext<'a> {
    pub assets_dir: &'a Path,
    pub stream_error: Arc<AtomicBool>,
    pub cmd_capacity: usize,
    pub midi_capacity: usize,
    pub min_instruments: usize,
    pub min_synths: usize,
    pub project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
}

impl<'a> From<&EngineContext<'a>> for ProjectContext<'a> {
    fn from(value: &EngineContext<'a>) -> Self {
        Self {
            assets_dir: value.assets_dir,
            min_instruments: value.min_instruments,
            min_synths: value.min_synths,
            stream_config: AudioOutputStreamConfig {
                channel_count: DEFAULT_CHANNEL_COUNT,
                sample_rate: DEFAULT_SAMPLE_RATE,
                max_block_size: DEFAULT_MAX_BLOCK_SIZE,
            },
        }
    }
}

pub async fn create_engine<'a>(
    project: ProjectAsset,
    ctx: EngineContext<'a>,
) -> Result<(Arc<Mutex<Engine>>, EngineHandle, Runtime), SynthGraphError> {
    let (cmd_tx, cmd_rx) = rt_channels::create_notified_mpsc(ctx.cmd_capacity);
    let (reply_tx, reply_rx) = rt_channels::create_spsc(ctx.cmd_capacity);
    let (midi_tx, midi_rx) = rt_channels::create_notified_mpsc(ctx.midi_capacity);
    let pending_replies = Arc::new(Mutex::new(HashMap::new()));

    let (ctr_project, rt_project) = project::prepare(project, (&ctx).into()).await?;

    let handle = EngineHandle::new(
        cmd_tx,
        Arc::clone(&pending_replies),
        midi_tx,
        Arc::new(AtomicU32::new(0)),
        Arc::new(RwLock::new(ctr_project)),
        ctx.project_change_tx,
    );

    let (cmd_rx, cmd_notify) = cmd_rx.unwrap();
    let (midi_rx, midi_notify) = midi_rx.unwrap();
    let engine = Engine { project: rt_project, audio_blocks: None, midi_rx, cmd_rx, reply_tx };
    let engine = Arc::new(Mutex::new(engine));

    let runtime = Runtime {
        pending_replies,
        reply_rx,
        cmd_notify,
        midi_notify,
        stream_error: ctx.stream_error,
        engine: Arc::clone(&engine),
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
