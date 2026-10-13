use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::server::AppState;

pub mod audio;
pub mod midi;
pub mod notifications;
pub mod system;

pub fn router() -> OpenApiRouter<AppState> {
    #[derive(OpenApi)]
    #[openapi(components(schemas(notifications::Notification)))]
    struct Api;

    OpenApiRouter::with_openapi(Api::openapi())
        .routes(routes!(health))
        .nest("/midi", midi::router())
        .nest("/audio", audio::router())
        .nest("/system", system::router())
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
