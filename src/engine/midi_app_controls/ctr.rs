use crate::{
    engine::{
        graph::{ControllerId, InstrumentId},
        midi_app_controls::{
            StoredMidiEngineControl, StoredMidiEngineControlKind, StoredMidiEngineControls,
            StoredMidiProjectControl, StoredMidiProjectControlKind, StoredMidiProjectControls,
        },
    },
    midi::MidiFilter,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CtrMidiEngineControlKind {
    NoteTransportToggle { key: u8 },
    NoteTransportEnable { key: u8 },
    NoteTransportDisable { key: u8 },
    CCTransportSwitch { controller: u8, threshold: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CtrMidiEngineControl {
    pub name: String,
    pub source_filter: MidiFilter,
    pub channel_filter: MidiFilter,
    pub kind: CtrMidiEngineControlKind,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CtrMidiEngineControls {
    pub controls: Vec<CtrMidiEngineControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CtrMidiProjectControlKind {
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
pub struct CtrMidiProjectControl {
    pub name: String,
    pub source_filter: MidiFilter,
    pub channel_filter: MidiFilter,
    pub kind: CtrMidiProjectControlKind,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CtrMidiProjectControls {
    pub controls: Vec<CtrMidiProjectControl>,
}

impl From<StoredMidiEngineControlKind> for CtrMidiEngineControlKind {
    fn from(value: StoredMidiEngineControlKind) -> Self {
        match value {
            StoredMidiEngineControlKind::NoteTransportToggle { key } => {
                Self::NoteTransportToggle { key }
            }
            StoredMidiEngineControlKind::NoteTransportEnable { key } => {
                Self::NoteTransportEnable { key }
            }
            StoredMidiEngineControlKind::NoteTransportDisable { key } => {
                Self::NoteTransportDisable { key }
            }
            StoredMidiEngineControlKind::CCTransportSwitch { controller, threshold } => {
                Self::CCTransportSwitch { controller, threshold }
            }
        }
    }
}

impl From<CtrMidiEngineControlKind> for StoredMidiEngineControlKind {
    fn from(value: CtrMidiEngineControlKind) -> Self {
        match value {
            CtrMidiEngineControlKind::NoteTransportToggle { key } => {
                Self::NoteTransportToggle { key }
            }
            CtrMidiEngineControlKind::NoteTransportEnable { key } => {
                Self::NoteTransportEnable { key }
            }
            CtrMidiEngineControlKind::NoteTransportDisable { key } => {
                Self::NoteTransportDisable { key }
            }
            CtrMidiEngineControlKind::CCTransportSwitch { controller, threshold } => {
                Self::CCTransportSwitch { controller, threshold }
            }
        }
    }
}

impl From<StoredMidiEngineControl> for CtrMidiEngineControl {
    fn from(value: StoredMidiEngineControl) -> Self {
        Self {
            name: value.name,
            source_filter: value.source_filter,
            channel_filter: value.channel_filter,
            kind: value.kind.into(),
        }
    }
}

impl From<CtrMidiEngineControl> for StoredMidiEngineControl {
    fn from(value: CtrMidiEngineControl) -> Self {
        Self {
            name: value.name,
            source_filter: value.source_filter,
            channel_filter: value.channel_filter,
            kind: value.kind.into(),
        }
    }
}

impl From<StoredMidiEngineControls> for CtrMidiEngineControls {
    fn from(value: StoredMidiEngineControls) -> Self {
        Self { controls: value.controls.into_iter().map(Into::into).collect() }
    }
}

impl From<CtrMidiEngineControls> for StoredMidiEngineControls {
    fn from(value: CtrMidiEngineControls) -> Self {
        Self { controls: value.controls.into_iter().map(Into::into).collect() }
    }
}

impl From<StoredMidiProjectControlKind> for CtrMidiProjectControlKind {
    fn from(value: StoredMidiProjectControlKind) -> Self {
        match value {
            StoredMidiProjectControlKind::NoteInstrumentControllerToggle {
                key,
                instrument,
                controller,
            } => Self::NoteInstrumentControllerToggle { key, instrument, controller },
            StoredMidiProjectControlKind::NoteInstrumentControllerEnable {
                key,
                instrument,
                controller,
            } => Self::NoteInstrumentControllerEnable { key, instrument, controller },
            StoredMidiProjectControlKind::NoteInstrumentControllerDisable {
                key,
                instrument,
                controller,
            } => Self::NoteInstrumentControllerDisable { key, instrument, controller },
            StoredMidiProjectControlKind::NoteUserPresetSelector { key, preset_index } => {
                Self::NoteUserPresetSelector { key, preset_index }
            }
            StoredMidiProjectControlKind::CCInstrumentControllerSwitch {
                controller,
                threshold,
                instrument,
                instrument_controller,
            } => Self::CCInstrumentControllerSwitch {
                controller,
                threshold,
                instrument,
                instrument_controller,
            },
            StoredMidiProjectControlKind::CCUserPresetSelector { controller, preset_index } => {
                Self::CCUserPresetSelector { controller, preset_index }
            }
        }
    }
}

impl From<CtrMidiProjectControlKind> for StoredMidiProjectControlKind {
    fn from(value: CtrMidiProjectControlKind) -> Self {
        match value {
            CtrMidiProjectControlKind::NoteInstrumentControllerToggle {
                key,
                instrument,
                controller,
            } => Self::NoteInstrumentControllerToggle { key, instrument, controller },
            CtrMidiProjectControlKind::NoteInstrumentControllerEnable {
                key,
                instrument,
                controller,
            } => Self::NoteInstrumentControllerEnable { key, instrument, controller },
            CtrMidiProjectControlKind::NoteInstrumentControllerDisable {
                key,
                instrument,
                controller,
            } => Self::NoteInstrumentControllerDisable { key, instrument, controller },
            CtrMidiProjectControlKind::NoteUserPresetSelector { key, preset_index } => {
                Self::NoteUserPresetSelector { key, preset_index }
            }
            CtrMidiProjectControlKind::CCInstrumentControllerSwitch {
                controller,
                threshold,
                instrument,
                instrument_controller,
            } => Self::CCInstrumentControllerSwitch {
                controller,
                threshold,
                instrument,
                instrument_controller,
            },
            CtrMidiProjectControlKind::CCUserPresetSelector { controller, preset_index } => {
                Self::CCUserPresetSelector { controller, preset_index }
            }
        }
    }
}

impl From<StoredMidiProjectControl> for CtrMidiProjectControl {
    fn from(value: StoredMidiProjectControl) -> Self {
        Self {
            name: value.name,
            source_filter: value.source_filter,
            channel_filter: value.channel_filter,
            kind: value.kind.into(),
        }
    }
}

impl From<CtrMidiProjectControl> for StoredMidiProjectControl {
    fn from(value: CtrMidiProjectControl) -> Self {
        Self {
            name: value.name,
            source_filter: value.source_filter,
            channel_filter: value.channel_filter,
            kind: value.kind.into(),
        }
    }
}

impl From<StoredMidiProjectControls> for CtrMidiProjectControls {
    fn from(value: StoredMidiProjectControls) -> Self {
        Self { controls: value.controls.into_iter().map(Into::into).collect() }
    }
}

impl From<CtrMidiProjectControls> for StoredMidiProjectControls {
    fn from(value: CtrMidiProjectControls) -> Self {
        Self { controls: value.controls.into_iter().map(Into::into).collect() }
    }
}
