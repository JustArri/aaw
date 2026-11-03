use crate::{
    engine::{
        graph::ControllerAddress,
        midi_app_controls::{
            CtrMidiEngineControlKind, CtrMidiEngineControls, CtrMidiProjectControlKind,
            CtrMidiProjectControls,
        },
    },
    midi::{MidiEvent, MidiEventKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiAppAction {
    ToggleTransport,
    SetTransportPlaying(bool),
    ToggleInstrumentController(ControllerAddress),
    SetInstrumentControllerEnabled(ControllerAddress, bool),
    ActivateUserPreset(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum RtMidiEngineControlKind {
    NoteTransportToggle { key: u8 },
    NoteTransportEnable { key: u8 },
    NoteTransportDisable { key: u8 },
    CCTransportSwitch { controller: u8, threshold: u8 },
}

#[derive(Debug, Clone, Copy)]
pub enum RtMidiProjectControlKind {
    NoteInstrumentControllerToggle { key: u8, address: ControllerAddress },
    NoteInstrumentControllerEnable { key: u8, address: ControllerAddress },
    NoteInstrumentControllerDisable { key: u8, address: ControllerAddress },
    NoteUserPresetSelector { key: u8, preset_index: usize },
    CCInstrumentControllerSwitch { controller: u8, threshold: u8, address: ControllerAddress },
    CCUserPresetSelector { controller: u8, preset_index: usize },
}

#[derive(Debug, Clone, Copy)]
struct RtMidiAppControl<K> {
    source_mask: u128,
    channel_mask: u16,
    kind: K,
}

impl<K> RtMidiAppControl<K> {
    fn accepts(&self, event: MidiEvent) -> bool {
        event.slot < u128::BITS as usize
            && self.source_mask & (1u128 << event.slot) != 0
            && self.channel_mask & (1u16 << event.channel) != 0
    }
}

#[derive(Default)]
pub struct RtMidiEngineControls {
    controls: Vec<RtMidiAppControl<RtMidiEngineControlKind>>,
}

#[derive(Default)]
pub struct RtMidiProjectControls {
    controls: Vec<RtMidiAppControl<RtMidiProjectControlKind>>,
}

impl RtMidiEngineControls {
    pub fn process(&self, event: MidiEvent, mut output: impl FnMut(MidiAppAction)) {
        for control in &self.controls {
            if !control.accepts(event) {
                continue;
            }
            let action = match (control.kind, event.kind) {
                (
                    RtMidiEngineControlKind::NoteTransportToggle { key: expected },
                    MidiEventKind::NoteOn { key, .. },
                ) if key == expected => Some(MidiAppAction::ToggleTransport),
                (
                    RtMidiEngineControlKind::NoteTransportEnable { key: expected },
                    MidiEventKind::NoteOn { key, .. },
                ) if key == expected => Some(MidiAppAction::SetTransportPlaying(true)),
                (
                    RtMidiEngineControlKind::NoteTransportDisable { key: expected },
                    MidiEventKind::NoteOn { key, .. },
                ) if key == expected => Some(MidiAppAction::SetTransportPlaying(false)),
                (
                    RtMidiEngineControlKind::CCTransportSwitch { controller: expected, threshold },
                    MidiEventKind::ControlChange { controller, value },
                ) if controller == expected => {
                    Some(MidiAppAction::SetTransportPlaying(value >= threshold))
                }
                _ => None,
            };
            if let Some(action) = action {
                output(action);
            }
        }
    }
}

impl RtMidiProjectControls {
    pub fn process(&self, event: MidiEvent, mut output: impl FnMut(MidiAppAction)) {
        for control in &self.controls {
            if !control.accepts(event) {
                continue;
            }
            let action = match (control.kind, event.kind) {
                (
                    RtMidiProjectControlKind::NoteInstrumentControllerToggle {
                        key: expected,
                        address,
                    },
                    MidiEventKind::NoteOn { key, .. },
                ) if key == expected => Some(MidiAppAction::ToggleInstrumentController(address)),
                (
                    RtMidiProjectControlKind::NoteInstrumentControllerEnable {
                        key: expected,
                        address,
                    },
                    MidiEventKind::NoteOn { key, .. },
                ) if key == expected => {
                    Some(MidiAppAction::SetInstrumentControllerEnabled(address, true))
                }
                (
                    RtMidiProjectControlKind::NoteInstrumentControllerDisable {
                        key: expected,
                        address,
                    },
                    MidiEventKind::NoteOn { key, .. },
                ) if key == expected => {
                    Some(MidiAppAction::SetInstrumentControllerEnabled(address, false))
                }
                (
                    RtMidiProjectControlKind::NoteUserPresetSelector {
                        key: expected,
                        preset_index,
                    },
                    MidiEventKind::NoteOn { key, .. },
                ) if key == expected => Some(MidiAppAction::ActivateUserPreset(preset_index)),
                (
                    RtMidiProjectControlKind::CCInstrumentControllerSwitch {
                        controller: expected,
                        threshold,
                        address,
                    },
                    MidiEventKind::ControlChange { controller, value },
                ) if controller == expected => {
                    Some(MidiAppAction::SetInstrumentControllerEnabled(address, value >= threshold))
                }
                (
                    RtMidiProjectControlKind::CCUserPresetSelector {
                        controller: expected,
                        preset_index,
                    },
                    MidiEventKind::ControlChange { controller, value },
                ) if controller == expected && value > 0 => {
                    Some(MidiAppAction::ActivateUserPreset(preset_index))
                }
                _ => None,
            };
            if let Some(action) = action {
                output(action);
            }
        }
    }
}

impl From<&CtrMidiEngineControls> for RtMidiEngineControls {
    fn from(value: &CtrMidiEngineControls) -> Self {
        Self {
            controls: value
                .controls
                .iter()
                .map(|control| RtMidiAppControl {
                    source_mask: control.source_filter.as_u128(),
                    channel_mask: control.channel_filter.as_u16(),
                    kind: match control.kind {
                        CtrMidiEngineControlKind::NoteTransportToggle { key } => {
                            RtMidiEngineControlKind::NoteTransportToggle { key }
                        }
                        CtrMidiEngineControlKind::NoteTransportEnable { key } => {
                            RtMidiEngineControlKind::NoteTransportEnable { key }
                        }
                        CtrMidiEngineControlKind::NoteTransportDisable { key } => {
                            RtMidiEngineControlKind::NoteTransportDisable { key }
                        }
                        CtrMidiEngineControlKind::CCTransportSwitch { controller, threshold } => {
                            RtMidiEngineControlKind::CCTransportSwitch { controller, threshold }
                        }
                    },
                })
                .collect(),
        }
    }
}

impl From<&CtrMidiProjectControls> for RtMidiProjectControls {
    fn from(value: &CtrMidiProjectControls) -> Self {
        Self {
            controls: value
                .controls
                .iter()
                .map(|control| RtMidiAppControl {
                    source_mask: control.source_filter.as_u128(),
                    channel_mask: control.channel_filter.as_u16(),
                    kind: match control.kind {
                        CtrMidiProjectControlKind::NoteInstrumentControllerToggle {
                            key,
                            instrument,
                            controller,
                        } => RtMidiProjectControlKind::NoteInstrumentControllerToggle {
                            key,
                            address: ControllerAddress {
                                instrument_id: instrument,
                                controller_id: controller,
                            },
                        },
                        CtrMidiProjectControlKind::NoteInstrumentControllerEnable {
                            key,
                            instrument,
                            controller,
                        } => RtMidiProjectControlKind::NoteInstrumentControllerEnable {
                            key,
                            address: ControllerAddress {
                                instrument_id: instrument,
                                controller_id: controller,
                            },
                        },
                        CtrMidiProjectControlKind::NoteInstrumentControllerDisable {
                            key,
                            instrument,
                            controller,
                        } => RtMidiProjectControlKind::NoteInstrumentControllerDisable {
                            key,
                            address: ControllerAddress {
                                instrument_id: instrument,
                                controller_id: controller,
                            },
                        },
                        CtrMidiProjectControlKind::NoteUserPresetSelector { key, preset_index } => {
                            RtMidiProjectControlKind::NoteUserPresetSelector { key, preset_index }
                        }
                        CtrMidiProjectControlKind::CCInstrumentControllerSwitch {
                            controller,
                            threshold,
                            instrument,
                            instrument_controller,
                        } => RtMidiProjectControlKind::CCInstrumentControllerSwitch {
                            controller,
                            threshold,
                            address: ControllerAddress {
                                instrument_id: instrument,
                                controller_id: instrument_controller,
                            },
                        },
                        CtrMidiProjectControlKind::CCUserPresetSelector {
                            controller,
                            preset_index,
                        } => RtMidiProjectControlKind::CCUserPresetSelector {
                            controller,
                            preset_index,
                        },
                    },
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{
            graph::{ControllerId, InstrumentId},
            midi_app_controls::{
                CtrMidiEngineControl, CtrMidiEngineControlKind, CtrMidiProjectControl,
                CtrMidiProjectControlKind,
            },
        },
        midi::{MidiFilter, MidiFilterKind},
    };
    use std::collections::HashSet;

    fn allow(values: impl IntoIterator<Item = u8>) -> MidiFilter {
        MidiFilter { kind: MidiFilterKind::Allowlist, list: HashSet::from_iter(values) }
    }

    #[test]
    fn engine_control_filters_source_and_channel() {
        let controls = CtrMidiEngineControls {
            controls: vec![CtrMidiEngineControl {
                name: "Pedal transport".into(),
                source_filter: allow([2]),
                channel_filter: allow([3]),
                kind: CtrMidiEngineControlKind::CCTransportSwitch { controller: 64, threshold: 64 },
            }],
        };
        let rt = RtMidiEngineControls::from(&controls);
        let mut actions = Vec::new();

        rt.process(
            MidiEvent {
                slot: 1,
                channel: 3,
                kind: MidiEventKind::ControlChange { controller: 64, value: 127 },
            },
            |action| actions.push(action),
        );
        rt.process(
            MidiEvent {
                slot: 2,
                channel: 4,
                kind: MidiEventKind::ControlChange { controller: 64, value: 127 },
            },
            |action| actions.push(action),
        );
        assert!(actions.is_empty());

        rt.process(
            MidiEvent {
                slot: 2,
                channel: 3,
                kind: MidiEventKind::ControlChange { controller: 64, value: 127 },
            },
            |action| actions.push(action),
        );
        assert_eq!(actions, [MidiAppAction::SetTransportPlaying(true)]);
    }

    #[test]
    fn cc_switch_changes_at_threshold() {
        let controls = CtrMidiEngineControls {
            controls: vec![CtrMidiEngineControl {
                name: String::new(),
                source_filter: MidiFilter::default(),
                channel_filter: MidiFilter::default(),
                kind: CtrMidiEngineControlKind::CCTransportSwitch { controller: 64, threshold: 64 },
            }],
        };
        let rt = RtMidiEngineControls::from(&controls);
        let mut actions = Vec::new();
        rt.process(
            MidiEvent {
                slot: 0,
                channel: 0,
                kind: MidiEventKind::ControlChange { controller: 64, value: 64 },
            },
            |action| actions.push(action),
        );
        assert_eq!(actions, [MidiAppAction::SetTransportPlaying(false)]);

        actions.clear();
        rt.process(
            MidiEvent {
                slot: 0,
                channel: 0,
                kind: MidiEventKind::ControlChange { controller: 64, value: 64 },
            },
            |action| actions.push(action),
        );
        assert_eq!(actions, [MidiAppAction::SetTransportPlaying(true)]);
    }

    #[test]
    fn note_controls_set_transport_state() {
        let controls = CtrMidiEngineControls {
            controls: vec![
                CtrMidiEngineControl {
                    name: String::new(),
                    source_filter: MidiFilter::default(),
                    channel_filter: MidiFilter::default(),
                    kind: CtrMidiEngineControlKind::NoteTransportEnable { key: 60 },
                },
                CtrMidiEngineControl {
                    name: String::new(),
                    source_filter: MidiFilter::default(),
                    channel_filter: MidiFilter::default(),
                    kind: CtrMidiEngineControlKind::NoteTransportDisable { key: 61 },
                },
            ],
        };
        let rt = RtMidiEngineControls::from(&controls);
        let mut actions = Vec::new();

        for key in [60, 61] {
            rt.process(
                MidiEvent {
                    slot: 0,
                    channel: 0,
                    kind: MidiEventKind::NoteOn { key, velocity: 127 },
                },
                |action| actions.push(action),
            );
        }

        assert_eq!(
            actions,
            [MidiAppAction::SetTransportPlaying(true), MidiAppAction::SetTransportPlaying(false),]
        );
    }

    #[test]
    fn note_controls_set_instrument_controller_state() {
        let instrument = InstrumentId(2);
        let controller = ControllerId(3);
        let address = ControllerAddress { instrument_id: instrument, controller_id: controller };
        let controls = CtrMidiProjectControls {
            controls: vec![
                CtrMidiProjectControl {
                    name: String::new(),
                    source_filter: MidiFilter::default(),
                    channel_filter: MidiFilter::default(),
                    kind: CtrMidiProjectControlKind::NoteInstrumentControllerEnable {
                        key: 62,
                        instrument,
                        controller,
                    },
                },
                CtrMidiProjectControl {
                    name: String::new(),
                    source_filter: MidiFilter::default(),
                    channel_filter: MidiFilter::default(),
                    kind: CtrMidiProjectControlKind::NoteInstrumentControllerDisable {
                        key: 63,
                        instrument,
                        controller,
                    },
                },
            ],
        };
        let rt = RtMidiProjectControls::from(&controls);
        let mut actions = Vec::new();

        for key in [62, 63] {
            rt.process(
                MidiEvent {
                    slot: 0,
                    channel: 0,
                    kind: MidiEventKind::NoteOn { key, velocity: 127 },
                },
                |action| actions.push(action),
            );
        }

        assert_eq!(
            actions,
            [
                MidiAppAction::SetInstrumentControllerEnabled(address, true),
                MidiAppAction::SetInstrumentControllerEnabled(address, false),
            ]
        );
    }
}
