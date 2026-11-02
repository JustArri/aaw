use crate::{
    engine::{
        controller_graph::{self, RtControllerGraph, RtInstrumentControllers},
        drum_machine::RtDrumMachine,
        effects::{self, EffectsContext},
        graph::{ControllerId, InstrumentId},
        resolve_asset_path,
    },
    memory::Siv,
    synths::{self, SoundFontProgram},
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

#[derive(Debug, thiserror::Error)]
pub enum InstrumentGraphError {
    #[error(transparent)]
    Instrument(#[from] synths::Error),

    #[error(transparent)]
    Effect(#[from] effects::Error),
}

#[derive(Clone, Copy)]
pub struct InstrumentGraphContext<'a> {
    pub assets_dir: &'a Path,
    pub num_instruments: usize,
    pub num_controllers: usize,
    pub channel_count: usize,
    pub sample_rate: u32,
    pub max_block_size: usize,
}

#[derive(Clone, Copy)]
pub struct DummyInstrumentContext {
    pub sample_rate: f32,
}

#[derive(Clone, Copy)]
pub struct FluidInstrumentContext<'a> {
    pub assets_dir: &'a Path,
    pub sample_rate: f64,
}

#[derive(Clone, Copy)]
pub struct SfizzInstrumentContext<'a> {
    pub assets_dir: &'a Path,
    pub sample_rate: f32,
    pub max_block_size: usize,
}

#[derive(Clone, Copy)]
pub struct ClapInstrumentContext<'a> {
    pub assets_dir: &'a Path,
    pub sample_rate: f64,
}

impl From<&InstrumentGraphContext<'_>> for EffectsContext {
    fn from(value: &InstrumentGraphContext<'_>) -> Self {
        Self { channel_count: value.channel_count, sample_rate: value.sample_rate }
    }
}

pub async fn prepare(
    stored: StoredInstrumentGraph,
    ctx: InstrumentGraphContext<'_>,
) -> Result<(CtrInstrumentGraph, RtInstrumentGraph, RtControllerGraph), InstrumentGraphError> {
    let mut ctr_instruments = HashMap::new();
    let mut rt_instruments = HashMap::new();
    let mut rt_controller_instruments = HashMap::new();

    for (id, stored) in stored.instruments {
        let mut ctr_controllers = HashMap::new();
        let mut rt_controllers = HashMap::new();
        for (controller_id, stored_controller) in stored.controllers {
            let (ctr, rt) = controller_graph::prepare(stored_controller);
            ctr_controllers.insert(controller_id, ctr);
            rt_controllers.insert(controller_id, rt);
        }
        let mut ctr_controllers: Siv<_, ControllerId> = ctr_controllers.into();
        let mut rt_controllers: Siv<_, ControllerId> = rt_controllers.into();
        let mut rt_drum_machines: Siv<_, ControllerId> = ctr_controllers.map_ref(|controller| {
            crate::engine::drum_machine::RtDrumMachineSlot::new(
                controller.drum_machine().map(RtDrumMachine::compile),
            )
        });
        ctr_controllers.ensure_capacity(ctx.num_controllers);
        rt_controllers.ensure_capacity(ctx.num_controllers);
        rt_drum_machines.ensure_capacity(ctx.num_controllers);

        let (effects, rt_effects) = effects::prepare(stored.effects, (&ctx).into())?;
        let (kind, rt_kind) = prepare_kind(stored.kind, &ctx).await?;
        ctr_instruments.insert(
            id,
            CtrInstrument {
                name: stored.name,
                kind,
                controllers: ctr_controllers,
                effects,
                gain: stored.gain,
                last_controller_values: stored.last_controller_values,
            },
        );
        rt_instruments.insert(
            id,
            RtInstrument {
                kind: rt_kind,
                effects: rt_effects,
                gain: stored.gain,
                drum_machines: rt_drum_machines,
            },
        );
        rt_controller_instruments
            .insert(id, RtInstrumentControllers { controllers: rt_controllers });
    }

    let mut ctr_instruments: Siv<_, InstrumentId> = ctr_instruments.into();
    let mut rt_instruments: Siv<_, InstrumentId> = rt_instruments.into();
    let mut rt_controller_instruments: Siv<_, InstrumentId> = rt_controller_instruments.into();
    ctr_instruments.ensure_capacity(ctx.num_instruments);
    rt_instruments.ensure_capacity(ctx.num_instruments);
    rt_controller_instruments.ensure_capacity(ctx.num_instruments);
    let (effects, rt_effects) = effects::prepare(stored.global_effects, (&ctx).into())?;

    Ok((
        CtrInstrumentGraph::new(
            ctr_instruments,
            effects,
            stored.global_gain,
            stored.global_transposition,
        ),
        RtInstrumentGraph::new(rt_instruments, rt_effects, stored.global_gain),
        RtControllerGraph {
            instruments: rt_controller_instruments,
            global_transposition: stored.global_transposition,
        },
    ))
}

async fn prepare_kind(
    stored: StoredInstrumentKind,
    ctx: &InstrumentGraphContext<'_>,
) -> Result<(CtrInstrumentKind, RtInstrumentKind), synths::Error> {
    match stored {
        StoredInstrumentKind::Dummy { max_voices } => Ok(prepare_dummy_instrument(
            max_voices,
            DummyInstrumentContext { sample_rate: ctx.sample_rate as f32 },
        )),
        StoredInstrumentKind::Fluid { file_path, program, max_voices } => {
            prepare_fluid_instrument(
                file_path,
                Some(program),
                max_voices,
                FluidInstrumentContext {
                    assets_dir: ctx.assets_dir,
                    sample_rate: ctx.sample_rate as f64,
                },
            )
            .await
        }
        StoredInstrumentKind::Sfizz { file_path, max_voices, preload_size } => {
            prepare_sfizz_instrument(
                file_path,
                max_voices,
                preload_size,
                SfizzInstrumentContext {
                    assets_dir: ctx.assets_dir,
                    sample_rate: ctx.sample_rate as f32,
                    max_block_size: ctx.max_block_size,
                },
            )
            .await
        }
        StoredInstrumentKind::Clap { file_path, plugin_id } => {
            prepare_clap_instrument(
                file_path,
                plugin_id,
                ClapInstrumentContext {
                    assets_dir: ctx.assets_dir,
                    sample_rate: ctx.sample_rate as f64,
                },
            )
            .await
        }
    }
}

pub fn prepare_dummy_instrument(
    max_voices: usize,
    ctx: DummyInstrumentContext,
) -> (CtrInstrumentKind, RtInstrumentKind) {
    (
        CtrInstrumentKind::Dummy { max_voices },
        RtInstrumentKind::Dummy(synths::Dummy::new(ctx.sample_rate, max_voices)),
    )
}

pub async fn prepare_fluid_instrument(
    relative_path: PathBuf,
    program: Option<SoundFontProgram>,
    max_voices: usize,
    ctx: FluidInstrumentContext<'_>,
) -> synths::Result<(CtrInstrumentKind, RtInstrumentKind)> {
    let file_path = resolve(ctx.assets_dir, &relative_path).await?;
    let mut instrument = synths::Fluid::load(ctx.sample_rate, max_voices, &file_path).await?;
    let info = instrument.soundfont_info()?;
    let program = program
        .filter(|program| info.has_program(*program))
        .unwrap_or_else(|| info.first_program().unwrap_or_default());
    let _ = instrument.select_program(program);
    Ok((
        CtrInstrumentKind::Fluid { file_path: relative_path, info, program, max_voices },
        RtInstrumentKind::Fluid(instrument),
    ))
}

pub async fn prepare_sfizz_instrument(
    relative_path: PathBuf,
    max_voices: usize,
    preload_size: u32,
    ctx: SfizzInstrumentContext<'_>,
) -> synths::Result<(CtrInstrumentKind, RtInstrumentKind)> {
    let file_path = resolve(ctx.assets_dir, &relative_path).await?;
    let mut instrument = synths::Sfizz::load(
        ctx.sample_rate,
        ctx.max_block_size,
        max_voices,
        preload_size,
        &file_path,
    )
    .await?;
    let info = instrument.sfz_info()?;
    Ok((
        CtrInstrumentKind::Sfizz { file_path: relative_path, info, max_voices, preload_size },
        RtInstrumentKind::Sfizz(instrument),
    ))
}

pub async fn prepare_clap_instrument(
    relative_path: PathBuf,
    plugin_id: String,
    ctx: ClapInstrumentContext<'_>,
) -> synths::Result<(CtrInstrumentKind, RtInstrumentKind)> {
    let file_path = resolve(ctx.assets_dir, &relative_path).await?;
    let mut instrument = synths::Clap::load(ctx.sample_rate, file_path, plugin_id.clone()).await?;
    let kind = CtrInstrumentKind::Clap {
        file_path: relative_path,
        plugin_id,
        plugin_name: instrument.plugin_name().map(str::to_owned),
        plugin_version: instrument.plugin_version().map(str::to_owned),
        plugin_parameters: instrument.take_parameters(),
    };
    Ok((kind, RtInstrumentKind::Clap(instrument)))
}

async fn resolve(assets_dir: &Path, relative_path: &Path) -> synths::Result<PathBuf> {
    resolve_asset_path(assets_dir, relative_path).await.map_err(|source| {
        synths::Error::AssetPathResolution { relative_path: relative_path.to_owned(), source }
    })
}

pub fn default(
    num_instruments: usize,
    effects_context: EffectsContext,
) -> Result<(CtrInstrumentGraph, RtInstrumentGraph, RtControllerGraph), InstrumentGraphError> {
    let (effects, rt_effects) = effects::default(effects_context)?;
    Ok((
        CtrInstrumentGraph::new(Siv::with_capacity(num_instruments), effects, 1.0, 0),
        RtInstrumentGraph::new(Siv::with_capacity(num_instruments), rt_effects, 1.0),
        RtControllerGraph {
            instruments: Siv::with_capacity(num_instruments),
            global_transposition: 0,
        },
    ))
}
