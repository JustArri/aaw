use crate::engine::{dummy_graph::ctr, graph::LocalId};
use migrations_derive::Migrations;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = DummyGraph)]
pub struct Graph {
    pub ids: Vec<LocalId>,
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
