use crate::{engine::graph::LocalId, memory::Siv};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = DummySynth)]
pub struct CtrSynth {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = DummyGraph)]
pub struct CtrGraph {
    pub synths: Siv<CtrSynth, LocalId>,
}

impl CtrGraph {
    pub fn new(synths: Siv<CtrSynth, LocalId>) -> Self {
        Self { synths }
    }

    pub fn push(&mut self, synth: CtrSynth) -> LocalId {
        self.synths.push(synth)
    }

    pub fn remove(&mut self, synth_id: LocalId) {
        let _ = self.synths.try_remove(synth_id);
    }

    pub fn set_name(&mut self, synth_id: LocalId, name: String) {
        if let Some(synth) = self.synths.get_mut(synth_id) {
            synth.name = name;
        }
    }
}
