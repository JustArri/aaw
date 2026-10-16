use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum MidiFilterKind {
    Allowlist,
    Denylist,
}

impl MidiFilterKind {
    fn application_value(&self) -> bool {
        match self {
            MidiFilterKind::Allowlist => true,
            MidiFilterKind::Denylist => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MidiFilter {
    #[serde(rename = "type")]
    pub kind: MidiFilterKind,

    pub list: HashSet<u8>,
}

impl MidiFilter {
    pub fn as_u16(&self) -> u16 {
        let value = self.kind.application_value();
        let mut mask = if value { 0 } else { u16::MAX };

        for &item in &self.list {
            if item < 16 {
                let bit = 1u16 << item;

                if value {
                    mask |= bit;
                } else {
                    mask &= !bit;
                }
            }
        }

        mask
    }

    pub fn as_u128(&self) -> u128 {
        let value = self.kind.application_value();
        let mut mask = if value { 0 } else { u128::MAX };

        for &item in &self.list {
            if item < 128 {
                let bit = 1u128 << item;

                if value {
                    mask |= bit;
                } else {
                    mask &= !bit;
                }
            }
        }

        mask
    }
}

impl Default for MidiFilter {
    fn default() -> Self {
        Self { kind: MidiFilterKind::Denylist, list: Default::default() }
    }
}
