use crate::{
    audio::block::AudioBlock,
    engine::graph::{LocalSynthId, RETIREMENT_QUEUE_CAPACITY},
    memory::{
        Siv,
        retirement::{RetirementThread, Retirer},
    },
    midi::MidiEvent,
    synths,
};
use std::time::Duration;

pub struct Graph {
    pub retirer: Retirer<synths::Sfizz>,
    pub retirement_thread: RetirementThread,
    pub synths: Siv<synths::Sfizz, LocalSynthId>,
}

impl Graph {
    pub fn new(synths: Siv<synths::Sfizz, LocalSynthId>) -> Self {
        let (retirer, retirement_thread) =
            RetirementThread::spawn(RETIREMENT_QUEUE_CAPACITY, Duration::from_millis(10))
                .expect("failed to spawn retirement thread");

        Self { retirer, retirement_thread, synths }
    }

    pub fn destroy(self) {
        let _ = self.retirement_thread.join_after_dropping(self.retirer);
    }

    pub fn panic(&mut self) {
        for synth in self.synths.iter_values_mut() {
            synth.panic();
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> synths::Result<()> {
        for synth in self.synths.iter_values_mut() {
            synth.set_sample_rate(sample_rate)?;
        }
        Ok(())
    }

    pub fn handle_midi_event(&mut self, id: LocalSynthId, event: MidiEvent) {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.handle_midi_event(event);
        }
    }

    pub fn render_audio_block(
        &mut self,
        id: LocalSynthId,
        output: &mut AudioBlock,
    ) -> synths::Result<()> {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.render_audio_block(output)?;
        }
        Ok(())
    }

    pub fn push(&mut self, synth: synths::Sfizz) -> bool {
        if self.synths.len() == self.synths.capacity() {
            self.retirer.retire_or_leak(synth);
            return false;
        }
        self.synths.push(synth);
        true
    }

    pub fn replace(&mut self, synth_id: LocalSynthId, synth: synths::Sfizz) -> bool {
        match self.synths.try_replace(synth_id, synth) {
            Ok(previous) => {
                self.retirer.retire_or_leak(previous);
                true
            }
            Err(synth) => {
                self.retirer.retire_or_leak(synth);
                false
            }
        }
    }

    pub fn remove(&mut self, synth_id: LocalSynthId) -> bool {
        if let Some(synth) = self.synths.try_remove(synth_id) {
            self.retirer.retire_or_leak(synth);
            true
        } else {
            false
        }
    }
}
