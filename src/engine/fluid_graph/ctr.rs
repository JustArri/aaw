use crate::{
    engine::graph::LocalId,
    memory::Siv,
    synths::{SoundFontInfo, SoundFontProgram},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = FluidSynth)]
pub struct CtrSynth {
    pub name: String,

    #[schema(value_type = String)]
    pub file_path: PathBuf,

    pub info: SoundFontInfo,
    pub program: SoundFontProgram,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = FluidGraph)]
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

    pub fn replace(&mut self, synth_id: LocalId, synth: CtrSynth) {
        let _ = self.synths.try_replace(synth_id, synth);
    }

    pub fn remove(&mut self, synth_id: LocalId) {
        let _ = self.synths.try_remove(synth_id);
    }

    pub fn select_program(&mut self, synth_id: LocalId, program: SoundFontProgram) {
        if let Some(synth) = self.synths.get_mut(synth_id) {
            synth.program = program;
        }
    }

    pub fn set_name(&mut self, synth_id: LocalId, name: String) {
        if let Some(synth) = self.synths.get_mut(synth_id) {
            synth.name = name;
        }
    }
}
