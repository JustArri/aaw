use crate::{
    audio::block::AudioBlock,
    engine::graph::{LocalId, RETIREMENT_QUEUE_CAPACITY},
    memory::{
        Siv,
        retirement::{RetirementThread, Retirer},
    },
    midi::MidiEvent,
    synths,
};
use std::time::Duration;

pub struct Graph {
    pub retirer: Retirer<synths::Dummy>,
    pub retirement_thread: RetirementThread,
    pub synths: Siv<synths::Dummy, LocalId>,
}

impl Graph {
    pub fn new(synths: Siv<synths::Dummy, LocalId>) -> Self {
        let (retirer, retirement_thread) =
            RetirementThread::spawn(RETIREMENT_QUEUE_CAPACITY, Duration::from_millis(10))
                .expect("failed to spawn retirement thread");

        Self { retirer, retirement_thread, synths }
    }

    pub fn destroy(self) {
        let _ = self.retirement_thread.join_after_dropping(self.retirer);
    }

    pub fn silence(&mut self) {
        for synth in self.synths.iter_values_mut() {
            synth.silence();
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        for synth in self.synths.iter_values_mut() {
            synth.set_sample_rate(sample_rate);
        }
    }

    pub fn handle_midi_event(&mut self, id: LocalId, event: MidiEvent) {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.handle_midi_event(event);
        }
    }

    pub fn render_audio_block(&mut self, id: LocalId, output: &mut AudioBlock) {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.render_audio_block(output);
        }
    }

    pub fn push(&mut self, synth: synths::Dummy) -> bool {
        if self.synths.len() == self.synths.capacity() {
            self.retirer.retire_or_leak(synth);
            return false;
        }
        self.synths.push(synth);
        true
    }

    pub fn remove(&mut self, synth_id: LocalId) -> bool {
        if let Some(synth) = self.synths.try_remove(synth_id) {
            self.retirer.retire_or_leak(synth);
            true
        } else {
            false
        }
    }
}
