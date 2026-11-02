use crate::{
    engine::drum_machine::DrumMachineController,
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum CtrController {
    MidiInput(CtrMidiInputController),
    DrumMachine(DrumMachineController),
}

impl Default for CtrController {
    fn default() -> Self {
        Self::MidiInput(CtrMidiInputController::default())
    }
}

impl CtrController {
    pub fn midi_input(&self) -> Option<&CtrMidiInputController> {
        match self {
            Self::MidiInput(controller) => Some(controller),
            Self::DrumMachine(_) => None,
        }
    }

    pub fn midi_input_mut(&mut self) -> Option<&mut CtrMidiInputController> {
        match self {
            Self::MidiInput(controller) => Some(controller),
            Self::DrumMachine(_) => None,
        }
    }

    pub fn drum_machine(&self) -> Option<&DrumMachineController> {
        match self {
            Self::DrumMachine(controller) => Some(controller),
            Self::MidiInput(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CtrMidiInputController {
    pub enabled: bool,
    pub ignore_key_transform: bool,
    pub key_transform: MidiKeyTransform,
    pub ignore_velocity_transform: bool,
    pub velocity_transform: MidiVelocityTransform,
    pub channel_filter: MidiFilter,
    pub cc_filter: MidiFilter,
    pub ignore_pitch_bend: bool,
    pub ignore_program_change: bool,
    pub ignore_channel_aftertouch: bool,
    pub ignore_polyphonic_aftertouch: bool,
    pub use_global_transposition: bool,
    pub name: String,
}

impl Default for CtrMidiInputController {
    fn default() -> Self {
        Self {
            enabled: true,
            ignore_key_transform: false,
            key_transform: Default::default(),
            ignore_velocity_transform: false,
            velocity_transform: Default::default(),
            channel_filter: Default::default(),
            cc_filter: Default::default(),
            ignore_pitch_bend: false,
            ignore_program_change: false,
            ignore_channel_aftertouch: false,
            ignore_polyphonic_aftertouch: false,
            use_global_transposition: true,
            name: "MIDI Input".into(),
        }
    }
}
