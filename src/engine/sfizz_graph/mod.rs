use crate::{MAX_BLOCK_SIZE, engine::graph::LocalId, memory::Siv, synths};
use std::{collections::HashMap, path::PathBuf};

pub mod ctr;
pub mod rt;
pub mod stored;

pub async fn prepare(
    stored: stored::Graph,
    num_slots: usize,
    sample_rate: u32,
) -> synths::Result<(ctr::Graph, rt::Graph)> {
    let mut rt_synths: HashMap<LocalId, synths::Sfizz> = HashMap::new();
    let mut ctr_synths: HashMap<LocalId, ctr::Synth> = HashMap::new();

    for (local_id, stored_synth) in stored.synths {
        let (ctr_synth, rt_synth) = prepare_synth(stored_synth.file_path, sample_rate).await?;
        ctr_synths.insert(local_id, ctr_synth);
        rt_synths.insert(local_id, rt_synth);
    }

    let mut rt_synths: Siv<_, _> = rt_synths.into();
    let mut ctr_synths: Siv<_, _> = ctr_synths.into();

    rt_synths.ensure_capacity(num_slots);
    ctr_synths.ensure_capacity(num_slots);

    Ok((ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths)))
}

pub async fn prepare_synth(
    file_path: PathBuf,
    sample_rate: u32,
) -> synths::Result<(ctr::Synth, synths::Sfizz)> {
    let mut rt_synth = synths::Sfizz::load(sample_rate as f32, MAX_BLOCK_SIZE, &file_path).await?;
    let info = rt_synth.sfz_info()?;
    let ctr_synth = ctr::Synth { file_path, info };
    Ok((ctr_synth, rt_synth))
}

pub fn default(num_slots: usize) -> (ctr::Graph, rt::Graph) {
    let rt_synths = Siv::with_capacity(num_slots);
    let ctr_synths = Siv::with_capacity(num_slots);
    (ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths))
}
