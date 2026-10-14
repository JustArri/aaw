use crate::{server::AppState, system};
use axum::http::StatusCode;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(shutdown)).routes(routes!(reboot))
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
    action_status(system::poweroff().await, "shut down")
}

#[utoipa::path(
    post,
    path = "/reboot",
    responses(
        (status = ACCEPTED, description = "Reboot initiated"),
        (status = FORBIDDEN, description = "Permission denied"),
        (status = INTERNAL_SERVER_ERROR, description = "Reboot failed"),
    )
)]
async fn reboot() -> StatusCode {
    action_status(system::reboot().await, "reboot")
}

fn action_status(result: zbus::Result<()>, action: &str) -> StatusCode {
    match result {
        Ok(()) => StatusCode::ACCEPTED,
        Err(err) => {
            tracing::error!(%err, %action, "System action failed");

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
