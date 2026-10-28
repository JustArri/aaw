use crate::{
    engine::{
        effects::Effects,
        graph::SynthId,
        synth_graph::{CtrSynth, CtrSynthGraph, CtrSynthParameters},
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
pub enum StoredSynthParameters {
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
    },
}

impl From<&CtrSynthParameters> for StoredSynthParameters {
    fn from(value: &CtrSynthParameters) -> Self {
        match value {
            CtrSynthParameters::Dummy { max_voices } => {
                StoredSynthParameters::Dummy { max_voices: *max_voices }
            }
            CtrSynthParameters::Fluid { file_path, program: p, max_voices, .. } => {
                StoredSynthParameters::Fluid {
                    file_path: file_path.clone(),
                    program: *p,
                    max_voices: *max_voices,
                }
            }
            CtrSynthParameters::Sfizz { file_path, max_voices, .. } => {
                StoredSynthParameters::Sfizz {
                    file_path: file_path.clone(),
                    max_voices: *max_voices,
                }
            }
        }
    }
}

impl From<CtrSynthParameters> for StoredSynthParameters {
    fn from(value: CtrSynthParameters) -> Self {
        match value {
            CtrSynthParameters::Dummy { max_voices } => StoredSynthParameters::Dummy { max_voices },
            CtrSynthParameters::Fluid { file_path, program, max_voices, .. } => {
                StoredSynthParameters::Fluid { file_path, program, max_voices }
            }
            CtrSynthParameters::Sfizz { file_path, max_voices, .. } => {
                StoredSynthParameters::Sfizz { file_path, max_voices }
            }
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredSynth {
    pub name: String,
    pub parameters: StoredSynthParameters,
    pub effects: Effects,
    pub gain: f32,

    #[serde_as(as = "Box<[_; 128]>")]
    #[schema(value_type = [Option<u8>; 128])]
    pub last_controller_values: Box<[Option<u8>; 128]>,
}

impl From<&CtrSynth> for StoredSynth {
    fn from(value: &CtrSynth) -> Self {
        Self {
            name: value.name.clone(),
            parameters: (&value.parameters).into(),
            effects: value.effects.clone(),
            gain: value.gain,
            last_controller_values: value.last_controller_values.clone(),
        }
    }
}

impl From<CtrSynth> for StoredSynth {
    fn from(value: CtrSynth) -> Self {
        Self {
            name: value.name,
            parameters: value.parameters.into(),
            effects: value.effects,
            gain: value.gain,
            last_controller_values: value.last_controller_values,
        }
    }
}

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredSynthGraph {
    pub synths: HashMap<SynthId, StoredSynth>,
    pub global_effects: Effects,
    pub global_gain: f32,
}

impl Default for StoredSynthGraph {
    fn default() -> Self {
        Self { synths: Default::default(), global_effects: Default::default(), global_gain: 1.0 }
    }
}

impl From<&CtrSynthGraph> for StoredSynthGraph {
    fn from(value: &CtrSynthGraph) -> Self {
        Self {
            synths: value.synths.map_ref(|synth| synth.into()).into(),
            global_effects: value.global_effects.clone(),
            global_gain: value.global_gain,
        }
    }
}

impl From<CtrSynthGraph> for StoredSynthGraph {
    fn from(value: CtrSynthGraph) -> Self {
        Self {
            synths: value.synths.map(|synth| synth.into()).into(),
            global_effects: value.global_effects,
            global_gain: value.global_gain,
        }
    }
}
