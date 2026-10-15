use crate::{
    engine::{ctr::EngineHandle, rt::Engine},
    midi::{MidiEvent, MidiRuntime},
    rt_channels::NotifiedRtSharedProducer,
    server::api::audio::AudioOutputStream,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, broadcast};
use tower_http::compression::CompressionLayer;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

pub mod api;

#[derive(Clone)]
pub struct AppState {
    pub midi_tx: broadcast::Sender<Arc<MidiEvent>>,
    pub midi_runtime: Arc<Mutex<MidiRuntime>>,
    pub midi_event_senders: Arc<[NotifiedRtSharedProducer<MidiEvent>]>,
    pub engine: Arc<Mutex<Engine>>,
    pub engine_handle: EngineHandle,
    pub output_stream: Arc<Mutex<Option<(cpal::Stream, AudioOutputStream)>>>,
    pub min_synth_slots: usize,
    pub min_synths_per_graph: usize,
    pub stream_error: Arc<AtomicBool>,
}

impl AppState {
    pub fn stream_error(&self) -> bool {
        self.stream_error.load(Ordering::Relaxed)
    }
}

pub fn router() -> axum::Router<AppState> {
    let root_router = api_router();
    let (root_router, openapi) = root_router.split_for_parts();
    let openapi_json = openapi.clone();
    let root_router = root_router.route(
        "/api/openapi.json",
        axum::routing::get(move || async move { axum::Json(openapi_json) }),
    );
    let root_router = root_router.merge(Scalar::with_url("/api/docs", openapi));
    let compression_layer = CompressionLayer::new();
    root_router.layer(compression_layer)
}

pub fn api_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().nest("/api", api::router())
}

enum WsHandlerAction {
    Continue,
    Exit,
}
