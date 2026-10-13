use crate::{engine::graph::DEFAULT_SAMPLE_RATE, memory::Siv, synths};
use std::collections::HashMap;

pub mod ctr;
pub mod rt;
pub mod stored;

pub fn prepare(stored: stored::Graph, num_slots: usize) -> (ctr::Graph, rt::Graph) {
    let rt_synths = HashMap::from_iter(
        stored.ids.iter().map(|id| (*id, synths::Dummy::new(DEFAULT_SAMPLE_RATE))),
    );
    let ctr_synths = HashMap::from_iter(stored.ids.iter().map(|id| (*id, ctr::Synth)));

    let mut rt_synths: Siv<_, _> = rt_synths.into();
    let mut ctr_synths: Siv<_, _> = ctr_synths.into();

    rt_synths.ensure_capacity(num_slots);
    ctr_synths.ensure_capacity(num_slots);

    (ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths))
}
