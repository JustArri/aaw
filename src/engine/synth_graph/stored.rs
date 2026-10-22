use crate::engine::{
    effects::Effects,
    graph::SynthId,
    synth_graph::{CtrSynth, CtrSynthGraph, CtrSynthParameters},
};
use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum StoredSynthParameters {
    Dummy,
    Fluid {
        #[schema(value_type = String)]
        file_path: PathBuf,
    },
    Sfizz {
        #[schema(value_type = String)]
        file_path: PathBuf,
    },
}

impl From<&CtrSynthParameters> for StoredSynthParameters {
    fn from(value: &CtrSynthParameters) -> Self {
        match value {
            CtrSynthParameters::Dummy => StoredSynthParameters::Dummy,
            CtrSynthParameters::Fluid { file_path, .. } => {
                StoredSynthParameters::Fluid { file_path: file_path.clone() }
            }
            CtrSynthParameters::Sfizz { file_path, .. } => {
                StoredSynthParameters::Sfizz { file_path: file_path.clone() }
            }
        }
    }
}

impl From<CtrSynthParameters> for StoredSynthParameters {
    fn from(value: CtrSynthParameters) -> Self {
        match value {
            CtrSynthParameters::Dummy => StoredSynthParameters::Dummy,
            CtrSynthParameters::Fluid { file_path, .. } => {
                StoredSynthParameters::Fluid { file_path }
            }
            CtrSynthParameters::Sfizz { file_path, .. } => {
                StoredSynthParameters::Sfizz { file_path }
            }
        }
    }
}

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredSynth {
    pub name: String,
    pub parameters: StoredSynthParameters,
    pub effects: Effects,
}

impl From<&CtrSynth> for StoredSynth {
    fn from(value: &CtrSynth) -> Self {
        Self {
            name: value.name.clone(),
            parameters: (&value.parameters).into(),
            effects: value.effects.clone(),
        }
    }
}

impl From<CtrSynth> for StoredSynth {
    fn from(value: CtrSynth) -> Self {
        Self { name: value.name, parameters: value.parameters.into(), effects: value.effects }
    }
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredSynthGraph {
    pub synths: HashMap<SynthId, StoredSynth>,
    pub effects: Effects,
}

impl From<&CtrSynthGraph> for StoredSynthGraph {
    fn from(value: &CtrSynthGraph) -> Self {
        Self {
            synths: value.synths.map_ref(|synth| synth.into()).into(),
            effects: value.effects.clone(),
        }
    }
}

impl From<CtrSynthGraph> for StoredSynthGraph {
    fn from(value: CtrSynthGraph) -> Self {
        Self { synths: value.synths.map(|synth| synth.into()).into(), effects: value.effects }
    }
}
