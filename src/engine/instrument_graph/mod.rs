use crate::{engine::graph::LocalId, memory::Siv};
pub use ctr::*;
pub use rt::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub use stored::*;
use utoipa::ToSchema;

mod ctr;
mod rt;
mod stored;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum SynthKind {
    Dummy,
    Fluid,
    Sfizz,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SynthId {
    pub kind: SynthKind,
    pub id: LocalId,
}

pub fn prepare(
    stored: StoredGraph,
    num_slots: usize,
    _channel_count: usize,
    _sample_rate: u32,
) -> (CtrGraph, RtGraph) {
    let global_transposition = stored.global_transposition;
    let mut ctr_slots = HashMap::new();
    let mut rt_slots = HashMap::new();

    for (slot_id, stored_slot) in stored.slots {
        let (ctr_slot, rt_slot) = prepare_slot(stored_slot);
        ctr_slots.insert(slot_id, ctr_slot);
        rt_slots.insert(slot_id, rt_slot);
    }

    let mut rt_slots: Siv<_, _> = rt_slots.into();
    let mut ctr_slots: Siv<_, _> = ctr_slots.into();

    rt_slots.ensure_capacity(num_slots);
    ctr_slots.ensure_capacity(num_slots);

    let rt_graph = RtGraph { slots: rt_slots, global_transposition };
    let ctr_graph = CtrGraph { slots: ctr_slots, global_transposition };

    (ctr_graph, rt_graph)
}

pub fn prepare_slot(stored: StoredSlot) -> (CtrSlot, rt::Slot) {
    let rt_slot = rt::Slot {
        synth_id: stored.synth_id,
        enabled: stored.enabled,
        key_transform: stored.key_transform.compile(),
        velocity_transform: stored.velocity_transform.compile(),
        channel_mask: stored.channel_filter.as_u16(),
        cc_mask: stored.cc_filter.as_u128(),
        use_global_transposition: stored.use_global_transposition,
    };
    let ctr_slot = CtrSlot {
        synth_id: stored.synth_id,
        enabled: stored.enabled,
        key_transform: stored.key_transform,
        velocity_transform: stored.velocity_transform,
        channel_filter: stored.channel_filter,
        cc_filter: stored.cc_filter,
        use_global_transposition: stored.use_global_transposition,
        name: stored.name,
    };
    (ctr_slot, rt_slot)
}
