use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum MidiVelocityTransform {
    #[default]
    Unit,

    Linear {
        min: u8,
        max: u8,
    },
    //TODO: add curves
}

impl MidiVelocityTransform {
    pub fn apply(&self, velocity: u8) -> u8 {
        match self {
            Self::Unit => velocity,
            Self::Linear { min, max } => {
                let value = velocity.min(127) as u16;
                let range = (max - min) as u16;
                min + ((value * range + 63) / 127) as u8
            }
        }
    }

    pub fn compile(&self) -> CompiledMidiVelocityTransform {
        CompiledMidiVelocityTransform::new(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompiledMidiVelocityTransform {
    Unit,

    Linear { min: u8, max: u8 },
}

impl CompiledMidiVelocityTransform {
    pub fn new(value: &MidiVelocityTransform) -> Self {
        match value {
            MidiVelocityTransform::Unit => Self::Unit,
            MidiVelocityTransform::Linear { min, max } => Self::Linear { min: *min, max: *max },
        }
    }

    pub fn apply(&self, velocity: u8) -> u8 {
        match self {
            Self::Unit => velocity,
            Self::Linear { min, max } => {
                let value = velocity.min(127) as u16;
                let range = (max - min) as u16;
                min + ((value * range + 63) / 127) as u8
            }
        }
    }
}

impl From<&MidiVelocityTransform> for CompiledMidiVelocityTransform {
    fn from(value: &MidiVelocityTransform) -> Self {
        Self::new(value)
    }
}
