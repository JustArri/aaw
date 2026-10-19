use crate::engine::{dummy_graph::ctr, graph::LocalId};
use migrations_derive::Migrations;
use std::collections::HashMap;

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = FluidSynth)]
pub struct Synth {
    pub name: String,
}

impl From<ctr::Synth> for Synth {
    fn from(value: ctr::Synth) -> Self {
        Self { name: value.name }
    }
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = FluidGraph)]
pub struct Graph {
    pub synths: HashMap<LocalId, Synth>,
}

impl From<&ctr::Graph> for Graph {
    fn from(value: &ctr::Graph) -> Self {
        Self { synths: value.synths.map_ref(|synth| synth.clone().into()).into() }
    }
}

impl From<ctr::Graph> for Graph {
    fn from(value: ctr::Graph) -> Self {
        Self { synths: value.synths.map(|synth| synth.into()).into() }
    }
}
