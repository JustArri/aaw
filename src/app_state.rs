use crate::{
    DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE,
    assets::Assets,
    audio::metrics::AudioMetrics,
    engine::{Engine, EngineHandle},
    files,
    midi::{MidiEvent, MidiRuntime},
    projects::Projects,
    rt_channels::NotifiedRtSharedProducer,
    server::api::audio::{AudioOutputStream, AudioOutputStreamConfig},
    synths::{
        DUMMY_DEFAULT_VOICE_COUNT, FLUID_DEFAULT_VOICE_COUNT, SFIZZ_DEFAULT_PRELOAD_SIZE,
        SFIZZ_DEFAULT_VOICE_COUNT,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex, RwLock, broadcast};
use utoipa::ToSchema;

#[derive(Clone)]
pub struct AppState {
    pub midi_tx: broadcast::Sender<Arc<MidiEvent>>,
    pub midi_runtime: Arc<Mutex<MidiRuntime>>,
    pub midi_event_senders: Arc<[NotifiedRtSharedProducer<MidiEvent>]>,
    pub engine: Arc<Mutex<Engine>>,
    pub engine_handle: EngineHandle,
    pub output_stream: Arc<Mutex<Option<AudioOutputStream>>>,
    pub min_instruments: usize,
    pub min_controllers: usize,
    pub stream_error: Arc<AtomicBool>,
    pub audio_metrics: Arc<AudioMetrics>,
    pub aaw_root_dir: Arc<Path>,
    pub projects: Arc<RwLock<Projects>>,
    pub assets: Arc<RwLock<Assets>>,
    pub recordings_dir: Arc<Path>,
    pub preferences: Arc<RwLock<Preferences>>,
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
                max_block_size: MAX_BLOCK_SIZE,
            },
        )
    }

    pub async fn save_last_project(&self, id: Option<uuid::Uuid>) {
        files::save(&self.aaw_root_dir, crate::CFG_LAST_PROJECT, &LastProject { id }).await;
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize)]
pub struct LastProject {
    pub id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Preferences {
    pub dummy_max_voices: usize,
    pub fluid_max_voices: usize,
    pub sfizz_max_voices: usize,
    pub sfizz_preload_size: u32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            dummy_max_voices: DUMMY_DEFAULT_VOICE_COUNT,
            fluid_max_voices: FLUID_DEFAULT_VOICE_COUNT,
            sfizz_max_voices: SFIZZ_DEFAULT_VOICE_COUNT,
            sfizz_preload_size: SFIZZ_DEFAULT_PRELOAD_SIZE,
        }
    }
}
