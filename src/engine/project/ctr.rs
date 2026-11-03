use crate::engine::{
    drum_machine::MusicalTransport, instrument_graph::CtrInstrumentGraph,
    midi_app_controls::CtrMidiProjectControls, user_presets::CtrUserPresets,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CtrProjectStorage {
    pub id: uuid::Uuid,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = Project)]
pub struct CtrProject {
    #[schema(required)]
    pub storage: Option<CtrProjectStorage>,

    pub revision: u32,
    pub name: String,
    pub instrument_graph: CtrInstrumentGraph,
    pub user_presets: CtrUserPresets,
    pub midi_app_controls: CtrMidiProjectControls,
    pub transport: MusicalTransport,
}

impl CtrProject {
    pub fn next_revision(revision: &mut u32) -> u32 {
        (*revision, _) = revision.overflowing_add(1);
        *revision
    }
}
