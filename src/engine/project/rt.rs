use crate::engine::{instrument_graph::RtInstrumentGraph, synth_graph::RtSynthGraph};

pub struct RtProject {
    pub instrument_graph: RtInstrumentGraph,
    pub synth_graph: RtSynthGraph,
}
