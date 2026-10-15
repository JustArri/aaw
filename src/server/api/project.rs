use crate::{engine, server::AppState};
use axum::{Json, extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(project)).routes(routes!(set_project))
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = OK, body = engine::stored::Project, description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn project(State(app_state): State<AppState>) -> Json<engine::stored::Project> {
    Json(app_state.engine_handle.save_project().await)
}

#[utoipa::path(
    post,
    path = "/",
    request_body = engine::stored::Project,
    responses(
        (status = NO_CONTENT, description = "Project loaded"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn set_project(
    State(app_state): State<AppState>,
    Json(project): Json<engine::stored::Project>,
) -> Result<StatusCode, (StatusCode, String)> {
    let (ctr_project, rt_project) =
        engine::load(project, app_state.min_synth_slots, app_state.min_synths_per_graph)
            .await
            .map_err(|err| {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to load project: {err}"))
            })?;

    app_state.engine_handle.replace_project(ctr_project, rt_project).await.map_err(|err| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to replace project: {err}"))
    })?;

    Ok(StatusCode::NO_CONTENT)
}
