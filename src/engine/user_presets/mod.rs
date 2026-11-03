use crate::{
    engine::graph::{ControllerAddress, InstrumentId},
    synths::SoundFontProgram,
};
pub use ctr::*;
use serde::{Deserialize, Serialize};
pub use stored::*;
use utoipa::ToSchema;

mod ctr;
mod stored;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum UserPresetAction {
    DisableAllInstrumentControllers,
    SetInstrumentControllerEnabled { controller: ControllerAddress, flag: bool },
    SetSoundFontProgram { instrument: InstrumentId, program: SoundFontProgram },
    SetClapParameter { instrument: InstrumentId, parameter_id: u32, value: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserPreset {
    pub name: String,
    pub actions: Vec<UserPresetAction>,
}

pub fn prepare(stored: StoredUserPresets) -> CtrUserPresets {
    CtrUserPresets { entries: stored.entries }
}
