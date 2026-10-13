use crate::{server::AppState, system};
use axum::http::StatusCode;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(shutdown))
}

#[utoipa::path(
    post,
    path = "/shutdown",
    responses(
        (status = ACCEPTED, description = "Shutdown initiated"),
        (status = FORBIDDEN, description = "Permission denied"),
        (status = INTERNAL_SERVER_ERROR, description = "Shutdown failed"),
    )
)]
async fn shutdown() -> StatusCode {
    match system::poweroff().await {
        Ok(()) => StatusCode::ACCEPTED,
        Err(err) => {
            tracing::error!(%err, "Failed to shut down system");

            match &err {
                zbus::Error::MethodError(name, _, _)
                    if name.as_str() == "org.freedesktop.DBus.Error.AccessDenied" =>
                {
                    StatusCode::FORBIDDEN
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }
}
