use crate::{
    MAX_BLOCK_SIZE,
    engine::{graph::LocalId, resolve_asset_path},
    memory::Siv,
    synths,
};
pub use ctr::*;
pub use rt::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
pub use stored::*;

mod ctr;
mod rt;
mod stored;

pub async fn prepare(
    assets_dir: &Path,
    stored: StoredGraph,
    num_slots: usize,
    sample_rate: u32,
) -> synths::Result<(CtrGraph, RtGraph)> {
    let mut rt_synths: HashMap<LocalId, synths::Sfizz> = HashMap::new();
    let mut ctr_synths: HashMap<LocalId, CtrSynth> = HashMap::new();

    for (local_id, stored_synth) in stored.synths {
        let (ctr_synth, rt_synth) =
            prepare_synth(assets_dir, stored_synth.name, stored_synth.file_path, sample_rate)
                .await?;
        ctr_synths.insert(local_id, ctr_synth);
        rt_synths.insert(local_id, rt_synth);
    }

    let mut rt_synths: Siv<_, _> = rt_synths.into();
    let mut ctr_synths: Siv<_, _> = ctr_synths.into();

    rt_synths.ensure_capacity(num_slots);
    ctr_synths.ensure_capacity(num_slots);

    Ok((CtrGraph::new(ctr_synths), RtGraph::new(rt_synths)))
}

pub async fn prepare_synth(
    assets_dir: &Path,
    name: String,
    relative_path: PathBuf,
    sample_rate: u32,
) -> synths::Result<(CtrSynth, synths::Sfizz)> {
    let file_path = resolve_asset_path(assets_dir, &relative_path).await.map_err(|source| {
        synths::Error::AssetPathResolution { relative_path: relative_path.clone(), source }
    })?;
    let mut rt_synth = synths::Sfizz::load(sample_rate as f32, MAX_BLOCK_SIZE, &file_path).await?;
    let info = rt_synth.sfz_info()?;
    let ctr_synth = CtrSynth { name, file_path: relative_path, info };
    Ok((ctr_synth, rt_synth))
}

pub fn default(num_slots: usize) -> (CtrGraph, RtGraph) {
    let rt_synths = Siv::with_capacity(num_slots);
    let ctr_synths = Siv::with_capacity(num_slots);
    (CtrGraph::new(ctr_synths), RtGraph::new(rt_synths))
}
