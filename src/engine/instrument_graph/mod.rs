use crate::{engine::graph::LocalId, memory::Siv};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

pub mod ctr;
pub mod rt;
pub mod stored;

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

pub fn prepare(stored: stored::Graph, num_slots: usize) -> (ctr::Graph, rt::Graph) {
    let global_transposition = stored.global_transposition;

    let rt_slots: HashMap<_, _> = stored.slots.iter().map(ctr_slot_to_rt).collect();
    let mut rt_slots: Siv<_, _> = rt_slots.into();
    let mut ctr_slots: Siv<_, _> = stored.slots.into();

    rt_slots.ensure_capacity(num_slots);
    ctr_slots.ensure_capacity(num_slots);

    let rt_graph = rt::Graph { slots: rt_slots, global_transposition };
    let ctr_graph = ctr::Graph { slots: ctr_slots, global_transposition };

    (ctr_graph, rt_graph)
}

fn ctr_slot_to_rt((slot_id, slot): (&LocalId, &ctr::Slot)) -> (LocalId, rt::Slot) {
    let slot = rt::Slot {
        synth_id: slot.synth_id,
        enabled: slot.enabled,
        key_transform: slot.key_transform.compile(),
        channel_mask: slot.channel_filter.as_u16(),
        cc_mask: slot.cc_filter.as_u128(),
        use_global_transposition: slot.use_global_transposition,
    };
    (*slot_id, slot)
}
