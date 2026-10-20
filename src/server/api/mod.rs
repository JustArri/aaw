use crate::{
    app_state::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

pub mod active_project;
pub mod assets_dav;
pub mod audio;
pub mod midi;
pub mod notifications;
pub mod projects;
pub mod system;

pub fn router(assets_dir: impl AsRef<std::path::Path>) -> OpenApiRouter<AppState> {
    #[derive(OpenApi)]
    #[openapi(components(schemas(notifications::Notification)))]
    struct Api;

    OpenApiRouter::with_openapi(Api::openapi())
        .routes(routes!(health))
        .routes(routes!(silence))
        .nest("/midi", midi::router())
        .nest("/audio", audio::router())
        .nest("/system", system::router())
        .nest("/active-project", active_project::router())
        .nest("/projects", projects::router())
        .merge(assets_dav::router("/assets", assets_dir))
        .routes(routes!(notifications::handler))
}

//--------------------------------------------------------------------------------------------------

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

//--------------------------------------------------------------------------------------------------

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
        .silence()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("command failed: {err}")))?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------
