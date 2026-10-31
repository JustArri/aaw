use crate::engine::graph::ControllerAddress;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = UserPreset)]
pub struct CtrUserPreset {
    pub name: String,
    pub controllers: Vec<ControllerAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = UserPresets)]
pub struct CtrUserPresets {
    pub entries: Vec<CtrUserPreset>,
}
