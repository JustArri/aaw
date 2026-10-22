pub use ctr::*;
pub use rt::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod ctr;
mod rt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct EffectsTuning {
    pub max_effects: usize,
    pub max_equalizer_bands: usize,
    pub max_delay_ms: f32,
    pub max_chorus_delay_ms: f32,
    pub filter_state_cutoff: f32,
    pub reverb_reference_sample_rate: u32,
    pub reverb_comb_delays: [usize; 4],
    pub reverb_all_pass_delays: [usize; 2],
    pub reverb_all_pass_feedback: f32,
}

pub const EFFECTS_TUNING: EffectsTuning = EffectsTuning {
    max_effects: 64,
    max_equalizer_bands: 16,
    max_delay_ms: 5000.0,
    max_chorus_delay_ms: 50.0,
    filter_state_cutoff: 1.0e-20,
    reverb_reference_sample_rate: 44100,
    reverb_comb_delays: [1557, 1617, 1491, 1422],
    reverb_all_pass_delays: [225, 556],
    reverb_all_pass_feedback: 0.5,
};

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

    #[error("high-pass parameters are invalid")]
    InvalidHighPass,

    #[error("equalizer contains {actual} bands, but its capacity is {maximum}")]
    TooManyEqualizerBands { actual: usize, maximum: usize },

    #[error("equalizer frequency must be finite and greater than zero")]
    InvalidEqualizerFrequency,

    #[error("equalizer gain must be finite")]
    InvalidEqualizerGain,

    #[error("equalizer resonance must be finite and greater than zero")]
    InvalidEqualizerResonance,

    #[error("compressor parameters are invalid")]
    InvalidCompressor,

    #[error("limiter parameters are invalid")]
    InvalidLimiter,

    #[error("delay parameters are invalid")]
    InvalidDelay,

    #[error("reverb parameters are invalid")]
    InvalidReverb,

    #[error("chorus parameters are invalid")]
    InvalidChorus,

    #[error("distortion parameters are invalid")]
    InvalidDistortion,
}

#[derive(Clone, Copy)]
pub struct EffectsContext {
    pub channel_count: usize,
    pub sample_rate: u32,
}

pub fn prepare(mut effects: Effects, ctx: EffectsContext) -> Result<(Effects, RtEffects), Error> {
    let rt = RtEffects::new_replacing_failed(&mut effects, ctx.channel_count, ctx.sample_rate)?;
    Ok((effects, rt))
}

pub fn default(ctx: EffectsContext) -> Result<(Effects, RtEffects), Error> {
    let mut effects = Effects::default();
    let rt = RtEffects::new_replacing_failed(&mut effects, ctx.channel_count, ctx.sample_rate)?;
    Ok((effects, rt))
}
