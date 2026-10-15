use crate::{engine::rt::Command, server::AppState};
use axum::{extract::State, http::StatusCode};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

pub mod audio;
pub mod midi;
pub mod notifications;
pub mod project;
pub mod system;

pub fn router() -> OpenApiRouter<AppState> {
    #[derive(OpenApi)]
    #[openapi(components(schemas(notifications::Notification)))]
    struct Api;

    OpenApiRouter::with_openapi(Api::openapi())
        .routes(routes!(health))
        .routes(routes!(silence))
        .nest("/midi", midi::router())
        .nest("/audio", audio::router())
        .nest("/system", system::router())
        .nest("/project", project::router())
        .routes(routes!(notifications::handler))
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = OK, body = &'static str, description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    post,
    path = "/silence",
    responses(
        (status = NO_CONTENT, description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn silence(State(app_state): State<AppState>) -> Result<StatusCode, (StatusCode, String)> {
    app_state
        .engine_handle
        .command(Command::Silence)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("command failed: {err}")))?;
    Ok(StatusCode::NO_CONTENT)
}
