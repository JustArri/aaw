use crate::{
    DEFAULT_SAMPLE_RATE,
    audio::AudioBlock,
    engine::{
        effects::RtEffects,
        graph::{InstrumentId, RETIREMENT_QUEUE_CAPACITY},
        instrument_graph::InstrumentGraphError,
    },
    memory::{
        Siv,
        retirement::{RetirementThread, Retirer},
    },
    midi::MidiEvent,
    synths::{self, SoundFontProgram},
};
use std::time::Duration;

pub struct RtInstrument {
    pub kind: RtInstrumentKind,
    pub effects: RtEffects,
    pub gain: f32,
}

#[allow(clippy::large_enum_variant)]
pub enum RtInstrumentKind {
    Dummy(synths::Dummy),
    Fluid(synths::Fluid),
    Sfizz(synths::Sfizz),
    Clap(synths::Clap),
}

impl RtInstrument {
    pub fn silence(&mut self) {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.silence(),
            RtInstrumentKind::Fluid(instrument) => {
                let _ = instrument.silence();
            }
            RtInstrumentKind::Sfizz(instrument) => instrument.silence(),
            RtInstrumentKind::Clap(instrument) => instrument.silence(),
        }
    }

    pub async fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), InstrumentGraphError> {
        self.effects.reconfigure(channel_count, sample_rate)?;
        if let RtInstrumentKind::Clap(instrument) = &mut self.kind {
            instrument.reconfigure(channel_count, sample_rate, max_block_size).await?;
        } else {
            self.set_sample_rate(sample_rate)?;
            self.set_max_block_size(max_block_size)?;
        }
        Ok(())
    }

    pub fn handle_midi_event(&mut self, event: MidiEvent) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.handle_midi_event(event),
            RtInstrumentKind::Fluid(instrument) => instrument.handle_midi_event(event)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.handle_midi_event(event),
            RtInstrumentKind::Clap(instrument) => instrument.handle_midi_event(event),
        }
        Ok(())
    }

    pub fn render_audio_block(
        &mut self,
        output: &mut AudioBlock,
    ) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.render_audio_block(output),
            RtInstrumentKind::Fluid(instrument) => instrument.render_audio_block(output)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.render_audio_block(output)?,
            RtInstrumentKind::Clap(instrument) => instrument.render_audio_block(output)?,
        }
        self.effects.apply(output);
        output.multiply_samples(self.gain);
        Ok(())
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.set_sample_rate(sample_rate),
            RtInstrumentKind::Fluid(instrument) => instrument.set_sample_rate(sample_rate)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.set_sample_rate(sample_rate)?,
            RtInstrumentKind::Clap(_) => {}
        }
        Ok(())
    }

    fn set_max_block_size(&mut self, size: usize) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.set_max_block_size(size),
            RtInstrumentKind::Fluid(instrument) => instrument.set_max_block_size(size)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.set_max_block_size(size)?,
            RtInstrumentKind::Clap(_) => {}
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
pub enum RetiredObject {
    Instrument(RtInstrument),
    InstrumentKind(RtInstrumentKind),
}

pub struct RtInstrumentGraph {
    pub retirer: Retirer<RetiredObject>,
    pub retirement_thread: RetirementThread,
    pub instruments: Siv<RtInstrument, InstrumentId>,
    pub global_effects: RtEffects,
    pub target_global_gain: f32,
    pub global_gain: f32,
    pub sample_rate: f32,
}

impl RtInstrumentGraph {
    pub fn new(
        instruments: Siv<RtInstrument, InstrumentId>,
        global_effects: RtEffects,
        global_gain: f32,
    ) -> Self {
        let (retirer, retirement_thread) =
            RetirementThread::spawn(RETIREMENT_QUEUE_CAPACITY, Duration::from_millis(10))
                .expect("failed to spawn retirement thread");
        Self {
            retirer,
            retirement_thread,
            instruments,
            global_effects,
            target_global_gain: global_gain,
            global_gain,
            sample_rate: DEFAULT_SAMPLE_RATE as f32,
        }
    }

    pub fn destroy(self) {
        let _ = self.retirement_thread.join_after_dropping(self.retirer);
    }

    pub fn silence(&mut self) {
        for instrument in self.instruments.iter_values_mut() {
            instrument.silence();
        }
    }

    pub async fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), InstrumentGraphError> {
        self.sample_rate = sample_rate as f32;
        for instrument in self.instruments.iter_values_mut() {
            instrument.reconfigure(channel_count, sample_rate, max_block_size).await?;
        }
        Ok(())
    }

    pub fn handle_midi_event(
        &mut self,
        id: InstrumentId,
        event: MidiEvent,
    ) -> Result<(), InstrumentGraphError> {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.handle_midi_event(event)?;
        }
        Ok(())
    }

    pub fn push(&mut self, instrument: RtInstrument) -> bool {
        if self.instruments.len() == self.instruments.capacity() {
            self.retirer.retire_or_leak(RetiredObject::Instrument(instrument));
            return false;
        }
        self.instruments.push(instrument);
        true
    }

    pub fn replace(&mut self, id: InstrumentId, kind: RtInstrumentKind) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            let previous = std::mem::replace(&mut instrument.kind, kind);
            self.retirer.retire_or_leak(RetiredObject::InstrumentKind(previous));
            true
        } else {
            self.retirer.retire_or_leak(RetiredObject::InstrumentKind(kind));
            false
        }
    }

    pub fn remove(&mut self, id: InstrumentId) -> bool {
        if let Some(instrument) = self.instruments.try_remove(id) {
            self.retirer.retire_or_leak(RetiredObject::Instrument(instrument));
            true
        } else {
            false
        }
    }

    pub fn select_soundfont_program(
        &mut self,
        id: InstrumentId,
        program: SoundFontProgram,
    ) -> bool {
        let Some(instrument) = self.instruments.get_mut(id) else {
            return false;
        };
        match &mut instrument.kind {
            RtInstrumentKind::Fluid(instrument) => instrument.select_program(program).is_ok(),
            _ => false,
        }
    }

    pub fn set_clap_parameter(&mut self, id: InstrumentId, parameter_id: u32, value: f64) -> bool {
        let Some(instrument) = self.instruments.get_mut(id) else {
            return false;
        };
        let RtInstrumentKind::Clap(instrument) = &mut instrument.kind else {
            return false;
        };
        instrument.set_parameter(parameter_id, value)
    }

    pub fn set_effects(&mut self, id: InstrumentId, effects: RtEffects) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.effects = effects;
            true
        } else {
            false
        }
    }

    pub fn set_gain(&mut self, id: InstrumentId, gain: f32) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.gain = gain;
            true
        } else {
            false
        }
    }
}
