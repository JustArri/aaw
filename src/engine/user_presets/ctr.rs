use crate::engine::graph::InstrumentId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = UserPreset)]
pub struct CtrUserPreset {
    pub name: String,
    pub instruments: Vec<InstrumentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = UserPresets)]
pub struct CtrUserPresets {
    pub entries: Vec<CtrUserPreset>,
}
