use crate::{
    engine::{graph::LocalId, instrument_graph::SynthId}, memory::Siv, midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Slot {
    #[schema(required)]
    pub synth_id: Option<SynthId>,

    pub enabled: bool,
    pub key_transform: MidiKeyTransform,
    pub velocity_transform: MidiVelocityTransform,
    pub channel_filter: MidiFilter,
    pub cc_filter: MidiFilter,
    pub use_global_transposition: bool,
    pub name: String,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            synth_id: None,
            enabled: true,
            key_transform: Default::default(),
            velocity_transform: Default::default(),
            channel_filter: Default::default(),
            cc_filter: Default::default(),
            use_global_transposition: true,
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Graph {
    pub slots: Siv<Slot, LocalId>,
    pub global_transposition: i8,
}

impl Graph {
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

    pub fn append_new(&mut self) -> LocalId {
        self.slots.push(Slot::default())
    }

    pub fn remove(&mut self, slot_id: LocalId) {
        let _ = self.slots.try_remove(slot_id);
    }

    pub fn set_synth(&mut self, slot_id: LocalId, synth_id: Option<SynthId>) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.synth_id = synth_id;
        }
    }

    pub fn set_enabled(&mut self, slot_id: LocalId, flag: bool) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.enabled = flag;
        }
    }

    pub fn set_key_transform(&mut self, slot_id: LocalId, transform: MidiKeyTransform) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.key_transform = transform;
        }
    }

    pub fn set_velocity_transform(&mut self, slot_id: LocalId, transform: MidiVelocityTransform) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.velocity_transform = transform;
        }
    }

    pub fn set_channel_filter(&mut self, slot_id: LocalId, filter: MidiFilter) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.channel_filter = filter;
        }
    }

    pub fn set_cc_filter(&mut self, slot_id: LocalId, filter: MidiFilter) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.cc_filter = filter;
        }
    }

    pub fn set_use_global_transposition(&mut self, slot_id: LocalId, flag: bool) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.use_global_transposition = flag;
        }
    }

    pub fn set_name(&mut self, slot_id: LocalId, name: String) {
        if let Some(slot) = self.slots.get_mut(slot_id) {
            slot.name = name;
        }
    }
}
