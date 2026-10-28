use crate::engine::{
    graph::InstrumentId,
    user_presets::{CtrUserPreset, CtrUserPresets},
};
use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoredUserPreset {
    pub name: String,
    pub instruments: Vec<InstrumentId>,
}

impl From<&CtrUserPreset> for StoredUserPreset {
    fn from(value: &CtrUserPreset) -> Self {
        Self { name: value.name.clone(), instruments: value.instruments.clone() }
    }
}

impl From<CtrUserPreset> for StoredUserPreset {
    fn from(value: CtrUserPreset) -> Self {
        Self { name: value.name, instruments: value.instruments }
    }
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredUserPresets {
    pub entries: Vec<StoredUserPreset>,
}

impl From<&CtrUserPresets> for StoredUserPresets {
    fn from(value: &CtrUserPresets) -> Self {
        Self { entries: value.entries.iter().map(Into::into).collect() }
    }
}

impl From<CtrUserPresets> for StoredUserPresets {
    fn from(value: CtrUserPresets) -> Self {
        Self { entries: value.entries.into_iter().map(Into::into).collect() }
    }
}
