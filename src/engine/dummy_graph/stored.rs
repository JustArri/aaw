use crate::engine::{dummy_graph::ctr, graph::LocalSynthId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = DummyGraph)]
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
