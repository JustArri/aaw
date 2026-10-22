use crate::{
    app_state::AppState,
    engine::{
        EngineError,
        project::{
            self, CtrProject, CtrProjectStorage, ProjectAsset, ProjectContext, StoredProject,
        },
        user_presets::{self, StoredUserPresets},
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

mod instrument_graph;
mod synth_graph;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(new_active_project))
        .routes(routes!(active_project))
        .routes(routes!(import_active_project))
        .routes(routes!(export_active_project))
        .routes(routes!(clone_active_project))
        .routes(routes!(set_active_project_name))
        .routes(routes!(active_project_name))
        .routes(routes!(active_project_storage))
        .routes(routes!(set_user_presets))
        .routes(routes!(user_presets))
        .routes(routes!(activate_user_preset))
        .nest("/graph/synth", synth_graph::router())
        .nest("/graph/instrument", instrument_graph::router())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = NO_CONTENT, description = "New project created"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn new_active_project(
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let stream_config = app_state.stream_config().await;

    let (ctr_project, rt_project) = project::prepare(
        ProjectAsset { project: StoredProject::default(), storage: None },
        ProjectContext {
            assets_dir: &app_state.assets.read().await.dir,
            min_instruments: app_state.min_instruments,
            min_synths: app_state.min_synths,
            stream_config,
        },
    )
    .await?;

    app_state.engine_handle.replace_project(ctr_project, rt_project).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/",
    responses(
        (
            status = OK,
            body = CtrProject,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn active_project(State(app_state): State<AppState>) -> Json<CtrProject> {
    Json(app_state.engine_handle.project().await.clone())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/import",
    request_body(
        content = StoredProject,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Project loaded"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn import_active_project(
    State(app_state): State<AppState>,
    Json(project): Json<StoredProject>,
) -> Result<StatusCode, (StatusCode, String)> {
    let stream_config = app_state.stream_config().await;

    let (ctr_project, rt_project) = project::prepare(
        ProjectAsset { project, storage: None },
        ProjectContext {
            assets_dir: &app_state.assets.read().await.dir,
            min_instruments: app_state.min_instruments,
            min_synths: app_state.min_synths,
            stream_config,
        },
    )
    .await?;

    app_state.engine_handle.replace_project(ctr_project, rt_project).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/export",
    responses(
        (
            status = OK,
            body = StoredProject,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn export_active_project(State(app_state): State<AppState>) -> Json<StoredProject> {
    Json(app_state.engine_handle.save_project().await)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/storage",
    responses(
        (status = NO_CONTENT, description = "Project cloned (storage deleted)"),
    )
)]
async fn clone_active_project(State(app_state): State<AppState>) -> StatusCode {
    app_state.engine_handle.change_project_storage(None).await;
    StatusCode::NO_CONTENT
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/name",
    request_body(
        content = String,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Project name set"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_active_project_name(
    State(app_state): State<AppState>,
    Json(name): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.set_project_name(name.clone()).await;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/name",
    responses(
        (status = OK, body = String, content_type = "application/json", description = "OK"),
    )
)]
async fn active_project_name(State(app_state): State<AppState>) -> Json<String> {
    Json(app_state.engine_handle.project().await.name.clone())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/storage",
    responses(
        (
            status = OK,
            body = Option<CtrProjectStorage>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn active_project_storage(
    State(app_state): State<AppState>,
) -> Json<Option<CtrProjectStorage>> {
    Json(app_state.engine_handle.project().await.storage.clone())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/user-presets",
    request_body(
        content = StoredUserPresets,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_user_presets(
    State(app_state): State<AppState>,
    Json(presets): Json<StoredUserPresets>,
) -> Result<StatusCode, (StatusCode, String)> {
    let presets = user_presets::prepare(presets);
    app_state.engine_handle.set_user_presets(presets).await;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/user-presets",
    responses(
        (
            status = OK,
            body = StoredUserPresets,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn user_presets(
    State(app_state): State<AppState>,
) -> Result<Json<StoredUserPresets>, (StatusCode, String)> {
    let project = app_state.engine_handle.project().await;
    Ok(Json((&project.user_presets).into()))
}

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, IntoParams)]
struct UserPresetPath {
    preset_id: usize,
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/user-presets/{preset_id}/activate",
    params(UserPresetPath),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "User preset not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn activate_user_preset(
    State(app_state): State<AppState>,
    Path(path): Path<UserPresetPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    let project = app_state.engine_handle.project().await;
    let Some(preset) = project.user_presets.entries.get(path.preset_id) else {
        return Err((StatusCode::NOT_FOUND, format!("user preset #{} not found", path.preset_id)));
    };
    let preset_instruments = preset.instruments.clone();
    let instrument_ids: Vec<_> = project.instrument_graph.instruments.ids().to_vec();
    drop(project);

    for instrument_id in instrument_ids {
        let enabled = preset_instruments.contains(&instrument_id);
        app_state.engine_handle.instrument_set_enabled(instrument_id, enabled).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

impl From<EngineError> for (StatusCode, String) {
    fn from(value: EngineError) -> Self {
        (StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}
