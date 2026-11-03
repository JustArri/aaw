use crate::{
    engine::graph::{ControllerAddress, ControllerId, InstrumentId},
    memory::Siv,
    midi::{
        CompiledMidiKeyTransform, CompiledMidiVelocityTransform, MidiEvent, MidiEventKind,
        MidiKeyTransform, MidiVelocityTransform,
    },
};

#[allow(clippy::large_enum_variant)]
pub enum RtController {
    MidiInput(RtMidiInputController),
    DrumMachine,
}

impl RtController {
    fn process(&self, event: MidiEvent, global_transposition: i8) -> Option<MidiEvent> {
        match self {
            Self::MidiInput(controller) => controller.process(event, global_transposition),
            Self::DrumMachine => None,
        }
    }
}

pub struct RtMidiInputController {
    pub enabled: bool,
    pub source_mask: u128,
    pub ignore_key_transform: bool,
    pub key_transform: CompiledMidiKeyTransform,
    pub ignore_velocity_transform: bool,
    pub velocity_transform: CompiledMidiVelocityTransform,
    pub channel_mask: u16,
    pub cc_mask: u128,
    pub ignore_pitch_bend: bool,
    pub ignore_program_change: bool,
    pub ignore_channel_aftertouch: bool,
    pub ignore_polyphonic_aftertouch: bool,
    pub use_global_transposition: bool,
}

impl Default for RtMidiInputController {
    fn default() -> Self {
        Self {
            enabled: true,
            source_mask: u128::MAX,
            ignore_key_transform: false,
            key_transform: MidiKeyTransform::default().compile(),
            ignore_velocity_transform: false,
            velocity_transform: MidiVelocityTransform::default().compile(),
            channel_mask: u16::MAX,
            cc_mask: u128::MAX,
            ignore_pitch_bend: false,
            ignore_program_change: false,
            ignore_channel_aftertouch: false,
            ignore_polyphonic_aftertouch: false,
            use_global_transposition: true,
        }
    }
}

impl RtMidiInputController {
    fn process(&self, event: MidiEvent, global_transposition: i8) -> Option<MidiEvent> {
        if event.slot >= u128::BITS as usize || self.source_mask & (1u128 << event.slot) == 0 {
            return None;
        }
        if self.channel_mask & (1u16 << event.channel) == 0 {
            return None;
        }
        if !self.enabled && matches!(event.kind, MidiEventKind::NoteOn { .. }) {
            return None;
        }
        if let MidiEventKind::ControlChange { controller, .. } = event.kind
            && self.cc_mask & (1u128 << controller) == 0
        {
            return None;
        }
        if let MidiEventKind::PitchBend { value } = event.kind
            && self.ignore_pitch_bend
            && value != 0
        {
            return None;
        }
        if matches!(event.kind, MidiEventKind::ProgramChange { .. }) && self.ignore_program_change {
            return None;
        }
        if matches!(event.kind, MidiEventKind::ChannelAftertouch { .. })
            && self.ignore_channel_aftertouch
        {
            return None;
        }
        if matches!(event.kind, MidiEventKind::PolyphonicAftertouch { .. })
            && self.ignore_polyphonic_aftertouch
        {
            return None;
        }

        event.transformed(
            (!self.ignore_key_transform).then_some(&self.key_transform),
            (!self.ignore_velocity_transform).then_some(&self.velocity_transform),
            if self.use_global_transposition { global_transposition } else { 0 },
        )
    }
}

pub struct RtInstrumentControllers {
    pub controllers: Siv<RtController, ControllerId>,
}

pub struct RtControllerGraph {
    pub instruments: Siv<RtInstrumentControllers, InstrumentId>,
    pub global_transposition: i8,
}

impl RtControllerGraph {
    pub fn process(&self, event: MidiEvent, mut output: impl FnMut(InstrumentId, MidiEvent)) {
        for (instrument_id, instrument) in &self.instruments {
            for controller in instrument.controllers.iter_values() {
                if let Some(event) = controller.process(event, self.global_transposition) {
                    output(*instrument_id, event);
                }
            }
        }
    }

    pub fn get_mut(&mut self, address: ControllerAddress) -> Option<&mut RtController> {
        self.instruments.get_mut(address.instrument_id)?.controllers.get_mut(address.controller_id)
    }
}
