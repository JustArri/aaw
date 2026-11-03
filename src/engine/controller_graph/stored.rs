use crate::{
    engine::{
        controller_graph::{CtrController, CtrMidiInputController},
        drum_machine::DrumMachineController,
    },
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
#[allow(clippy::large_enum_variant)]
pub enum StoredController {
    MidiInput(StoredMidiInputController),
    DrumMachine(DrumMachineController),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StoredControllerV1 {
    MidiInput(StoredMidiInputControllerV1),
    DrumMachine(DrumMachineController),
}

impl From<StoredControllerV1> for StoredController {
    fn from(value: StoredControllerV1) -> Self {
        match value {
            StoredControllerV1::MidiInput(controller) => {
                Self::MidiInput(StoredMidiInputControllerVersioned::V1(controller).migrate())
            }
            StoredControllerV1::DrumMachine(controller) => Self::DrumMachine(controller),
        }
    }
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

#[derive(Debug, Clone, Deserialize)]
pub struct StoredMidiInputControllerV1 {
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

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 2, version_type = u32)]
pub struct StoredMidiInputController {
    pub enabled: bool,
    pub source_filter: MidiFilter,
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

impl From<StoredMidiInputControllerV1> for StoredMidiInputControllerV2 {
    fn from(value: StoredMidiInputControllerV1) -> Self {
        Self {
            enabled: value.enabled,
            source_filter: MidiFilter::default(),
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

impl From<&CtrMidiInputController> for StoredMidiInputController {
    fn from(value: &CtrMidiInputController) -> Self {
        Self {
            enabled: value.enabled,
            source_filter: value.source_filter.clone(),
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
            source_filter: value.source_filter,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_v1_with_an_allow_all_source_filter() {
        let stored = StoredMidiInputController::from(CtrMidiInputController::default());
        let mut value = serde_json::to_value(stored).expect("serialize MIDI input controller");
        let object = value.as_object_mut().expect("controller should be an object");
        object.insert("version".into(), 1.into());
        object.remove("source_filter");

        let versioned: StoredMidiInputControllerVersioned =
            serde_json::from_value(value).expect("deserialize v1 MIDI input controller");
        assert_eq!(versioned.version(), Some(1));

        let stored = versioned.migrate();

        assert_eq!(stored.source_filter.as_u128(), u128::MAX);
    }

    #[test]
    fn serializes_current_controller_as_v2() {
        let stored = StoredMidiInputController::from(CtrMidiInputController::default());
        let value = serde_json::to_value(stored).expect("serialize MIDI input controller");

        assert_eq!(value["version"], 2);
        assert!(value.get("source_filter").is_some());
    }
}
