use crate::{
    engine::graph::{InstrumentId, SynthId},
    memory::Siv,
    midi::{
        CompiledMidiKeyTransform, CompiledMidiVelocityTransform, MidiEvent, MidiEventKind,
        MidiKeyTransform, MidiVelocityTransform,
    },
};

pub struct RtInstrument {
    pub synth_id: Option<SynthId>,
    pub enabled: bool,
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

impl Default for RtInstrument {
    fn default() -> Self {
        Self {
            synth_id: None,
            enabled: true,
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

pub struct MidiHandlerIter<'a> {
    instruments: std::slice::Iter<'a, RtInstrument>,
    global_transposition: i8,
    event: MidiEvent,
}

impl<'a> MidiHandlerIter<'a> {
    fn new(graph: &'a RtInstrumentGraph, event: MidiEvent) -> Self {
        Self {
            instruments: graph.instruments.iter_values(),
            global_transposition: graph.global_transposition,
            event,
        }
    }
}

impl<'a> Iterator for MidiHandlerIter<'a> {
    type Item = (SynthId, MidiEvent);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let instrument = self.instruments.next()?;

            let Some(synth_id) = instrument.synth_id else {
                continue;
            };

            if instrument.channel_mask & (1u16 << self.event.channel) == 0 {
                continue;
            }

            if !instrument.enabled && matches!(self.event.kind, MidiEventKind::NoteOn { .. }) {
                continue;
            }

            if let MidiEventKind::ControlChange { controller, .. } = &self.event.kind
                && instrument.cc_mask & (1u128 << controller) == 0
            {
                continue;
            }

            if let MidiEventKind::PitchBend { value } = &self.event.kind
                && instrument.ignore_pitch_bend
                && *value != 0
            {
                continue;
            }

            if matches!(self.event.kind, MidiEventKind::ProgramChange { .. })
                && instrument.ignore_program_change
            {
                continue;
            }

            if matches!(self.event.kind, MidiEventKind::ChannelAftertouch { .. })
                && instrument.ignore_channel_aftertouch
            {
                continue;
            }

            if matches!(self.event.kind, MidiEventKind::PolyphonicAftertouch { .. })
                && instrument.ignore_polyphonic_aftertouch
            {
                continue;
            }

            let transposition =
                if instrument.use_global_transposition { self.global_transposition } else { 0 };

            let Some(event) = self.event.transformed(
                (!instrument.ignore_key_transform).then_some(&instrument.key_transform),
                (!instrument.ignore_velocity_transform).then_some(&instrument.velocity_transform),
                transposition,
            ) else {
                continue;
            };

            return Some((synth_id, event));
        }
    }
}

pub struct RtInstrumentGraph {
    pub instruments: Siv<RtInstrument, InstrumentId>,
    pub global_transposition: i8,
}

impl RtInstrumentGraph {
    pub fn midi_handler_iter(&'_ self, event: MidiEvent) -> MidiHandlerIter<'_> {
        MidiHandlerIter::new(self, event)
    }

    pub fn remove_synth_references(&mut self, synth_id: SynthId) {
        for instrument in self.instruments.iter_values_mut() {
            if instrument.synth_id == Some(synth_id) {
                instrument.synth_id = None;
            }
        }
    }

    pub fn set_global_transposition(&mut self, transposition: i8) {
        self.global_transposition = transposition;
    }

    pub fn append_new(&mut self) -> bool {
        if self.instruments.len() < self.instruments.capacity() {
            self.instruments.push(RtInstrument::default());
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, id: InstrumentId) -> bool {
        self.instruments.try_remove(id).is_some()
    }

    pub fn set_synth(&mut self, id: InstrumentId, synth_id: Option<SynthId>) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.synth_id = synth_id;
            true
        } else {
            false
        }
    }

    pub fn set_enabled(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.enabled = flag;
            true
        } else {
            false
        }
    }

    pub fn set_ignore_key_transform(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.ignore_key_transform = flag;
            true
        } else {
            false
        }
    }

    pub fn set_key_transform(
        &mut self,
        id: InstrumentId,
        transform: CompiledMidiKeyTransform,
    ) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.key_transform = transform;
            true
        } else {
            false
        }
    }

    pub fn set_ignore_velocity_transform(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.ignore_velocity_transform = flag;
            true
        } else {
            false
        }
    }

    pub fn set_velocity_transform(
        &mut self,
        id: InstrumentId,
        transform: CompiledMidiVelocityTransform,
    ) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.velocity_transform = transform;
            true
        } else {
            false
        }
    }

    pub fn set_channel_filter(&mut self, id: InstrumentId, mask: u16) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.channel_mask = mask;
            true
        } else {
            false
        }
    }

    pub fn set_cc_filter(&mut self, id: InstrumentId, mask: u128) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.cc_mask = mask;
            true
        } else {
            false
        }
    }

    pub fn set_ignore_pitch_bend(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.ignore_pitch_bend = flag;
            true
        } else {
            false
        }
    }

    pub fn set_ignore_program_change(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.ignore_program_change = flag;
            true
        } else {
            false
        }
    }

    pub fn set_ignore_channel_aftertouch(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.ignore_channel_aftertouch = flag;
            true
        } else {
            false
        }
    }

    pub fn set_ignore_polyphonic_aftertouch(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.ignore_polyphonic_aftertouch = flag;
            true
        } else {
            false
        }
    }

    pub fn set_use_global_transposition(&mut self, id: InstrumentId, flag: bool) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.use_global_transposition = flag;
            true
        } else {
            false
        }
    }
}
