use crate::{
    engine::{
        controller_graph::StoredController,
        drum_machine::MusicalTransport,
        effects::Effects,
        graph::{ControllerId, InstrumentId},
        instrument_graph::{CtrInstrument, CtrInstrumentGraph, CtrInstrumentKind},
    },
    synths::SoundFontProgram,
};
use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{collections::HashMap, path::PathBuf};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum StoredInstrumentKind {
    Dummy {
        max_voices: usize,
    },
    Fluid {
        #[schema(value_type = String)]
        file_path: PathBuf,

        program: SoundFontProgram,
        max_voices: usize,
    },
    Sfizz {
        #[schema(value_type = String)]
        file_path: PathBuf,

        max_voices: usize,
        preload_size: u32,
    },
    Clap {
        #[schema(value_type = String)]
        file_path: PathBuf,

        plugin_id: String,
    },
}

impl From<&CtrInstrumentKind> for StoredInstrumentKind {
    fn from(value: &CtrInstrumentKind) -> Self {
        match value {
            CtrInstrumentKind::Dummy { max_voices } => Self::Dummy { max_voices: *max_voices },
            CtrInstrumentKind::Fluid { file_path, program, max_voices, .. } => Self::Fluid {
                file_path: file_path.clone(),
                program: *program,
                max_voices: *max_voices,
            },
            CtrInstrumentKind::Sfizz { file_path, max_voices, preload_size, .. } => Self::Sfizz {
                file_path: file_path.clone(),
                max_voices: *max_voices,
                preload_size: *preload_size,
            },
            CtrInstrumentKind::Clap { file_path, plugin_id, .. } => {
                Self::Clap { file_path: file_path.clone(), plugin_id: plugin_id.clone() }
            }
        }
    }
}

impl From<CtrInstrumentKind> for StoredInstrumentKind {
    fn from(value: CtrInstrumentKind) -> Self {
        match value {
            CtrInstrumentKind::Dummy { max_voices } => Self::Dummy { max_voices },
            CtrInstrumentKind::Fluid { file_path, program, max_voices, .. } => {
                Self::Fluid { file_path, program, max_voices }
            }
            CtrInstrumentKind::Sfizz { file_path, max_voices, preload_size, .. } => {
                Self::Sfizz { file_path, max_voices, preload_size }
            }
            CtrInstrumentKind::Clap { file_path, plugin_id, .. } => {
                Self::Clap { file_path, plugin_id }
            }
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredInstrument {
    pub name: String,
    pub kind: StoredInstrumentKind,
    pub controllers: HashMap<ControllerId, StoredController>,
    pub effects: Effects,
    pub gain: f32,

    #[serde_as(as = "Box<[_; 128]>")]
    #[schema(value_type = [Option<u8>; 128])]
    pub last_controller_values: Box<[Option<u8>; 128]>,
}

impl From<&CtrInstrument> for StoredInstrument {
    fn from(value: &CtrInstrument) -> Self {
        Self {
            name: value.name.clone(),
            kind: (&value.kind).into(),
            controllers: value.controllers.map_ref(|controller| controller.into()).into(),
            effects: value.effects.clone(),
            gain: value.gain,
            last_controller_values: value.last_controller_values.clone(),
        }
    }
}

impl From<CtrInstrument> for StoredInstrument {
    fn from(value: CtrInstrument) -> Self {
        Self {
            name: value.name,
            kind: value.kind.into(),
            controllers: value.controllers.map(Into::into).into(),
            effects: value.effects,
            gain: value.gain,
            last_controller_values: value.last_controller_values,
        }
    }
}

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredInstrumentGraph {
    pub instruments: HashMap<InstrumentId, StoredInstrument>,
    pub global_effects: Effects,
    pub global_gain: f32,
    pub global_transposition: i8,
}

impl Default for StoredInstrumentGraph {
    fn default() -> Self {
        Self {
            instruments: Default::default(),
            global_effects: Default::default(),
            global_gain: 1.0,
            global_transposition: 0,
        }
    }
}

impl StoredInstrumentGraph {
    pub fn resize_drum_machines_to_bar(&mut self, transport: MusicalTransport) {
        for instrument in self.instruments.values_mut() {
            for controller in instrument.controllers.values_mut() {
                if let StoredController::DrumMachine(machine) = controller {
                    machine.resize_to_bar(transport);
                }
            }
        }
    }
}

impl From<&CtrInstrumentGraph> for StoredInstrumentGraph {
    fn from(value: &CtrInstrumentGraph) -> Self {
        Self {
            instruments: value.instruments.map_ref(|instrument| instrument.into()).into(),
            global_effects: value.global_effects.clone(),
            global_gain: value.global_gain,
            global_transposition: value.global_transposition,
        }
    }
}

impl From<CtrInstrumentGraph> for StoredInstrumentGraph {
    fn from(value: CtrInstrumentGraph) -> Self {
        Self {
            instruments: value.instruments.map(Into::into).into(),
            global_effects: value.global_effects,
            global_gain: value.global_gain,
            global_transposition: value.global_transposition,
        }
    }
}
