use crate::{
    engine::{self, ctr::EngineHandle, graph::DEFAULT_SAMPLE_RATE, rt::Engine},
    midi::{MidiEvent, MidiRuntime},
    projects::Projects,
    rt_channels::NotifiedRtSharedProducer,
    server::api::{
        audio::AudioOutputStream,
        notifications::{ProjectChange, ProjectChangeKind},
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
    pub assets_dir: Arc<Path>,
    pub active_project_id: Arc<RwLock<Option<uuid::Uuid>>>,
    pub project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
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

    pub async fn broadcast_project_change(&self, kind: ProjectChangeKind) {
        let change =
            ProjectChange { revision: self.engine_handle.project.read().await.revision, kind };
        let _ = self.project_change_tx.send(Arc::new(change));
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize)]
pub enum LastProject {
    Cached(engine::stored::Project),
    Saved(uuid::Uuid),
}
