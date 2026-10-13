use crate::{
    engine::graph::{DEFAULT_SAMPLE_RATE, LocalSynthId},
    memory::Siv,
    synths,
};
use std::collections::HashMap;

pub mod ctr;
pub mod rt;
pub mod stored;

pub async fn prepare(
    stored: stored::Graph,
    num_slots: usize,
) -> synths::Result<(ctr::Graph, rt::Graph)> {
    let mut ctr_synths: HashMap<LocalSynthId, ctr::Synth> = HashMap::new();
    let mut rt_synths: HashMap<LocalSynthId, synths::Fluid> = HashMap::new();

    for (local_id, stored_synth) in stored.synths {
        let mut synth =
            synths::Fluid::load(DEFAULT_SAMPLE_RATE as f64, &stored_synth.file_path).await?;
        let info = synth.soundfont_info()?;

        ctr_synths.insert(local_id, ctr::Synth { file_path: stored_synth.file_path, info });
        rt_synths.insert(local_id, synth);
    }

    let mut ctr_synths: Siv<_, _> = ctr_synths.into();
    let mut rt_synths: Siv<_, _> = rt_synths.into();

    ctr_synths.ensure_capacity(num_slots);
    rt_synths.ensure_capacity(num_slots);

    Ok((ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths)))
}

pub fn default(num_slots: usize) -> (ctr::Graph, rt::Graph) {
    let rt_synths = Siv::with_capacity(num_slots);
    let ctr_synths = Siv::with_capacity(num_slots);
    (ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths))
}
