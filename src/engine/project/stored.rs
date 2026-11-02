use crate::engine::{
    drum_machine::MusicalTransport, instrument_graph::StoredInstrumentGraph, project::CtrProject,
    user_presets::StoredUserPresets,
};
use migrations_derive::Migrations;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredProject {
    pub revision: u32,
    pub name: String,
    pub instrument_graph: StoredInstrumentGraph,
    pub user_presets: StoredUserPresets,
    pub transport: MusicalTransport,
}

impl From<&CtrProject> for StoredProject {
    fn from(value: &CtrProject) -> Self {
        Self {
            revision: value.revision,
            name: value.name.clone(),
            instrument_graph: (&value.instrument_graph).into(),
            user_presets: (&value.user_presets).into(),
            transport: value.transport,
        }
    }
}

impl From<CtrProject> for StoredProject {
    fn from(value: CtrProject) -> Self {
        Self {
            revision: value.revision,
            name: value.name,
            instrument_graph: value.instrument_graph.into(),
            user_presets: value.user_presets.into(),
            transport: value.transport,
        }
    }
}
