use crate::{
    engine::{effects::Effects, graph::SynthId},
    memory::Siv,
    synths::{SfzInfo, SoundFontInfo, SoundFontProgram},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = Synth)]
pub struct CtrSynth {
    pub name: String,
    pub parameters: CtrSynthParameters,
    pub effects: Effects,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CtrSynthParameters {
    Dummy,
    Fluid {
        #[schema(value_type = String)]
        file_path: PathBuf,

        info: SoundFontInfo,
        program: SoundFontProgram,
    },
    Sfizz {
        #[schema(value_type = String)]
        file_path: PathBuf,

        info: SfzInfo,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = SynthGraph)]
pub struct CtrSynthGraph {
    pub synths: Siv<CtrSynth, SynthId>,
    pub effects: Effects,
}

impl CtrSynthGraph {
    pub fn new(synths: Siv<CtrSynth, SynthId>, effects: Effects) -> Self {
        Self { synths, effects }
    }

    pub fn set_effects(&mut self, effects: Effects) {
        self.effects = effects;
    }

    pub fn push(&mut self, synth: CtrSynth) -> SynthId {
        self.synths.push(synth)
    }

    pub fn replace(&mut self, id: SynthId, parameters: CtrSynthParameters) {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.parameters = parameters;
        }
    }

    pub fn remove(&mut self, id: SynthId) {
        let _ = self.synths.try_remove(id);
    }

    pub fn synth_select_soundfont_program(&mut self, id: SynthId, program: SoundFontProgram) {
        if let Some(synth) = self.synths.get_mut(id) {
            match &mut synth.parameters {
                CtrSynthParameters::Dummy => {}
                CtrSynthParameters::Fluid { program: fluid_program, .. } => {
                    *fluid_program = program
                }
                CtrSynthParameters::Sfizz { .. } => {}
            }
        }
    }

    pub fn synth_set_name(&mut self, id: SynthId, name: String) {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.name = name;
        }
    }

    pub fn synth_set_effects(&mut self, id: SynthId, effects: Effects) {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.effects = effects;
        }
    }
}
