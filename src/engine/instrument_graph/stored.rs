use crate::{
    engine::{
        graph::{InstrumentId, SynthId},
        instrument_graph::{CtrInstrument, CtrInstrumentGraph},
    },
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use migrations_derive::Migrations;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredInstrument {
    #[schema(required)]
    pub synth_id: Option<SynthId>,

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

impl From<&CtrInstrument> for StoredInstrument {
    fn from(value: &CtrInstrument) -> Self {
        Self {
            synth_id: value.synth_id,
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

impl From<CtrInstrument> for StoredInstrument {
    fn from(value: CtrInstrument) -> Self {
        Self {
            synth_id: value.synth_id,
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

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredInstrumentGraph {
    pub instruments: HashMap<InstrumentId, StoredInstrument>,
    pub global_transposition: i8,
}

impl From<&CtrInstrumentGraph> for StoredInstrumentGraph {
    fn from(value: &CtrInstrumentGraph) -> Self {
        Self {
            instruments: value.instruments.map_ref(|s| s.into()).into(),
            global_transposition: value.global_transposition,
        }
    }
}

impl From<CtrInstrumentGraph> for StoredInstrumentGraph {
    fn from(value: CtrInstrumentGraph) -> Self {
        Self {
            instruments: value.instruments.map(|s| s.into()).into(),
            global_transposition: value.global_transposition,
        }
    }
}
