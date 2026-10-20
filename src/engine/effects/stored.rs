use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = StoredEffects)]
pub struct StoredEffects {
    pub effects: Vec<StoredEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
#[schema(as = StoredEffect)]
pub enum StoredEffect {
    Gain { value: f32 },
    LowPassFilter { frequency: f32, resonance: f32 },
    Equalizer { bands: Vec<StoredEqualizerBand> },
    LoadFailed { description: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
#[schema(as = StoredEqualizerBand)]
pub enum StoredEqualizerBand {
    LowShelf { frequency: f32, gain_db: f32, resonance: f32 },
    Peaking { frequency: f32, gain_db: f32, resonance: f32 },
    HighShelf { frequency: f32, gain_db: f32, resonance: f32 },
}
