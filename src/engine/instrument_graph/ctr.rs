use crate::{
    engine::{
        controller_graph::CtrController,
        effects::Effects,
        graph::{ControllerId, InstrumentId},
    },
    memory::Siv,
    synths::{ClapParameterInfo, SfzInfo, SoundFontInfo, SoundFontProgram},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::path::PathBuf;
use utoipa::ToSchema;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = Instrument)]
pub struct CtrInstrument {
    pub name: String,
    pub kind: CtrInstrumentKind,

    #[schema(value_type = std::collections::HashMap<ControllerId, CtrController>)]
    pub controllers: Siv<CtrController, ControllerId>,
    pub effects: Effects,
    pub gain: f32,

    #[serde_as(as = "Box<[_; 128]>")]
    #[schema(value_type = [Option<u8>; 128])]
    pub last_controller_values: Box<[Option<u8>; 128]>,
}

pub enum CtrInstrumentProgramChange {
    SoundFont(SoundFontProgram),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CtrInstrumentKind {
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
        preload_size: u32,
    },
    Clap {
        #[schema(value_type = String)]
        file_path: PathBuf,

        plugin_id: String,

        #[schema(required)]
        plugin_name: Option<String>,

        #[schema(required)]
        plugin_version: Option<String>,

        plugin_parameters: Vec<ClapParameterInfo>,
    },
}

impl CtrInstrumentKind {
    fn change_program(&mut self, preset_id: u8) -> Option<CtrInstrumentProgramChange> {
        match self {
            Self::Fluid { program, .. } => {
                *program = SoundFontProgram { bank: program.bank, preset_id };
                Some(CtrInstrumentProgramChange::SoundFont(*program))
            }
            _ => None,
        }
    }

    fn change_bank_lsb(&mut self, bank: u8) -> Option<CtrInstrumentProgramChange> {
        match self {
            Self::Fluid { program, .. } => {
                *program = SoundFontProgram { bank: bank.into(), preset_id: program.preset_id };
                Some(CtrInstrumentProgramChange::SoundFont(*program))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = InstrumentGraph)]
pub struct CtrInstrumentGraph {
    #[schema(value_type = std::collections::HashMap<InstrumentId, CtrInstrument>)]
    pub instruments: Siv<CtrInstrument, InstrumentId>,
    pub global_effects: Effects,
    pub global_gain: f32,
    pub global_transposition: i8,
}

impl CtrInstrumentGraph {
    pub fn new(
        instruments: Siv<CtrInstrument, InstrumentId>,
        global_effects: Effects,
        global_gain: f32,
        global_transposition: i8,
    ) -> Self {
        Self { instruments, global_effects, global_gain, global_transposition }
    }

    pub fn save_controller_value(&mut self, id: InstrumentId, controller: u8, value: u8) {
        if let Some(instrument) = self.instruments.get_mut(id)
            && let Some(slot) = instrument.last_controller_values.get_mut(controller as usize)
        {
            *slot = Some(value);
        }
    }

    pub fn change_program(
        &mut self,
        id: InstrumentId,
        program: u8,
    ) -> Option<CtrInstrumentProgramChange> {
        self.instruments.get_mut(id)?.kind.change_program(program)
    }

    pub fn change_bank_lsb(
        &mut self,
        id: InstrumentId,
        bank: u8,
    ) -> Option<CtrInstrumentProgramChange> {
        self.instruments.get_mut(id)?.kind.change_bank_lsb(bank)
    }

    pub fn push(&mut self, instrument: CtrInstrument) -> InstrumentId {
        self.instruments.push(instrument)
    }

    pub fn remove(&mut self, id: InstrumentId) {
        let _ = self.instruments.try_remove(id);
    }

    pub fn replace_kind(&mut self, id: InstrumentId, kind: CtrInstrumentKind) {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.kind = kind;
        }
    }

    pub fn controller(
        &self,
        instrument: InstrumentId,
        controller: ControllerId,
    ) -> Option<&CtrController> {
        self.instruments.get(instrument)?.controllers.get(controller)
    }

    pub fn controller_mut(
        &mut self,
        instrument: InstrumentId,
        controller: ControllerId,
    ) -> Option<&mut CtrController> {
        self.instruments.get_mut(instrument)?.controllers.get_mut(controller)
    }

    pub fn append_controller(&mut self, instrument: InstrumentId) -> Option<ControllerId> {
        Some(self.instruments.get_mut(instrument)?.controllers.push(CtrController::default()))
    }

    pub fn remove_controller(
        &mut self,
        instrument: InstrumentId,
        controller: ControllerId,
    ) -> bool {
        self.instruments
            .get_mut(instrument)
            .and_then(|instrument| instrument.controllers.try_remove(controller))
            .is_some()
    }

    pub fn select_soundfont_program(&mut self, id: InstrumentId, program: SoundFontProgram) {
        if let Some(instrument) = self.instruments.get_mut(id)
            && let CtrInstrumentKind::Fluid { program: current, .. } = &mut instrument.kind
        {
            *current = program;
        }
    }

    pub fn set_clap_parameter(
        &mut self,
        id: InstrumentId,
        parameter_id: u32,
        value: f64,
    ) -> crate::synths::Result<()> {
        let parameter = self.clap_parameter_mut(id, parameter_id)?;
        parameter.set_value(value)
    }

    pub fn validate_clap_parameter(
        &self,
        id: InstrumentId,
        parameter_id: u32,
        value: f64,
    ) -> crate::synths::Result<()> {
        let Some(instrument) = self.instruments.get(id) else {
            return Err(invalid_parameter(parameter_id, "instrument does not exist"));
        };
        let CtrInstrumentKind::Clap { plugin_parameters, .. } = &instrument.kind else {
            return Err(invalid_parameter(parameter_id, "instrument is not a CLAP plugin"));
        };
        let Some(mut parameter) =
            plugin_parameters.iter().find(|parameter| parameter.id == parameter_id).cloned()
        else {
            return Err(invalid_parameter(parameter_id, "parameter does not exist"));
        };
        parameter.set_value(value)
    }

    pub fn update_clap_parameter(
        &mut self,
        id: InstrumentId,
        parameter_id: u32,
        value: f64,
    ) -> bool {
        let Ok(parameter) = self.clap_parameter_mut(id, parameter_id) else {
            return false;
        };
        parameter.value = Some(value);
        parameter.value_text = None;
        true
    }

    fn clap_parameter_mut(
        &mut self,
        id: InstrumentId,
        parameter_id: u32,
    ) -> crate::synths::Result<&mut ClapParameterInfo> {
        let Some(instrument) = self.instruments.get_mut(id) else {
            return Err(invalid_parameter(parameter_id, "instrument does not exist"));
        };
        let CtrInstrumentKind::Clap { plugin_parameters, .. } = &mut instrument.kind else {
            return Err(invalid_parameter(parameter_id, "instrument is not a CLAP plugin"));
        };
        plugin_parameters
            .iter_mut()
            .find(|parameter| parameter.id == parameter_id)
            .ok_or_else(|| invalid_parameter(parameter_id, "parameter does not exist"))
    }
}

fn invalid_parameter(parameter_id: u32, reason: &'static str) -> crate::synths::Error {
    crate::synths::Error::InvalidClapParameter { parameter_id, reason }
}
