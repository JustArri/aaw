use crate::{rt_channels, synths};
use rt::Command;
use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicU32},
};
use tokio::sync::{Mutex, RwLock};

pub mod ctr;
pub mod dummy_graph;
pub mod effect;
pub mod fluid_graph;
pub mod graph;
pub mod rt;
pub mod sfizz_graph;
pub mod stored;
pub mod synth_graph;
// TODO: make mods private?

pub async fn create_engine(
    project: stored::Project,
    cmd_capacity: usize,
    midi_capacity: usize,
    slot_capacity: usize,
    synth_capacity: usize,
) -> synths::Result<(rt::Engine, ctr::EngineHandle, ctr::Runtime)> {
    let (cmd_tx, cmd_rx) = rt_channels::create_mpsc(cmd_capacity);
    let (reply_tx, reply_rx) = rt_channels::create_spsc(cmd_capacity);
    let (midi_tx, midi_rx) = rt_channels::create_notified_mpsc(midi_capacity);

    let (ctr_synth_graph, rt_synth_graph) =
        synth_graph::prepare(project.synth_graph, slot_capacity);

    let (ctr_dummy_graph, rt_dummy_graph) =
        dummy_graph::prepare(project.dummy_graph, synth_capacity);

    let (ctr_fluid_graph, rt_fluid_graph) =
        fluid_graph::prepare(project.fluid_graph, synth_capacity)
            .await
            .unwrap_or_else(|_| fluid_graph::default(synth_capacity));

    let (ctr_sfizz_graph, rt_sfizz_graph) =
        sfizz_graph::prepare(project.sfizz_graph, synth_capacity)
            .await
            .unwrap_or_else(|_| sfizz_graph::default(synth_capacity));

    let rt_engine = rt::Engine {
        synth_graph: rt_synth_graph,
        dummy_graph: rt_dummy_graph,
        fluid_graph: rt_fluid_graph,
        sfizz_graph: rt_sfizz_graph,
        audio_blocks: None,
        midi_rx,
        cmd_rx,
        reply_tx,
    };

    let ctr_engine = ctr::Engine {
        synth_graph: ctr_synth_graph,
        dummy_graph: ctr_dummy_graph,
        fluid_graph: ctr_fluid_graph,
        sfizz_graph: ctr_sfizz_graph,
    };

    let pending_replies = Arc::new(Mutex::new(HashMap::new()));

    let controller = ctr::EngineHandle {
        cmd_tx,
        pending_replies: Arc::clone(&pending_replies),
        midi_tx,
        next_cmd_id: Arc::new(AtomicU32::new(0)),
        engine: Arc::new(RwLock::new(ctr_engine)),
    };

    let runtime = ctr::Runtime { pending_replies, reply_rx };

    Ok((rt_engine, controller, runtime))
}
