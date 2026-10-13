use crate::{
    engine::graph::{DEFAULT_BLOCK_SIZE, DEFAULT_SAMPLE_RATE, LocalSynthId},
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
    let mut rt_synths: HashMap<LocalSynthId, synths::Sfizz> = HashMap::new();
    let mut ctr_synths: HashMap<LocalSynthId, ctr::Synth> = HashMap::new();

    for (local_id, stored_synth) in stored.synths {
        let mut synth = synths::Sfizz::load(
            DEFAULT_SAMPLE_RATE as f32,
            DEFAULT_BLOCK_SIZE,
            &stored_synth.file_path,
        )
        .await?;
        let info = synth.sfz_info()?;

        ctr_synths.insert(local_id, ctr::Synth { file_path: stored_synth.file_path, info });
        rt_synths.insert(local_id, synth);
    }

    let mut rt_synths: Siv<_, _> = rt_synths.into();
    let mut ctr_synths: Siv<_, _> = ctr_synths.into();

    rt_synths.ensure_capacity(num_slots);
    ctr_synths.ensure_capacity(num_slots);

    Ok((ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths)))
}

pub fn default(num_slots: usize) -> (ctr::Graph, rt::Graph) {
    let rt_synths = Siv::with_capacity(num_slots);
    let ctr_synths = Siv::with_capacity(num_slots);
    (ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths))
}
