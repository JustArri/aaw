use crate::{engine::graph::LocalId, memory::Siv, synths};
use std::collections::HashMap;

pub mod ctr;
pub mod rt;
pub mod stored;

pub fn prepare(
    stored: stored::Graph,
    num_slots: usize,
    sample_rate: u32,
) -> (ctr::Graph, rt::Graph) {
    let mut ctr_synths: HashMap<LocalId, ctr::Synth> = HashMap::new();
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

    (ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths))
}

pub fn prepare_synth(name: String, sample_rate: u32) -> (ctr::Synth, synths::Dummy) {
    (ctr::Synth { name }, synths::Dummy::new(sample_rate))
}
