use crate::{engine::graph::LocalSynthId, memory::Siv, synths::SoundFontInfo};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Synth {
    pub file_path: String,
    pub info: SoundFontInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Graph {
    pub synths: Siv<Synth, LocalSynthId>,
}

impl Graph {
    pub fn new(synths: Siv<Synth, LocalSynthId>) -> Self {
        Self { synths }
    }

    pub fn push(&mut self, synth: Synth) {
        self.synths.push(synth);
    }

    pub fn replace(&mut self, synth_id: LocalSynthId, synth: Synth) {
        let _ = self.synths.try_replace(synth_id, synth);
    }

    pub fn remove(&mut self, synth_id: LocalSynthId) {
        let _ = self.synths.try_remove(synth_id);
    }
}
