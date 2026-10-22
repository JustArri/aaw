use crate::memory::Siv;
pub use ctr::*;
pub use rt::*;
use std::collections::HashMap;
pub use stored::*;

mod ctr;
mod rt;
mod stored;

#[derive(Clone, Copy)]
pub struct InstrumentGraphContext {
    pub num_instruments: usize,
    pub channel_count: usize,
    pub sample_rate: u32,
}

pub fn prepare(
    stored: StoredInstrumentGraph,
    ctx: InstrumentGraphContext,
) -> (CtrInstrumentGraph, RtInstrumentGraph) {
    let global_transposition = stored.global_transposition;
    let mut ctr_instruments = HashMap::new();
    let mut rt_instruments = HashMap::new();

    for (id, strored) in stored.instruments {
        let (ctr, rt) = prepare_instrument(strored);
        ctr_instruments.insert(id, ctr);
        rt_instruments.insert(id, rt);
    }

    let mut rt_instruments: Siv<_, _> = rt_instruments.into();
    let mut ctr_instruments: Siv<_, _> = ctr_instruments.into();

    rt_instruments.ensure_capacity(ctx.num_instruments);
    ctr_instruments.ensure_capacity(ctx.num_instruments);

    let rt_graph = RtInstrumentGraph { instruments: rt_instruments, global_transposition };
    let ctr_graph = CtrInstrumentGraph { instruments: ctr_instruments, global_transposition };

    (ctr_graph, rt_graph)
}

pub fn prepare_instrument(stored: StoredInstrument) -> (CtrInstrument, RtInstrument) {
    let rt_instrument = RtInstrument {
        synth_id: stored.synth_id,
        enabled: stored.enabled,
        ignore_key_transform: stored.ignore_key_transform,
        key_transform: stored.key_transform.compile(),
        ignore_velocity_transform: stored.ignore_velocity_transform,
        velocity_transform: stored.velocity_transform.compile(),
        channel_mask: stored.channel_filter.as_u16(),
        cc_mask: stored.cc_filter.as_u128(),
        ignore_pitch_bend: stored.ignore_pitch_bend,
        ignore_program_change: stored.ignore_program_change,
        ignore_channel_aftertouch: stored.ignore_channel_aftertouch,
        ignore_polyphonic_aftertouch: stored.ignore_polyphonic_aftertouch,
        use_global_transposition: stored.use_global_transposition,
    };
    let ctr_instrument = CtrInstrument {
        synth_id: stored.synth_id,
        enabled: stored.enabled,
        ignore_key_transform: stored.ignore_key_transform,
        key_transform: stored.key_transform,
        ignore_velocity_transform: stored.ignore_velocity_transform,
        velocity_transform: stored.velocity_transform,
        channel_filter: stored.channel_filter,
        cc_filter: stored.cc_filter,
        ignore_pitch_bend: stored.ignore_pitch_bend,
        ignore_program_change: stored.ignore_program_change,
        ignore_channel_aftertouch: stored.ignore_channel_aftertouch,
        ignore_polyphonic_aftertouch: stored.ignore_polyphonic_aftertouch,
        use_global_transposition: stored.use_global_transposition,
        name: stored.name,
    };
    (ctr_instrument, rt_instrument)
}
