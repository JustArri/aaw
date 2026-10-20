use crate::engine::{ctr::CtrProject, dummy_graph, fluid_graph, instrument_graph, sfizz_graph};
use migrations_derive::Migrations;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredProject {
    pub revision: u32,
    pub name: String,
    pub instrument_graph: instrument_graph::StoredGraph,
    pub dummy_graph: dummy_graph::StoredGraph,
    pub fluid_graph: fluid_graph::StoredGraph,
    pub sfizz_graph: sfizz_graph::StoredGraph,
}

impl From<&CtrProject> for StoredProject {
    fn from(value: &CtrProject) -> Self {
        Self {
            revision: value.revision,
            name: value.name.clone(),
            instrument_graph: (&value.instrument_graph).into(),
            dummy_graph: (&value.dummy_graph).into(),
            fluid_graph: (&value.fluid_graph).into(),
            sfizz_graph: (&value.sfizz_graph).into(),
        }
    }
}

impl From<CtrProject> for StoredProject {
    fn from(value: CtrProject) -> Self {
        Self {
            revision: value.revision,
            name: value.name,
            instrument_graph: value.instrument_graph.into(),
            dummy_graph: value.dummy_graph.into(),
            fluid_graph: value.fluid_graph.into(),
            sfizz_graph: value.sfizz_graph.into(),
        }
    }
}
