use crate::{
    app_state::AppState,
    engine::{self, CtrProjectStorage, StoredProject},
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
        .routes(routes!(write_new_project))
        .routes(routes!(list_projects))
        .routes(routes!(refresh_projects))
        .routes(routes!(load_active_project))
        .routes(routes!(save_active_project))
        .routes(routes!(delete_project))
        .routes(routes!(read_project))
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
        content = StoredProject,
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
async fn write_new_project(
    State(app_state): State<AppState>,
    Json(project): Json<StoredProject>,
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
async fn list_projects(
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
async fn refresh_projects(
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
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
    let channel_count = app_state.channel_count().await;

    let revision = project.revision;
    let id = path.project_id;

    let (ctr_project, rt_project) = engine::load(
        &app_state.assets.read().await.dir,
        project,
        Some(CtrProjectStorage { id, revision }),
        app_state.min_instrument_slots,
        app_state.min_synths_per_graph,
        channel_count,
        sample_rate,
    )
    .await?;

    app_state.engine_handle.replace_project(ctr_project, rt_project).await?;

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
    let (stored_project, project_storage, revision) = {
        let project = app_state.engine_handle.project().await;
        (StoredProject::from(&*project), project.storage.clone(), project.revision)
    };

    let mut projects = app_state.projects.write().await;
    let id = if let Some(storage) = project_storage {
        projects
            .save(storage.id, stored_project)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        storage.id
    } else {
        projects
            .save_new(stored_project)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    };

    app_state.engine_handle.change_project_storage(Some(CtrProjectStorage { id, revision })).await;
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
async fn delete_project(
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
            body = StoredProject,
            content_type = "application/json",
            description = "Project deleted"
        ),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn read_project(
    State(app_state): State<AppState>,
    Path(path): Path<ProjectPath>,
) -> Result<Json<StoredProject>, (StatusCode, String)> {
    let mut projects = app_state.projects.write().await;
    let project = projects
        .load(path.project_id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(project))
}

//--------------------------------------------------------------------------------------------------
