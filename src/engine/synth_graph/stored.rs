use crate::engine::synth_graph::{
    SlotId,
    ctr::{self, Slot},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub slots: HashMap<SlotId, Slot>,
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
