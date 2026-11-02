use crate::engine::{drum_machine::MusicalTransport, instrument_graph::RtInstrumentGraph};

pub struct RtProject {
    pub instrument_graph: RtInstrumentGraph,
    pub transport: RtTransport,
}

pub struct RtTransport {
    pub config: MusicalTransport,
    pub sample_position: u64,
    pub segment_start_frame: u64,
    pub segment_start_tick: f64,
}
