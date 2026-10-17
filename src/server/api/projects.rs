use crate::{
    app_state::AppState,
    engine::{self, stored::Project},
    projects::Projects,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(save))
        .routes(routes!(list))
        .routes(routes!(refresh))
        .routes(routes!(load_active_project))
        .routes(routes!(save_active_project))
        .routes(routes!(delete))
        .routes(routes!(load))
}

#[derive(Deserialize, IntoParams)]
struct ProjectPath {
    project_id: uuid::Uuid,
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/",
    request_body(
        content = Project,
        content_type = "application/json"
    ),
    responses(
        (
            status = OK,
            body = uuid::Uuid,
            content_type = "application/json",
            description = "Project saved"
        ),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn save(
    State(app_state): State<AppState>,
    Json(project): Json<Project>,
) -> Result<Json<uuid::Uuid>, (StatusCode, String)> {
    let mut projects = app_state.projects.write().await;
    let id = projects
        .save_new(project)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(id))
}

//--------------------------------------------------------------------------------------------------

#[derive(Serialize, ToSchema)]
struct ProjectInfo {
    id: uuid::Uuid,
    name: String,
    version: u32,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (
            status = OK,
            body = Vec<ProjectInfo>,
            content_type = "application/json",
            description = "OK"
        ),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn list(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<ProjectInfo>>, (StatusCode, String)> {
    let projects = app_state.projects.read().await;
    let values = projects
        .list
        .iter()
        .map(|(id, preview)| ProjectInfo {
            id: *id,
            name: preview.name.clone(),
            version: preview.version,
        })
        .collect();
    Ok(Json(values))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/refresh",
    responses(
        (status = NO_CONTENT, description = "Projects refreshed"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn refresh(State(app_state): State<AppState>) -> Result<StatusCode, (StatusCode, String)> {
    let mut projects = app_state.projects.write().await;
    *projects = Projects::new(projects.dir.clone())
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/load-active-project/{project_id}",
    params(ProjectPath),
    responses(
        (status = NO_CONTENT, description = "Active project loaded"),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn load_active_project(
    State(app_state): State<AppState>,
    Path(path): Path<ProjectPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut projects = app_state.projects.write().await;
    let project = projects
        .load(path.project_id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let sample_rate = app_state.sample_rate().await;
    let (ctr_project, rt_project) = engine::load(
        app_state.assets_dir.as_ref(),
        project,
        app_state.min_instrument_slots,
        app_state.min_synths_per_graph,
        sample_rate,
    )
    .await?;
    app_state.engine_handle.replace_project(ctr_project, rt_project).await?;
    *app_state.active_project_id.write().await = Some(path.project_id);
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/save-active-project",
    responses(
        (
            status = OK,
            body = uuid::Uuid,
            content_type = "application/json",
            description = "Active project saved"
        ),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn save_active_project(
    State(app_state): State<AppState>,
) -> Result<Json<uuid::Uuid>, (StatusCode, String)> {
    let mut projects = app_state.projects.write().await;
    let id = if let Some(id) = *app_state.active_project_id.read().await {
        projects
            .save(id, app_state.engine_handle.save_project().await)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        id
    } else {
        projects
            .save_new(app_state.engine_handle.save_project().await)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    };
    *app_state.active_project_id.write().await = Some(id);
    Ok(Json(id))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/{project_id}",
    params(ProjectPath),
    responses(
        (status = NO_CONTENT, description = "Project deleted"),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn delete(
    State(app_state): State<AppState>,
    Path(path): Path<ProjectPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut projects = app_state.projects.write().await;
    projects
        .delete(path.project_id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/{project_id}",
    params(ProjectPath),
    responses(
        (
            status = OK,
            body = Project,
            content_type = "application/json",
            description = "Project deleted"
        ),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn load(
    State(app_state): State<AppState>,
    Path(path): Path<ProjectPath>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let mut projects = app_state.projects.write().await;
    let project = projects
        .load(path.project_id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(project))
}

//--------------------------------------------------------------------------------------------------
