use crate::{
    engine::{
        graph::LocalId,
        instrument_graph::{CtrGraph, CtrSlot, SynthId},
    },
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use migrations_derive::Migrations;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = StoredInstrumentSlot)]
pub struct StoredSlot {
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

impl From<&CtrSlot> for StoredSlot {
    fn from(value: &CtrSlot) -> Self {
        Self {
            synth_id: value.synth_id,
            enabled: value.enabled,
            key_transform: value.key_transform.clone(),
            velocity_transform: value.velocity_transform.clone(),
            channel_filter: value.channel_filter.clone(),
            cc_filter: value.cc_filter.clone(),
            use_global_transposition: value.use_global_transposition,
            name: value.name.clone(),
        }
    }
}

impl From<CtrSlot> for StoredSlot {
    fn from(value: CtrSlot) -> Self {
        Self {
            synth_id: value.synth_id,
            enabled: value.enabled,
            key_transform: value.key_transform,
            velocity_transform: value.velocity_transform,
            channel_filter: value.channel_filter,
            cc_filter: value.cc_filter,
            use_global_transposition: value.use_global_transposition,
            name: value.name,
        }
    }
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = StoredInstrumentGraph)]
pub struct StoredGraph {
    pub slots: HashMap<LocalId, StoredSlot>,
    pub global_transposition: i8,
}

impl From<&CtrGraph> for StoredGraph {
    fn from(value: &CtrGraph) -> Self {
        Self {
            slots: value.slots.map_ref(|s| s.into()).into(),
            global_transposition: value.global_transposition,
        }
    }
}

impl From<CtrGraph> for StoredGraph {
    fn from(value: CtrGraph) -> Self {
        Self {
            slots: value.slots.map(|s| s.into()).into(),
            global_transposition: value.global_transposition,
        }
    }
}
