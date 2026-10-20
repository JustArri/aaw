use crate::{
    engine::graph::{DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE},
    rt_channels,
    server::api::notifications::ProjectChange,
    synths,
};
pub use CtrProject;
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
pub use stored::*;
use tokio::sync::{Mutex, RwLock, broadcast};

mod ctr;
pub mod dummy_graph;
pub mod effects;
pub mod fluid_graph;
pub mod graph;
pub mod instrument_graph;
mod rt;
pub mod sfizz_graph;
mod stored;

pub async fn load(
    assets_dir: &Path,
    stored: StoredProject,
    min_instrument_slots: usize,
    min_synths_per_graph: usize,
    channel_count: usize,
    sample_rate: u32,
) -> synths::Result<(CtrProject, RtProject)> {
    let (ctr_instrument_graph, rt_instrument_graph) = instrument_graph::prepare(
        stored.instrument_graph,
        min_instrument_slots,
        channel_count,
        sample_rate,
    );

    let (ctr_dummy_graph, rt_dummy_graph) =
        dummy_graph::prepare(stored.dummy_graph, min_synths_per_graph, sample_rate);

    let (ctr_fluid_graph, rt_fluid_graph) =
        fluid_graph::prepare(assets_dir, stored.fluid_graph, min_synths_per_graph, sample_rate)
            .await
            .unwrap_or_else(|_| fluid_graph::default(min_synths_per_graph));

    let (ctr_sfizz_graph, rt_sfizz_graph) =
        sfizz_graph::prepare(assets_dir, stored.sfizz_graph, min_synths_per_graph, sample_rate)
            .await
            .unwrap_or_else(|_| sfizz_graph::default(min_synths_per_graph));

    let ctr_project = CtrProject {
        revision: 0,
        name: stored.name,
        instrument_graph: ctr_instrument_graph,
        dummy_graph: ctr_dummy_graph,
        fluid_graph: ctr_fluid_graph,
        sfizz_graph: ctr_sfizz_graph,
    };

    let rt_project = RtProject {
        instrument_graph: rt_instrument_graph,
        dummy_graph: rt_dummy_graph,
        fluid_graph: rt_fluid_graph,
        sfizz_graph: rt_sfizz_graph,
    };

    Ok((ctr_project, rt_project))
}

#[allow(clippy::too_many_arguments)]
pub async fn create_engine(
    //TODO: cleanup args
    assets_dir: &Path,
    project: StoredProject,
    stream_error: Arc<AtomicBool>,
    cmd_capacity: usize,
    midi_capacity: usize,
    min_instrument_slots: usize,
    min_synths_per_graph: usize,
    project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
) -> synths::Result<(Arc<Mutex<Engine>>, EngineHandle, Runtime)> {
    let (cmd_tx, cmd_rx) = rt_channels::create_notified_mpsc(cmd_capacity);
    let (reply_tx, reply_rx) = rt_channels::create_spsc(cmd_capacity);
    let (midi_tx, midi_rx) = rt_channels::create_notified_mpsc(midi_capacity);
    let pending_replies = Arc::new(Mutex::new(HashMap::new()));

    let (ctr_project, rt_project) = load(
        assets_dir,
        project,
        min_instrument_slots,
        min_synths_per_graph,
        DEFAULT_CHANNEL_COUNT,
        DEFAULT_SAMPLE_RATE,
    )
    .await?;

    let handle = EngineHandle {
        cmd_tx,
        pending_replies: Arc::clone(&pending_replies),
        midi_tx,
        next_cmd_id: Arc::new(AtomicU32::new(0)),
        project: Arc::new(RwLock::new(ctr_project)),
        project_change_tx,
    };

    let (cmd_rx, cmd_notify) = cmd_rx.unwrap();
    let (midi_rx, midi_notify) = midi_rx.unwrap();
    let engine = Engine { project: rt_project, audio_blocks: None, midi_rx, cmd_rx, reply_tx };
    let engine = Arc::new(Mutex::new(engine));

    let runtime = Runtime {
        pending_replies,
        reply_rx,
        cmd_notify,
        midi_notify,
        stream_error,
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
