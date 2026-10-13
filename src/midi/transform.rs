use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct MidiKeyTransform {
    pub selection: MidiKeySelection,
    pub mapping: MidiKeyMapping,
    pub transposition: i8,
}

impl MidiKeyTransform {
    pub fn apply(&self, key: u8, transposition: i8) -> Option<u8> {
        transpose(self.apply_local(key)?, transposition)
    }

    pub fn compile(&self) -> CompiledMidiKeyTransform {
        CompiledMidiKeyTransform::new(self)
    }

    fn apply_local(&self, key: u8) -> Option<u8> {
        if key >= 128 {
            return None;
        }

        let selected = match &self.selection {
            MidiKeySelection::All => true,
            MidiKeySelection::Range { min, max } => (*min..=*max).contains(&key),
            MidiKeySelection::Keys { keys } => keys.contains(&key),
        };

        if !selected {
            return None;
        }

        let key = match &self.mapping {
            MidiKeyMapping::Identity => key,
            MidiKeyMapping::Custom { map } => *map.get(&key)?,
        };

        transpose(key, self.transposition)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum MidiKeySelection {
    #[default]
    All,

    Range {
        min: u8,
        max: u8,
    },

    Keys {
        keys: Vec<u8>,
    },
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum MidiKeyMapping {
    #[default]
    Identity,

    Custom {
        map: HashMap<u8, u8>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledMidiKeyTransform {
    keys: [Option<u8>; 128],
}

impl CompiledMidiKeyTransform {
    pub fn new(config: &MidiKeyTransform) -> Self {
        let keys = std::array::from_fn(|key| config.apply_local(key as u8));

        Self { keys }
    }

    pub fn update(&mut self, config: &MidiKeyTransform) {
        for (key, output) in self.keys.iter_mut().enumerate() {
            *output = config.apply_local(key as u8);
        }
    }

    #[inline]
    pub fn apply(&self, key: u8, transposition: i8) -> Option<u8> {
        let key = self.keys.get(key as usize).copied().flatten()?;
        transpose(key, transposition)
    }
}

impl From<&MidiKeyTransform> for CompiledMidiKeyTransform {
    fn from(config: &MidiKeyTransform) -> Self {
        Self::new(config)
    }
}

#[inline]
pub fn transpose(key: u8, amount: i8) -> Option<u8> {
    let result = i16::from(key) + i16::from(amount);
    u8::try_from(result).ok().filter(|&key| key < 128)
}
