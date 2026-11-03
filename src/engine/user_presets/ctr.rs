use crate::engine::user_presets::UserPreset;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = UserPresets)]
pub struct CtrUserPresets {
    pub entries: Vec<UserPreset>,
}
