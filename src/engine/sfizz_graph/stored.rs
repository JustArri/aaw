use crate::engine::{graph::LocalSynthId, sfizz_graph::ctr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = SfizzSynth)]
pub struct Synth {
    pub file_path: String,
}

impl From<ctr::Synth> for Synth {
    fn from(value: ctr::Synth) -> Self {
        Self { file_path: value.file_path }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = SfizzGraph)]
pub struct Graph {
    pub synths: HashMap<LocalSynthId, Synth>,
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
