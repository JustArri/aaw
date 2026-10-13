use crate::{engine::graph::LocalSynthId, memory::Siv};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Synth;

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

    pub fn remove(&mut self, synth_id: LocalSynthId) {
        let _ = self.synths.try_remove(synth_id);
    }
}
