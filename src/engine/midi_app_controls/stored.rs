use crate::{
    engine::graph::{ControllerId, InstrumentId},
    midi::MidiFilter,
};
use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const DEFAULT_CC_SWITCH_THRESHOLD: u8 = 64;

fn default_cc_switch_threshold() -> u8 {
    DEFAULT_CC_SWITCH_THRESHOLD
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum StoredMidiEngineControlKind {
    NoteTransportToggle {
        key: u8,
    },
    NoteTransportEnable {
        key: u8,
    },
    NoteTransportDisable {
        key: u8,
    },
    CCTransportSwitch {
        controller: u8,

        #[serde(default = "default_cc_switch_threshold")]
        #[schema(default = 64)]
        threshold: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoredMidiEngineControl {
    pub name: String,
    pub source_filter: MidiFilter,
    pub channel_filter: MidiFilter,
    pub kind: StoredMidiEngineControlKind,
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredMidiEngineControls {
    pub controls: Vec<StoredMidiEngineControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum StoredMidiProjectControlKind {
    NoteInstrumentControllerToggle {
        key: u8,
        instrument: InstrumentId,
        controller: ControllerId,
    },
    NoteInstrumentControllerEnable {
        key: u8,
        instrument: InstrumentId,
        controller: ControllerId,
    },
    NoteInstrumentControllerDisable {
        key: u8,
        instrument: InstrumentId,
        controller: ControllerId,
    },
    NoteUserPresetSelector {
        key: u8,
        preset_index: usize,
    },
    CCInstrumentControllerSwitch {
        controller: u8,

        #[serde(default = "default_cc_switch_threshold")]
        #[schema(default = 64)]
        threshold: u8,

        instrument: InstrumentId,
        instrument_controller: ControllerId,
    },
    CCUserPresetSelector {
        controller: u8,
        preset_index: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoredMidiProjectControl {
    pub name: String,
    pub source_filter: MidiFilter,
    pub channel_filter: MidiFilter,
    pub kind: StoredMidiProjectControlKind,
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredMidiProjectControls {
    pub controls: Vec<StoredMidiProjectControl>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_cc_switch_threshold_uses_ui_default() {
        let kind: StoredMidiEngineControlKind =
            serde_json::from_str(r#"{"CCTransportSwitch":{"controller":64}}"#)
                .expect("control should deserialize");

        assert!(matches!(
            kind,
            StoredMidiEngineControlKind::CCTransportSwitch {
                controller: 64,
                threshold: DEFAULT_CC_SWITCH_THRESHOLD,
            }
        ));
    }
}
