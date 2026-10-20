pub use ctr::*;
pub use rt::*;
use serde::{Deserialize, Serialize};
pub use stored::*;
use utoipa::ToSchema;

mod ctr;
mod rt;
mod stored;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct EffectsTuning {
    pub max_effects: usize,
    pub max_equalizer_bands: usize,
    pub filter_state_cutoff: f32,
}

pub const EFFECTS_TUNING: EffectsTuning =
    EffectsTuning { max_effects: 64, max_equalizer_bands: 16, filter_state_cutoff: 1.0e-20 };

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("effect queue contains {actual} effects, but its capacity is {maximum}")]
    TooManyEffects { actual: usize, maximum: usize },

    #[error("gain must be finite")]
    InvalidGain,

    #[error("low-pass frequency must be finite and greater than zero")]
    InvalidLowPassFrequency,

    #[error("low-pass resonance must be finite and greater than zero")]
    InvalidLowPassResonance,

    #[error("equalizer contains {actual} bands, but its capacity is {maximum}")]
    TooManyEqualizerBands { actual: usize, maximum: usize },

    #[error("equalizer frequency must be finite and greater than zero")]
    InvalidEqualizerFrequency,

    #[error("equalizer gain must be finite")]
    InvalidEqualizerGain,

    #[error("equalizer resonance must be finite and greater than zero")]
    InvalidEqualizerResonance,
}

pub fn prepare(
    stored: StoredEffects,
    channel_count: usize,
    sample_rate: u32,
) -> Result<(CtrEffects, RtEffects), Error> {
    let mut ctr = CtrEffects::from(stored);
    let rt = RtEffects::new_replacing_failed(&mut ctr, channel_count, sample_rate)?;
    Ok((ctr, rt))
}
