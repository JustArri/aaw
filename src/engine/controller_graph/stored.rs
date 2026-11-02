use crate::{
    engine::{
        controller_graph::{CtrController, CtrMidiInputController},
        drum_machine::DrumMachineController,
    },
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum StoredController {
    MidiInput(StoredMidiInputController),
    DrumMachine(DrumMachineController),
}

impl From<&CtrController> for StoredController {
    fn from(value: &CtrController) -> Self {
        match value {
            CtrController::MidiInput(controller) => Self::MidiInput(controller.into()),
            CtrController::DrumMachine(controller) => Self::DrumMachine(controller.clone()),
        }
    }
}

impl From<CtrController> for StoredController {
    fn from(value: CtrController) -> Self {
        match value {
            CtrController::MidiInput(controller) => Self::MidiInput(controller.into()),
            CtrController::DrumMachine(controller) => Self::DrumMachine(controller),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoredMidiInputController {
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

impl From<&CtrMidiInputController> for StoredMidiInputController {
    fn from(value: &CtrMidiInputController) -> Self {
        Self {
            enabled: value.enabled,
            ignore_key_transform: value.ignore_key_transform,
            key_transform: value.key_transform.clone(),
            ignore_velocity_transform: value.ignore_velocity_transform,
            velocity_transform: value.velocity_transform.clone(),
            channel_filter: value.channel_filter.clone(),
            cc_filter: value.cc_filter.clone(),
            ignore_pitch_bend: value.ignore_pitch_bend,
            ignore_program_change: value.ignore_program_change,
            ignore_channel_aftertouch: value.ignore_channel_aftertouch,
            ignore_polyphonic_aftertouch: value.ignore_polyphonic_aftertouch,
            use_global_transposition: value.use_global_transposition,
            name: value.name.clone(),
        }
    }
}

impl From<CtrMidiInputController> for StoredMidiInputController {
    fn from(value: CtrMidiInputController) -> Self {
        Self {
            enabled: value.enabled,
            ignore_key_transform: value.ignore_key_transform,
            key_transform: value.key_transform,
            ignore_velocity_transform: value.ignore_velocity_transform,
            velocity_transform: value.velocity_transform,
            channel_filter: value.channel_filter,
            cc_filter: value.cc_filter,
            ignore_pitch_bend: value.ignore_pitch_bend,
            ignore_program_change: value.ignore_program_change,
            ignore_channel_aftertouch: value.ignore_channel_aftertouch,
            ignore_polyphonic_aftertouch: value.ignore_polyphonic_aftertouch,
            use_global_transposition: value.use_global_transposition,
            name: value.name,
        }
    }
}
