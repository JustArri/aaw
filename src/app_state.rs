use crate::{
    assets::Assets,
    engine::{
        self, Engine, EngineHandle,
        graph::{DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE},
    },
    midi::{MidiEvent, MidiRuntime},
    projects::Projects,
    rt_channels::NotifiedRtSharedProducer,
    server::api::audio::AudioOutputStream,
};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, RwLock, broadcast};

#[derive(Clone)]
pub struct AppState {
    pub midi_tx: broadcast::Sender<Arc<MidiEvent>>,
    pub midi_runtime: Arc<Mutex<MidiRuntime>>,
    pub midi_event_senders: Arc<[NotifiedRtSharedProducer<MidiEvent>]>,
    pub engine: Arc<Mutex<Engine>>,
    pub engine_handle: EngineHandle,
    pub output_stream: Arc<Mutex<Option<(cpal::Stream, AudioOutputStream)>>>,
    pub min_instrument_slots: usize,
    pub min_synths_per_graph: usize,
    pub stream_error: Arc<AtomicBool>,
    pub projects: Arc<RwLock<Projects>>,
    pub assets: Arc<RwLock<Assets>>,
}

impl AppState {
    pub fn stream_error(&self) -> bool {
        self.stream_error.load(Ordering::Relaxed)
    }

    pub async fn sample_rate(&self) -> u32 {
        self.output_stream
            .lock()
            .await
            .as_ref()
            .map(|s| s.1.sample_rate)
            .unwrap_or(DEFAULT_SAMPLE_RATE)
    }

    pub async fn channel_count(&self) -> usize {
        self.output_stream
            .lock()
            .await
            .as_ref()
            .map(|s| s.1.channel_count as usize)
            .unwrap_or(DEFAULT_CHANNEL_COUNT)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize)]
pub enum LastProject {
    Cached(engine::StoredProject),
    Saved(uuid::Uuid),
}
