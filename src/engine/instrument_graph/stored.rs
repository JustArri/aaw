use crate::engine::{
    graph::LocalId,
    instrument_graph::ctr::{self, Slot},
};
use migrations_derive::Migrations;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
#[schema(as = InstrumentGraph)]
pub struct Graph {
    pub slots: HashMap<LocalId, Slot>,
    pub global_transposition: i8,
}

impl From<&ctr::Graph> for Graph {
    fn from(value: &ctr::Graph) -> Self {
        Self { slots: value.slots.clone().into(), global_transposition: value.global_transposition }
    }
}

impl From<ctr::Graph> for Graph {
    fn from(value: ctr::Graph) -> Self {
        Self { slots: value.slots.into(), global_transposition: value.global_transposition }
    }
}
