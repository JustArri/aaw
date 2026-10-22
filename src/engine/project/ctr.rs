use crate::engine::{
    instrument_graph::CtrInstrumentGraph, synth_graph::CtrSynthGraph, user_presets::CtrUserPresets,
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
    pub synth_graph: CtrSynthGraph,
    pub user_presets: CtrUserPresets,
}

impl CtrProject {
    pub fn next_revision(&mut self) -> u32 {
        (self.revision, _) = self.revision.overflowing_add(1);
        self.revision
    }
}
