use crate::{
    engine::{graph::LocalId, instrument_graph::SynthId},
    memory::Siv,
    midi::{
        CompiledMidiKeyTransform, CompiledMidiVelocityTransform, MidiEvent, MidiEventKind,
        MidiKeyTransform, MidiVelocityTransform,
    },
};

pub struct Slot {
    pub synth_id: Option<SynthId>,
    pub enabled: bool,
    pub key_transform: CompiledMidiKeyTransform,
    pub velocity_transform: CompiledMidiVelocityTransform,
    pub channel_mask: u16,
    pub cc_mask: u128,
    pub use_global_transposition: bool,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            synth_id: None,
            enabled: true,
            key_transform: MidiKeyTransform::default().compile(),
            velocity_transform: MidiVelocityTransform::default().compile(),
            channel_mask: u16::MAX,
            cc_mask: u128::MAX,
            use_global_transposition: true,
        }
    }
}

pub struct MidiHandlerIter<'a> {
    slots: std::slice::Iter<'a, Slot>,
    global_transposition: i8,
    event: MidiEvent,
}

impl<'a> MidiHandlerIter<'a> {
    fn new(graph: &'a RtGraph, event: MidiEvent) -> Self {
        Self {
            slots: graph.slots.iter_values(),
            global_transposition: graph.global_transposition,
            event,
        }
    }
}

impl<'a> Iterator for MidiHandlerIter<'a> {
    type Item = (SynthId, MidiEvent);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slot = self.slots.next()?;

            let Some(synth_id) = slot.synth_id else {
                continue;
            };

            if slot.channel_mask & (1u16 << self.event.channel) == 0 {
                continue;
            }

            if !slot.enabled && matches!(self.event.kind, MidiEventKind::NoteOn { .. }) {
                continue;
            }

            if let MidiEventKind::ControlChange { controller, .. } = &self.event.kind
                && slot.cc_mask & (1u128 << controller) == 0
            {
                continue;
            }

            let transposition =
                if slot.use_global_transposition { self.global_transposition } else { 0 };

            let Some(event) = self.event.transformed(
                &slot.key_transform,
                &slot.velocity_transform,
                transposition,
            ) else {
                continue;
            };

            return Some((synth_id, event));
        }
    }
}

pub struct AudioRendererIter<'a> {
    slots: std::slice::Iter<'a, Slot>,
}

impl<'a> AudioRendererIter<'a> {
    fn new(graph: &'a mut RtGraph) -> Self {
        Self { slots: graph.slots.iter_values() }
    }
}

impl<'a> Iterator for AudioRendererIter<'a> {
    type Item = SynthId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slot = self.slots.next()?;
            let Some(synth_id) = slot.synth_id else {
                continue;
            };

            // TODO: this doesn't take release into account, maybe fix it later?
            // if !slot.enabled {
            //     continue;
            // }

            return Some(synth_id);
        }
    }
}

pub struct RtGraph {
    pub slots: Siv<Slot, LocalId>,
    pub global_transposition: i8,
}

impl RtGraph {
    pub fn midi_handler_iter(&'_ self, event: MidiEvent) -> MidiHandlerIter<'_> {
        MidiHandlerIter::new(self, event)
    }

    pub fn audio_renderer_iter(&'_ mut self) -> AudioRendererIter<'_> {
        AudioRendererIter::new(self)
    }

    pub fn remove_synth_references(&mut self, synth_id: SynthId) {
        for slot in self.slots.iter_values_mut() {
            if slot.synth_id == Some(synth_id) {
                slot.synth_id = None;
            }
        }
    }

    pub fn set_global_transposition(&mut self, transposition: i8) {
        self.global_transposition = transposition;
    }

    pub fn append_new(&mut self) -> bool {
        if self.slots.len() < self.slots.capacity() {
            self.slots.push(Slot::default());
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, slot_id: LocalId) -> bool {
        self.slots.try_remove(slot_id).is_some()
    }

    pub fn set_synth(&mut self, slot_id: LocalId, synth_id: Option<SynthId>) -> bool {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.synth_id = synth_id;
            true
        } else {
            false
        }
    }

    pub fn set_enabled(&mut self, slot_id: LocalId, flag: bool) -> bool {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.enabled = flag;
            true
        } else {
            false
        }
    }

    pub fn set_key_transform(
        &mut self,
        slot_id: LocalId,
        transform: CompiledMidiKeyTransform,
    ) -> bool {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.key_transform = transform;
            true
        } else {
            false
        }
    }

    pub fn set_velocity_transform(
        &mut self,
        slot_id: LocalId,
        transform: CompiledMidiVelocityTransform,
    ) -> bool {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.velocity_transform = transform;
            true
        } else {
            false
        }
    }

    pub fn set_channel_filter(&mut self, slot_id: LocalId, mask: u16) -> bool {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.channel_mask = mask;
            true
        } else {
            false
        }
    }

    pub fn set_cc_filter(&mut self, slot_id: LocalId, mask: u128) -> bool {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.cc_mask = mask;
            true
        } else {
            false
        }
    }

    pub fn set_use_global_transposition(&mut self, slot_id: LocalId, flag: bool) -> bool {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.use_global_transposition = flag;
            true
        } else {
            false
        }
    }
}
