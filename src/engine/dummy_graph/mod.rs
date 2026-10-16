use crate::{memory::Siv, synths};
use std::collections::HashMap;

pub mod ctr;
pub mod rt;
pub mod stored;

pub fn prepare(
    stored: stored::Graph,
    num_slots: usize,
    sample_rate: u32,
) -> (ctr::Graph, rt::Graph) {
    let rt_synths =
        HashMap::from_iter(stored.ids.iter().map(|id| (*id, synths::Dummy::new(sample_rate))));
    let ctr_synths = HashMap::from_iter(stored.ids.iter().map(|id| (*id, ctr::Synth)));

    let mut rt_synths: Siv<_, _> = rt_synths.into();
    let mut ctr_synths: Siv<_, _> = ctr_synths.into();

    rt_synths.ensure_capacity(num_slots);
    ctr_synths.ensure_capacity(num_slots);

    (ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths))
}

pub fn prepare_synth(sample_rate: u32) -> (ctr::Synth, synths::Dummy) {
    (ctr::Synth, synths::Dummy::new(sample_rate))
}
