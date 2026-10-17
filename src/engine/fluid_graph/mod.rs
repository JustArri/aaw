use crate::{
    engine::{graph::LocalId, resolve_asset_path},
    memory::Siv,
    synths,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub mod ctr;
pub mod rt;
pub mod stored;

pub async fn prepare(
    assets_dir: &Path,
    stored: stored::Graph,
    num_slots: usize,
    sample_rate: u32,
) -> synths::Result<(ctr::Graph, rt::Graph)> {
    let mut ctr_synths: HashMap<LocalId, ctr::Synth> = HashMap::new();
    let mut rt_synths: HashMap<LocalId, synths::Fluid> = HashMap::new();

    for (local_id, stored_synth) in stored.synths {
        let (ctr_synth, rt_synth) =
            prepare_synth(assets_dir, stored_synth.file_path, sample_rate).await?;
        ctr_synths.insert(local_id, ctr_synth);
        rt_synths.insert(local_id, rt_synth);
    }

    let mut ctr_synths: Siv<_, _> = ctr_synths.into();
    let mut rt_synths: Siv<_, _> = rt_synths.into();

    ctr_synths.ensure_capacity(num_slots);
    rt_synths.ensure_capacity(num_slots);

    Ok((ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths)))
}

pub async fn prepare_synth(
    assets_dir: &Path,
    relative_path: PathBuf,
    sample_rate: u32,
) -> synths::Result<(ctr::Synth, synths::Fluid)> {
    let file_path = resolve_asset_path(assets_dir, &relative_path)
        .await
        .map_err(|source| synths::Error::AssetPathResolution { relative_path, source })?;
    let mut rt_synth = synths::Fluid::load(sample_rate as f64, &file_path).await?;
    let info = rt_synth.soundfont_info()?;
    let program = info.first_program().unwrap_or_default();
    let _ = rt_synth.select_program(program);
    let ctr_synth = ctr::Synth { file_path, info, program };
    Ok((ctr_synth, rt_synth))
}

pub fn default(num_slots: usize) -> (ctr::Graph, rt::Graph) {
    let rt_synths = Siv::with_capacity(num_slots);
    let ctr_synths = Siv::with_capacity(num_slots);
    (ctr::Graph::new(ctr_synths), rt::Graph::new(rt_synths))
}
