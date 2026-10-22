use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct Effects {
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Effect {
    #[serde(default = "effect_enabled_by_default")]
    #[schema(required)]
    pub enabled: bool,

    #[serde(flatten)]
    pub kind: EffectKind,
}

const fn effect_enabled_by_default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum EffectKind {
    Gain {
        value: f32,
    },
    LowPassFilter {
        frequency: f32,
        resonance: f32,
    },
    HighPassFilter {
        frequency: f32,
        resonance: f32,
    },
    Equalizer {
        bands: Vec<EqualizerBand>,
    },
    Compressor {
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_gain_db: f32,
    },
    Limiter {
        threshold_db: f32,
        release_ms: f32,
    },
    Delay {
        time_ms: f32,
        feedback: f32,
        mix: f32,
    },
    Reverb {
        room_size: f32,
        damping: f32,
        mix: f32,
    },
    Chorus {
        rate_hz: f32,
        depth_ms: f32,
        delay_ms: f32,
        mix: f32,
    },
    Distortion {
        drive_db: f32,
        mix: f32,
    },
    LoadFailed {
        description: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum EqualizerBand {
    LowShelf { frequency: f32, gain_db: f32, resonance: f32 },
    Peaking { frequency: f32, gain_db: f32, resonance: f32 },
    HighShelf { frequency: f32, gain_db: f32, resonance: f32 },
}
