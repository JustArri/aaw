use crate::{
    DEFAULT_SAMPLE_RATE,
    audio::AudioBlock,
    engine::{
        effects::RtEffects,
        graph::{RETIREMENT_QUEUE_CAPACITY, SynthId},
        synth_graph::SynthGraphError,
    },
    memory::{
        Siv,
        retirement::{RetirementThread, Retirer},
    },
    midi::MidiEvent,
    synths::{self, SoundFontProgram},
};
use std::time::Duration;

pub struct RtSynth {
    pub kind: RtSynthKind,
    pub effects: RtEffects,
    pub gain: f32,
}

pub enum RtSynthKind {
    Dummy(synths::Dummy),
    Fluid(synths::Fluid),
    Sfizz(synths::Sfizz),
}

impl RtSynth {
    pub fn silence(&mut self) {
        match &mut self.kind {
            RtSynthKind::Dummy(synth) => synth.silence(),
            RtSynthKind::Fluid(synth) => {
                let _ = synth.silence();
            }
            RtSynthKind::Sfizz(synth) => synth.silence(),
        }
    }

    pub fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), SynthGraphError> {
        self.effects.reconfigure(channel_count, sample_rate)?;
        self.set_sample_rate(sample_rate)?;
        self.set_max_block_size(max_block_size)?;
        Ok(())
    }

    pub fn handle_midi_event(&mut self, event: MidiEvent) -> Result<(), SynthGraphError> {
        match &mut self.kind {
            RtSynthKind::Dummy(synth) => synth.handle_midi_event(event),
            RtSynthKind::Fluid(synth) => synth.handle_midi_event(event)?,
            RtSynthKind::Sfizz(synth) => synth.handle_midi_event(event),
        }
        Ok(())
    }

    pub fn render_audio_block(&mut self, output: &mut AudioBlock) -> Result<(), SynthGraphError> {
        match &mut self.kind {
            RtSynthKind::Dummy(synth) => synth.render_audio_block(output),
            RtSynthKind::Fluid(synth) => synth.render_audio_block(output)?,
            RtSynthKind::Sfizz(synth) => synth.render_audio_block(output)?,
        }
        self.effects.apply(output);
        output.multiply_samples(self.gain);
        Ok(())
    }

    pub fn select_soundfont_program(&mut self, program: SoundFontProgram) -> bool {
        match &mut self.kind {
            RtSynthKind::Dummy(_) => false,
            RtSynthKind::Fluid(synth) => synth.select_program(program).is_ok(),
            RtSynthKind::Sfizz(_) => false,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), SynthGraphError> {
        match &mut self.kind {
            RtSynthKind::Dummy(synth) => synth.set_sample_rate(sample_rate),
            RtSynthKind::Fluid(synth) => synth.set_sample_rate(sample_rate)?,
            RtSynthKind::Sfizz(synth) => synth.set_sample_rate(sample_rate)?,
        }
        Ok(())
    }

    fn set_max_block_size(&mut self, max_block_size: usize) -> Result<(), SynthGraphError> {
        match &mut self.kind {
            RtSynthKind::Dummy(synth) => synth.set_max_block_size(max_block_size),
            RtSynthKind::Fluid(synth) => synth.set_max_block_size(max_block_size)?,
            RtSynthKind::Sfizz(synth) => synth.set_max_block_size(max_block_size)?,
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
pub enum RetiredObject {
    Synth(RtSynth),
    SynthKind(RtSynthKind),
}

pub struct RtSynthGraph {
    pub retirer: Retirer<RetiredObject>,
    pub retirement_thread: RetirementThread,
    pub synths: Siv<RtSynth, SynthId>,
    pub global_effects: RtEffects,
    pub target_global_gain: f32,
    pub global_gain: f32,
    pub sample_rate: f32,
}

impl RtSynthGraph {
    pub fn new(synths: Siv<RtSynth, SynthId>, global_effects: RtEffects, global_gain: f32) -> Self {
        let (retirer, retirement_thread) =
            RetirementThread::spawn(RETIREMENT_QUEUE_CAPACITY, Duration::from_millis(10))
                .expect("failed to spawn retirement thread");

        Self {
            retirer,
            retirement_thread,
            synths,
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
        for synth in self.synths.iter_values_mut() {
            synth.silence();
        }
    }

    pub fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), SynthGraphError> {
        self.sample_rate = sample_rate as f32;

        for synth in self.synths.iter_values_mut() {
            synth.reconfigure(channel_count, sample_rate, max_block_size)?;
        }

        Ok(())
    }

    pub fn handle_midi_event(
        &mut self,
        id: SynthId,
        event: MidiEvent,
    ) -> Result<(), SynthGraphError> {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.handle_midi_event(event)?;
        }
        Ok(())
    }

    pub fn set_global_effects(&mut self, effects: RtEffects) {
        self.global_effects = effects;
    }

    pub fn set_global_gain(&mut self, gain: f32) {
        self.target_global_gain = gain;
    }

    pub fn push(&mut self, synth: RtSynth) -> bool {
        if self.synths.len() == self.synths.capacity() {
            self.retirer.retire_or_leak(RetiredObject::Synth(synth));
            return false;
        }
        self.synths.push(synth);
        true
    }

    pub fn replace(&mut self, id: SynthId, kind: RtSynthKind) -> bool {
        if let Some(synth) = self.synths.get_mut(id) {
            let previous = std::mem::replace(&mut synth.kind, kind);
            self.retirer.retire_or_leak(RetiredObject::SynthKind(previous));
            true
        } else {
            self.retirer.retire_or_leak(RetiredObject::SynthKind(kind));
            false
        }
    }

    pub fn remove(&mut self, id: SynthId) -> bool {
        if let Some(synth) = self.synths.try_remove(id) {
            self.retirer.retire_or_leak(RetiredObject::Synth(synth));
            true
        } else {
            false
        }
    }

    pub fn select_soundfont_program(&mut self, id: SynthId, program: SoundFontProgram) -> bool {
        if let Some(synth) = self.synths.get_mut(id) {
            match &mut synth.kind {
                RtSynthKind::Dummy(_) => false,
                RtSynthKind::Fluid(synth) => synth.select_program(program).is_ok(),
                RtSynthKind::Sfizz(_) => false,
            }
        } else {
            false
        }
    }

    pub fn set_synth_effects(&mut self, id: SynthId, effects: RtEffects) -> bool {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.effects = effects;
            true
        } else {
            false
        }
    }

    pub fn set_synth_gain(&mut self, id: SynthId, gain: f32) -> bool {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.gain = gain;
            true
        } else {
            false
        }
    }
}
