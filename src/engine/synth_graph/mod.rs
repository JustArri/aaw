use crate::{
    engine::{
        effects::{self, EffectsContext},
        graph::SynthId,
        resolve_asset_path,
    },
    memory::Siv,
    synths,
};
pub use ctr::*;
pub use rt::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
pub use stored::*;
use utoipa::ToSchema;

mod ctr;
mod rt;
mod stored;

#[derive(Debug, thiserror::Error)]
pub enum SynthGraphError {
    #[error(transparent)]
    Synth(#[from] synths::Error),

    #[error(transparent)]
    Effect(#[from] effects::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum SynthKind {
    Dummy,
    Fluid,
    Sfizz,
}

#[derive(Clone, Copy)]
pub struct SynthGraphContext<'a> {
    pub assets_dir: &'a Path,
    pub num_synths: usize,
    pub channel_count: usize,
    pub sample_rate: u32,
    pub max_block_size: usize,
}

#[derive(Clone, Copy)]
pub struct FluidSynthContext<'a> {
    pub assets_dir: &'a Path,
    pub sample_rate: f64,
}

impl<'a> From<&SynthGraphContext<'a>> for FluidSynthContext<'a> {
    fn from(value: &SynthGraphContext<'a>) -> Self {
        Self { assets_dir: value.assets_dir, sample_rate: value.sample_rate as f64 }
    }
}

#[derive(Clone, Copy)]
pub struct SfizzSynthContext<'a> {
    pub assets_dir: &'a Path,
    pub sample_rate: f32,
    pub max_block_size: usize,
}

impl<'a> From<&SynthGraphContext<'a>> for SfizzSynthContext<'a> {
    fn from(value: &SynthGraphContext<'a>) -> Self {
        Self {
            assets_dir: value.assets_dir,
            sample_rate: value.sample_rate as f32,
            max_block_size: value.max_block_size,
        }
    }
}

impl<'a> From<&SynthGraphContext<'a>> for EffectsContext {
    fn from(value: &SynthGraphContext<'a>) -> Self {
        Self { channel_count: value.channel_count, sample_rate: value.sample_rate }
    }
}

pub async fn prepare<'a>(
    stored: StoredSynthGraph,
    ctx: SynthGraphContext<'a>,
) -> Result<(CtrSynthGraph, RtSynthGraph), SynthGraphError> {
    let mut ctr_synths: HashMap<SynthId, CtrSynth> = HashMap::new();
    let mut rt_synths: HashMap<SynthId, RtSynth> = HashMap::new();

    for (local_id, stored_synth) in stored.synths {
        let name = stored_synth.name;
        let (effects, rt_effects) = effects::prepare(stored_synth.effects, (&ctx).into())?;

        let (parameters, kind) = match stored_synth.parameters {
            StoredSynthParameters::Dummy => prepare_dummy_synth(ctx.sample_rate),
            StoredSynthParameters::Fluid { file_path } => {
                prepare_fluid_synth(file_path, (&ctx).into()).await?
            }
            StoredSynthParameters::Sfizz { file_path } => {
                prepare_sfizz_synth(file_path, (&ctx).into()).await?
            }
        };

        ctr_synths.insert(local_id, CtrSynth { name, parameters, effects });
        rt_synths.insert(local_id, RtSynth { kind, effects: rt_effects });
    }

    let mut ctr_synths: Siv<_, _> = ctr_synths.into();
    let mut rt_synths: Siv<_, _> = rt_synths.into();

    ctr_synths.ensure_capacity(ctx.num_synths);
    rt_synths.ensure_capacity(ctx.num_synths);

    let (effects, rt_effects) = effects::prepare(stored.effects, (&ctx).into())?;

    Ok((CtrSynthGraph::new(ctr_synths, effects), RtSynthGraph::new(rt_synths, rt_effects)))
}

pub fn prepare_dummy_synth(sample_rate: u32) -> (CtrSynthParameters, RtSynthKind) {
    (CtrSynthParameters::Dummy, RtSynthKind::Dummy(synths::Dummy::new(sample_rate)))
}

pub async fn prepare_fluid_synth<'a>(
    relative_path: PathBuf,
    ctx: FluidSynthContext<'a>,
) -> synths::Result<(CtrSynthParameters, RtSynthKind)> {
    let file_path = resolve_asset_path(ctx.assets_dir, &relative_path).await.map_err(|source| {
        synths::Error::AssetPathResolution { relative_path: relative_path.clone(), source }
    })?;
    let mut synth = synths::Fluid::load(ctx.sample_rate, &file_path).await?;
    let info = synth.soundfont_info()?;
    let program = info.first_program().unwrap_or_default();
    let _ = synth.select_program(program);
    Ok((
        CtrSynthParameters::Fluid { file_path: relative_path, info, program },
        RtSynthKind::Fluid(synth),
    ))
}

pub async fn prepare_sfizz_synth<'a>(
    relative_path: PathBuf,
    ctx: SfizzSynthContext<'a>,
) -> synths::Result<(CtrSynthParameters, RtSynthKind)> {
    let file_path = resolve_asset_path(ctx.assets_dir, &relative_path).await.map_err(|source| {
        synths::Error::AssetPathResolution { relative_path: relative_path.clone(), source }
    })?;
    let mut synth = synths::Sfizz::load(ctx.sample_rate, ctx.max_block_size, &file_path).await?;
    let info = synth.sfz_info()?;
    Ok((CtrSynthParameters::Sfizz { file_path: relative_path, info }, RtSynthKind::Sfizz(synth)))
}

pub fn default(
    num_instruments: usize,
    ctx: EffectsContext,
) -> Result<(CtrSynthGraph, RtSynthGraph), SynthGraphError> {
    let rt_synths = Siv::with_capacity(num_instruments);
    let ctr_synths = Siv::with_capacity(num_instruments);
    let (effects, rt_effects) = effects::default(ctx)?;
    Ok((CtrSynthGraph::new(ctr_synths, effects), RtSynthGraph::new(rt_synths, rt_effects)))
}
