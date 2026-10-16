use crate::engine::{ctr, dummy_graph, fluid_graph, instrument_graph, sfizz_graph};
use migrations_derive::Migrations;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct Project {
    pub instrument_graph: instrument_graph::stored::Graph,
    pub dummy_graph: dummy_graph::stored::Graph,
    pub fluid_graph: fluid_graph::stored::Graph,
    pub sfizz_graph: sfizz_graph::stored::Graph,
}

impl From<&ctr::Project> for Project {
    fn from(value: &ctr::Project) -> Self {
        Self {
            instrument_graph: (&value.instrument_graph).into(),
            dummy_graph: (&value.dummy_graph).into(),
            fluid_graph: (&value.fluid_graph).into(),
            sfizz_graph: (&value.sfizz_graph).into(),
        }
    }
}

impl From<ctr::Project> for Project {
    fn from(value: ctr::Project) -> Self {
        Self {
            instrument_graph: value.instrument_graph.into(),
            dummy_graph: value.dummy_graph.into(),
            fluid_graph: value.fluid_graph.into(),
            sfizz_graph: value.sfizz_graph.into(),
        }
    }
}
