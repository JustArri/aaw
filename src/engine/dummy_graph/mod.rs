use crate::{engine::graph::LocalId, memory::Siv, synths};
pub use ctr::*;
pub use rt::*;
use std::collections::HashMap;
pub use stored::*;

mod ctr;
mod rt;
mod stored;

pub fn prepare(
    stored: stored::StoredGraph,
    num_slots: usize,
    sample_rate: u32,
) -> (CtrGraph, RtGraph) {
    let mut ctr_synths: HashMap<LocalId, CtrSynth> = HashMap::new();
    let mut rt_synths: HashMap<LocalId, synths::Dummy> = HashMap::new();

    for (local_id, stored_synth) in stored.synths {
        let (ctr_synth, rt_synth) = prepare_synth(stored_synth.name, sample_rate);
        ctr_synths.insert(local_id, ctr_synth);
        rt_synths.insert(local_id, rt_synth);
    }

    let mut ctr_synths: Siv<_, _> = ctr_synths.into();
    let mut rt_synths: Siv<_, _> = rt_synths.into();

    ctr_synths.ensure_capacity(num_slots);
    rt_synths.ensure_capacity(num_slots);

    (CtrGraph::new(ctr_synths), RtGraph::new(rt_synths))
}

pub fn prepare_synth(name: String, sample_rate: u32) -> (CtrSynth, synths::Dummy) {
    (CtrSynth { name }, synths::Dummy::new(sample_rate))
}
