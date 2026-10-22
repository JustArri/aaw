use crate::{
    engine::graph::{InstrumentId, SynthId},
    memory::Siv,
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = Instrument)]
pub struct CtrInstrument {
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

impl Default for CtrInstrument {
    fn default() -> Self {
        Self {
            synth_id: None,
            enabled: true,
            ignore_key_transform: false,
            key_transform: Default::default(),
            ignore_velocity_transform: false,
            velocity_transform: Default::default(),
            channel_filter: Default::default(),
            cc_filter: Default::default(),
            ignore_pitch_bend: false,
            ignore_program_change: false,
            ignore_channel_aftertouch: false,
            ignore_polyphonic_aftertouch: false,
            use_global_transposition: true,
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = InstrumentGraph)]
pub struct CtrInstrumentGraph {
    pub instruments: Siv<CtrInstrument, InstrumentId>,
    pub global_transposition: i8,
}

impl CtrInstrumentGraph {
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

    pub fn append_new(&mut self) -> InstrumentId {
        self.instruments.push(CtrInstrument::default())
    }

    pub fn remove(&mut self, instrument_id: InstrumentId) {
        let _ = self.instruments.try_remove(instrument_id);
    }

    pub fn set_synth(&mut self, instrument_id: InstrumentId, synth_id: Option<SynthId>) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.synth_id = synth_id;
        }
    }

    pub fn set_enabled(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.enabled = flag;
        }
    }

    pub fn set_ignore_key_transform(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.ignore_key_transform = flag;
        }
    }

    pub fn set_key_transform(&mut self, instrument_id: InstrumentId, transform: MidiKeyTransform) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.key_transform = transform;
        }
    }

    pub fn set_velocity_transform(
        &mut self,
        instrument_id: InstrumentId,
        transform: MidiVelocityTransform,
    ) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.velocity_transform = transform;
        }
    }

    pub fn set_ignore_velocity_transform(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.ignore_velocity_transform = flag;
        }
    }

    pub fn set_channel_filter(&mut self, instrument_id: InstrumentId, filter: MidiFilter) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.channel_filter = filter;
        }
    }

    pub fn set_cc_filter(&mut self, instrument_id: InstrumentId, filter: MidiFilter) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.cc_filter = filter;
        }
    }

    pub fn set_ignore_pitch_bend(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.ignore_pitch_bend = flag;
        }
    }

    pub fn set_ignore_program_change(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.ignore_program_change = flag;
        }
    }

    pub fn set_ignore_channel_aftertouch(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.ignore_channel_aftertouch = flag;
        }
    }

    pub fn set_ignore_polyphonic_aftertouch(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.ignore_polyphonic_aftertouch = flag;
        }
    }

    pub fn set_use_global_transposition(&mut self, instrument_id: InstrumentId, flag: bool) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.use_global_transposition = flag;
        }
    }

    pub fn set_name(&mut self, instrument_id: InstrumentId, name: String) {
        if let Some(instrument) = self.instruments.get_mut(instrument_id) {
            instrument.name = name;
        }
    }
}
