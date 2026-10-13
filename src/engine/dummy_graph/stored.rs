use crate::engine::{dummy_graph::ctr, graph::LocalSynthId};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub ids: Vec<LocalSynthId>,
}

impl From<&ctr::Graph> for Graph {
    fn from(value: &ctr::Graph) -> Self {
        Self { ids: value.synths.ids().to_vec() }
    }
}

impl From<ctr::Graph> for Graph {
    fn from(value: ctr::Graph) -> Self {
        Self { ids: value.synths.ids().to_vec() }
    }
}
