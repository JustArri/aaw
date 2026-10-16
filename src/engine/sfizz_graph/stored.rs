use crate::engine::{graph::LocalId, sfizz_graph::ctr};
use migrations_derive::Migrations;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = SfizzSynth)]
pub struct Synth {
    #[schema(value_type = String)]
    pub file_path: PathBuf,
}

impl From<ctr::Synth> for Synth {
    fn from(value: ctr::Synth) -> Self {
        Self { file_path: value.file_path }
    }
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = SfizzGraph)]
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
