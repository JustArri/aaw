use crate::{
    app_state::{AppState, Preferences},
    system,
};
use axum::{Json, extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(system_shutdown))
        .routes(routes!(system_reboot))
        .routes(routes!(set_preferences))
        .routes(routes!(preferences))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/shutdown",
    responses(
        (status = ACCEPTED, description = "Shutdown initiated"),
        (status = FORBIDDEN, description = "Permission denied"),
        (status = INTERNAL_SERVER_ERROR, description = "Shutdown failed"),
    )
)]
async fn system_shutdown() -> StatusCode {
    action_status(system::poweroff().await, "shut down")
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/reboot",
    responses(
        (status = ACCEPTED, description = "Reboot initiated"),
        (status = FORBIDDEN, description = "Permission denied"),
        (status = INTERNAL_SERVER_ERROR, description = "Reboot failed"),
    )
)]
async fn system_reboot() -> StatusCode {
    action_status(system::reboot().await, "reboot")
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/preferences",
    request_body(
        content = Preferences,
        content_type = "application/json",
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = BAD_REQUEST, description = "Invalid request"),
    )
)]
async fn set_preferences(
    State(app_state): State<AppState>,
    Json(preferences): Json<Preferences>,
) -> Result<StatusCode, (StatusCode, String)> {
    *app_state.preferences.write().await = preferences;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/preferences",
    responses(
        (
            status = OK,
            body = Preferences,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn preferences(State(app_state): State<AppState>) -> Json<Preferences> {
    Json(app_state.preferences.read().await.clone())
}

//--------------------------------------------------------------------------------------------------

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
