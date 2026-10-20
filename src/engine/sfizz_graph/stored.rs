use crate::engine::{
    graph::LocalId,
    sfizz_graph::{CtrGraph, CtrSynth},
};
use migrations_derive::Migrations;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = StoredSfizzSynth)]
pub struct StoredSynth {
    pub name: String,

    #[schema(value_type = String)]
    pub file_path: PathBuf,
}

impl From<&CtrSynth> for StoredSynth {
    fn from(value: &CtrSynth) -> Self {
        Self { name: value.name.clone(), file_path: value.file_path.clone() }
    }
}

impl From<CtrSynth> for StoredSynth {
    fn from(value: CtrSynth) -> Self {
        Self { name: value.name, file_path: value.file_path }
    }
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = StoredSfizzGraph)]
pub struct StoredGraph {
    pub synths: HashMap<LocalId, StoredSynth>,
}

impl From<&CtrGraph> for StoredGraph {
    fn from(value: &CtrGraph) -> Self {
        Self { synths: value.synths.map_ref(|synth| synth.into()).into() }
    }
}

impl From<CtrGraph> for StoredGraph {
    fn from(value: CtrGraph) -> Self {
        Self { synths: value.synths.map(|synth| synth.into()).into() }
    }
}
