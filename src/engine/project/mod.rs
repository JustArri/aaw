use crate::{
    engine::{
        effects::EffectsContext,
        instrument_graph::{self, InstrumentGraphContext},
        synth_graph::{self, SynthGraphContext, SynthGraphError},
        user_presets,
    },
    server::api::audio::AudioOutputStreamConfig,
};
pub use ctr::*;
pub use rt::*;
use std::path::Path;
pub use stored::*;

mod ctr;
mod rt;
mod stored;

pub struct ProjectAsset {
    pub project: StoredProject,
    pub storage: Option<CtrProjectStorage>,
}

#[derive(Clone, Copy)]
pub struct ProjectContext<'a> {
    pub assets_dir: &'a Path,
    pub min_instruments: usize,
    pub min_synths: usize,
    pub stream_config: AudioOutputStreamConfig,
}

impl<'a> From<&ProjectContext<'a>> for InstrumentGraphContext {
    fn from(value: &ProjectContext<'a>) -> Self {
        Self {
            num_instruments: value.min_instruments,
            channel_count: value.stream_config.channel_count,
            sample_rate: value.stream_config.sample_rate,
        }
    }
}

impl<'a> From<&ProjectContext<'a>> for SynthGraphContext<'a> {
    fn from(value: &ProjectContext<'a>) -> Self {
        Self {
            assets_dir: value.assets_dir,
            num_synths: value.min_synths,
            channel_count: value.stream_config.channel_count,
            sample_rate: value.stream_config.sample_rate,
            max_block_size: value.stream_config.max_block_size,
        }
    }
}

pub async fn prepare<'a>(
    asset: ProjectAsset,
    ctx: ProjectContext<'a>,
) -> Result<(CtrProject, RtProject), SynthGraphError> {
    let (ctr_instrument_graph, rt_instrument_graph) =
        instrument_graph::prepare(asset.project.instrument_graph, (&ctx).into());

    let sg_res = synth_graph::prepare(asset.project.synth_graph, (&ctx).into()).await;
    let (ctr_synth_graph, rt_synth_graph) = match sg_res {
        Ok(res) => res,
        Err(_) => synth_graph::default(
            ctx.min_synths,
            EffectsContext {
                channel_count: ctx.stream_config.channel_count,
                sample_rate: ctx.stream_config.sample_rate,
            },
        )?,
    };

    let ctr_project = CtrProject {
        storage: asset.storage,
        revision: asset.project.revision,
        name: asset.project.name,
        instrument_graph: ctr_instrument_graph,
        synth_graph: ctr_synth_graph,
        user_presets: user_presets::prepare(asset.project.user_presets),
    };

    let rt_project =
        RtProject { instrument_graph: rt_instrument_graph, synth_graph: rt_synth_graph };

    Ok((ctr_project, rt_project))
}
