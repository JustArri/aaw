use crate::{app_state::AppState, engine::metrics::EngineMetricsSnapshot};
use axum::{Json, extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(engine_metrics, reset_engine_metrics))
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses((status = OK, body = EngineMetricsSnapshot, description = "Real-time engine performance metrics"))
)]
async fn engine_metrics(State(app_state): State<AppState>) -> Json<EngineMetricsSnapshot> {
    Json(app_state.engine_metrics.snapshot())
}

#[utoipa::path(
    post,
    path = "/metrics/reset",
    responses((status = NO_CONTENT, description = "Engine metrics reset"))
)]
async fn reset_engine_metrics(State(app_state): State<AppState>) -> StatusCode {
    app_state.engine_metrics.reset();
    StatusCode::NO_CONTENT
}
