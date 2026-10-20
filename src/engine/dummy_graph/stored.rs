use crate::engine::{
    dummy_graph::{CtrGraph, CtrSynth},
    graph::LocalId,
};
use migrations_derive::Migrations;
use std::collections::HashMap;

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = StoredDummySynth)]
pub struct StoredSynth {
    pub name: String,
}

impl From<&CtrSynth> for StoredSynth {
    fn from(value: &CtrSynth) -> Self {
        Self { name: value.name.clone() }
    }
}

impl From<CtrSynth> for StoredSynth {
    fn from(value: CtrSynth) -> Self {
        Self { name: value.name }
    }
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = StoredDummyGraph)]
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
