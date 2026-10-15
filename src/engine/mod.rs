use crate::{rt_channels, synths};
use rt::Command;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32},
    },
};
use tokio::sync::{Mutex, RwLock};

pub mod ctr;
pub mod dummy_graph;
pub mod effect;
pub mod fluid_graph;
pub mod graph;
pub mod instrument_graph;
pub mod rt;
pub mod sfizz_graph;
pub mod stored;

pub async fn load(
    stored: stored::Project,
    min_instrument_slots: usize,
    min_synths_per_graph: usize,
) -> synths::Result<(ctr::Project, rt::Project)> {
    let (ctr_instrument_graph, rt_instrument_graph) =
        instrument_graph::prepare(stored.instrument_graph, min_instrument_slots);

    let (ctr_dummy_graph, rt_dummy_graph) =
        dummy_graph::prepare(stored.dummy_graph, min_synths_per_graph);

    let (ctr_fluid_graph, rt_fluid_graph) =
        fluid_graph::prepare(stored.fluid_graph, min_synths_per_graph)
            .await
            .unwrap_or_else(|_| fluid_graph::default(min_synths_per_graph));

    let (ctr_sfizz_graph, rt_sfizz_graph) =
        sfizz_graph::prepare(stored.sfizz_graph, min_synths_per_graph)
            .await
            .unwrap_or_else(|_| sfizz_graph::default(min_synths_per_graph));

    let ctr_project = ctr::Project {
        instrument_graph: ctr_instrument_graph,
        dummy_graph: ctr_dummy_graph,
        fluid_graph: ctr_fluid_graph,
        sfizz_graph: ctr_sfizz_graph,
    };

    let rt_project = rt::Project {
        instrument_graph: rt_instrument_graph,
        dummy_graph: rt_dummy_graph,
        fluid_graph: rt_fluid_graph,
        sfizz_graph: rt_sfizz_graph,
    };

    Ok((ctr_project, rt_project))
}

pub async fn create_engine(
    project: stored::Project,
    stream_error: Arc<AtomicBool>,
    cmd_capacity: usize,
    midi_capacity: usize,
    min_instrument_slots: usize,
    min_synths_per_graph: usize,
) -> synths::Result<(Arc<Mutex<rt::Engine>>, ctr::EngineHandle, ctr::Runtime)> {
    let (cmd_tx, cmd_rx) = rt_channels::create_notified_mpsc(cmd_capacity);
    let (reply_tx, reply_rx) = rt_channels::create_spsc(cmd_capacity);
    let (midi_tx, midi_rx) = rt_channels::create_notified_mpsc(midi_capacity);
    let pending_replies = Arc::new(Mutex::new(HashMap::new()));

    let (ctr_project, rt_project) =
        load(project, min_instrument_slots, min_synths_per_graph).await?;

    let handle = ctr::EngineHandle {
        cmd_tx,
        pending_replies: Arc::clone(&pending_replies),
        midi_tx,
        next_cmd_id: Arc::new(AtomicU32::new(0)),
        project: Arc::new(RwLock::new(ctr_project)),
    };

    let (cmd_rx, cmd_notify) = cmd_rx.unwrap();
    let (midi_rx, midi_notify) = midi_rx.unwrap();
    let engine = rt::Engine { project: rt_project, audio_blocks: None, midi_rx, cmd_rx, reply_tx };
    let engine = Arc::new(Mutex::new(engine));

    let runtime = ctr::Runtime {
        pending_replies,
        reply_rx,
        cmd_notify,
        midi_notify,
        stream_error,
        engine: Arc::clone(&engine),
    };

    Ok((engine, handle, runtime))
}
