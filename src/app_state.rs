use crate::{
    DEFAULT_CHANNEL_COUNT, DEFAULT_MAX_BLOCK_SIZE, DEFAULT_SAMPLE_RATE,
    assets::Assets,
    engine::{Engine, EngineHandle, project::StoredProject},
    midi::{MidiEvent, MidiRuntime},
    projects::Projects,
    rt_channels::NotifiedRtSharedProducer,
    server::api::audio::{AudioOutputStream, AudioOutputStreamConfig},
};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, RwLock, broadcast};

#[derive(Clone)]
pub struct AppState {
    pub midi_tx: broadcast::Sender<Arc<MidiEvent>>,
    pub midi_runtime: Arc<Mutex<MidiRuntime>>,
    pub midi_event_senders: Arc<[NotifiedRtSharedProducer<MidiEvent>]>,
    pub engine: Arc<Mutex<Engine>>,
    pub engine_handle: EngineHandle,
    pub output_stream: Arc<Mutex<Option<AudioOutputStream>>>,
    pub min_instruments: usize,
    pub min_synths: usize,
    pub stream_error: Arc<AtomicBool>,
    pub projects: Arc<RwLock<Projects>>,
    pub assets: Arc<RwLock<Assets>>,
}

impl AppState {
    pub fn stream_error(&self) -> bool {
        self.stream_error.load(Ordering::Relaxed)
    }

    pub async fn stream_config(&self) -> AudioOutputStreamConfig {
        self.output_stream.lock().await.as_ref().map(|s| (&s.info).into()).unwrap_or(
            AudioOutputStreamConfig {
                channel_count: DEFAULT_CHANNEL_COUNT,
                sample_rate: DEFAULT_SAMPLE_RATE,
                max_block_size: DEFAULT_MAX_BLOCK_SIZE,
            },
        )
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize)]
pub enum LastProject {
    Cached(StoredProject),
    Saved(uuid::Uuid),
}
