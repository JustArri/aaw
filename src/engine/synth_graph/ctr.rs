use crate::{
    engine::{effects::Effects, graph::SynthId},
    memory::Siv,
    synths::{SfzInfo, SoundFontInfo, SoundFontProgram},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::path::PathBuf;
use utoipa::ToSchema;

// TODO: Implement configurable CC memory (restore last CC values if listed)
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = Synth)]
pub struct CtrSynth {
    pub name: String,
    pub parameters: CtrSynthParameters,
    pub effects: Effects,
    pub gain: f32,

    #[serde_as(as = "Box<[_; 128]>")]
    #[schema(value_type = [Option<u8>; 128])]
    pub last_controller_values: Box<[Option<u8>; 128]>,
}

pub enum CtrSynthProgramChange {
    SoundFont(SoundFontProgram),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CtrSynthParameters {
    Dummy {
        max_voices: usize,
    },
    Fluid {
        #[schema(value_type = String)]
        file_path: PathBuf,

        info: SoundFontInfo,
        program: SoundFontProgram,
        max_voices: usize,
    },
    Sfizz {
        #[schema(value_type = String)]
        file_path: PathBuf,

        info: SfzInfo,
        max_voices: usize,
    },
}

impl CtrSynthParameters {
    fn change_program(&mut self, new_preset_id: u8) -> Option<CtrSynthProgramChange> {
        match self {
            CtrSynthParameters::Dummy { .. } => None,
            CtrSynthParameters::Fluid { program, .. } => {
                *program = SoundFontProgram { bank: program.bank, preset_id: new_preset_id };
                Some(CtrSynthProgramChange::SoundFont(*program))
            }
            CtrSynthParameters::Sfizz { .. } => None,
        }
    }

    fn change_bank_lsb(&mut self, new_bank_lsb: u8) -> Option<CtrSynthProgramChange> {
        match self {
            CtrSynthParameters::Dummy { .. } => None,
            CtrSynthParameters::Fluid { program, .. } => {
                *program =
                    SoundFontProgram { bank: new_bank_lsb as u16, preset_id: program.preset_id };
                Some(CtrSynthProgramChange::SoundFont(*program))
            }
            CtrSynthParameters::Sfizz { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = SynthGraph)]
pub struct CtrSynthGraph {
    pub synths: Siv<CtrSynth, SynthId>,
    pub global_effects: Effects,
    pub global_gain: f32,
}

impl CtrSynthGraph {
    pub fn new(synths: Siv<CtrSynth, SynthId>, global_effects: Effects, global_gain: f32) -> Self {
        Self { synths, global_effects, global_gain }
    }

    pub fn save_controller_value(&mut self, id: SynthId, controller: u8, value: u8) {
        if let Some(synth) = self.synths.get_mut(id)
            && let Some(controller_value) =
                synth.last_controller_values.get_mut(controller as usize)
        {
            *controller_value = Some(value);
        }
    }

    pub fn change_program(
        &mut self,
        id: SynthId,
        new_preset_id: u8,
    ) -> Option<CtrSynthProgramChange> {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.parameters.change_program(new_preset_id)
        } else {
            None
        }
    }

    pub fn change_bank_lsb(
        &mut self,
        id: SynthId,
        new_bank_lsb: u8,
    ) -> Option<CtrSynthProgramChange> {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.parameters.change_bank_lsb(new_bank_lsb)
        } else {
            None
        }
    }

    pub fn set_global_effects(&mut self, effects: Effects) {
        self.global_effects = effects;
    }

    pub fn set_global_gain(&mut self, gain: f32) {
        self.global_gain = gain;
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
                CtrSynthParameters::Dummy { .. } => {}
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

    pub fn set_synth_gain(&mut self, id: SynthId, gain: f32) {
        if let Some(synth) = self.synths.get_mut(id) {
            synth.gain = gain;
        }
    }
}
