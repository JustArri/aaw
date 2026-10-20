use crate::engine::effects::{StoredEffect, StoredEffects, StoredEqualizerBand};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CtrEffect {
    Gain { value: f32 },
    LowPassFilter { frequency: f32, resonance: f32 },
    Equalizer { bands: Vec<CtrEqualizerBand> },
    LoadFailed { description: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CtrEqualizerBand {
    LowShelf { frequency: f32, gain_db: f32, resonance: f32 },
    Peaking { frequency: f32, gain_db: f32, resonance: f32 },
    HighShelf { frequency: f32, gain_db: f32, resonance: f32 },
}

impl From<&CtrEffect> for StoredEffect {
    fn from(value: &CtrEffect) -> Self {
        match value {
            CtrEffect::Gain { value } => Self::Gain { value: *value },
            CtrEffect::LowPassFilter { frequency, resonance } => {
                Self::LowPassFilter { frequency: *frequency, resonance: *resonance }
            }
            CtrEffect::Equalizer { bands } => {
                Self::Equalizer { bands: bands.iter().map(Into::into).collect() }
            }
            CtrEffect::LoadFailed { description } => {
                Self::LoadFailed { description: description.clone() }
            }
        }
    }
}

impl From<StoredEffect> for CtrEffect {
    fn from(value: StoredEffect) -> Self {
        match value {
            StoredEffect::Gain { value } => Self::Gain { value },
            StoredEffect::LowPassFilter { frequency, resonance } => {
                Self::LowPassFilter { frequency, resonance }
            }
            StoredEffect::Equalizer { bands } => {
                Self::Equalizer { bands: bands.into_iter().map(Into::into).collect() }
            }
            StoredEffect::LoadFailed { description } => Self::LoadFailed { description },
        }
    }
}

impl From<&CtrEqualizerBand> for StoredEqualizerBand {
    fn from(value: &CtrEqualizerBand) -> Self {
        match *value {
            CtrEqualizerBand::LowShelf { frequency, gain_db, resonance } => {
                Self::LowShelf { frequency, gain_db, resonance }
            }
            CtrEqualizerBand::Peaking { frequency, gain_db, resonance } => {
                Self::Peaking { frequency, gain_db, resonance }
            }
            CtrEqualizerBand::HighShelf { frequency, gain_db, resonance } => {
                Self::HighShelf { frequency, gain_db, resonance }
            }
        }
    }
}

impl From<StoredEqualizerBand> for CtrEqualizerBand {
    fn from(value: StoredEqualizerBand) -> Self {
        match value {
            StoredEqualizerBand::LowShelf { frequency, gain_db, resonance } => {
                Self::LowShelf { frequency, gain_db, resonance }
            }
            StoredEqualizerBand::Peaking { frequency, gain_db, resonance } => {
                Self::Peaking { frequency, gain_db, resonance }
            }
            StoredEqualizerBand::HighShelf { frequency, gain_db, resonance } => {
                Self::HighShelf { frequency, gain_db, resonance }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CtrEffects {
    pub effects: Vec<CtrEffect>,
}

impl From<&CtrEffects> for StoredEffects {
    fn from(value: &CtrEffects) -> Self {
        Self { effects: value.effects.iter().map(Into::into).collect() }
    }
}

impl From<StoredEffects> for CtrEffects {
    fn from(value: StoredEffects) -> Self {
        Self { effects: value.effects.into_iter().map(Into::into).collect() }
    }
}
