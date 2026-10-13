use crate::{
    engine,
    midi::{MidiEvent, MidiRuntime},
    server::api::audio::AudioOutputStream,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::compression::CompressionLayer;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

pub mod api;

#[derive(Clone)]
pub struct AppState {
    pub midi_tx: tokio::sync::broadcast::Sender<Arc<MidiEvent>>,
    pub midi_runtime: Arc<Mutex<MidiRuntime>>,
    pub engine: Arc<std::sync::Mutex<engine::rt::Engine>>,
    pub output_stream: Arc<Mutex<Option<(cpal::Stream, AudioOutputStream)>>>,
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
