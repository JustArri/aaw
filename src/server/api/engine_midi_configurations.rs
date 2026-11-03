use crate::{
    app_state::AppState,
    engine::midi_app_controls::{StoredMidiEngineControls, prepare_engine},
    engine_midi_configurations::{
        CtrEngineMidiConfiguration, CtrEngineMidiConfigurationStorage, EngineMidiConfigurations,
        StoredEngineMidiConfiguration,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(write_new_configuration, list_configurations))
        .routes(routes!(refresh_configurations))
        .routes(routes!(load_active_configuration))
        .routes(routes!(save_active_configuration))
        .routes(routes!(active_configuration, import_active_configuration))
        .routes(routes!(new_active_configuration))
        .routes(routes!(export_active_configuration))
        .routes(routes!(set_active_configuration_name))
        .routes(routes!(delete_configuration, read_configuration))
}

#[derive(Deserialize, IntoParams)]
struct ConfigurationPath {
    configuration_id: uuid::Uuid,
}

#[derive(Serialize, ToSchema)]
struct EngineMidiConfigurationInfo {
    id: uuid::Uuid,
    name: String,
    version: u32,
}

#[utoipa::path(
    get,
    path = "/",
    operation_id = "list_engine_midi_configurations",
    responses((status = OK, body = Vec<EngineMidiConfigurationInfo>, description = "OK"))
)]
async fn list_configurations(
    State(app_state): State<AppState>,
) -> Json<Vec<EngineMidiConfigurationInfo>> {
    let configurations = app_state.engine_midi_configurations.read().await;
    Json(
        configurations
            .list
            .iter()
            .map(|(id, preview)| EngineMidiConfigurationInfo {
                id: *id,
                name: preview.name.clone(),
                version: preview.version,
            })
            .collect(),
    )
}

#[utoipa::path(
    post,
    path = "/",
    operation_id = "write_new_engine_midi_configuration",
    request_body(content = StoredEngineMidiConfiguration, content_type = "application/json"),
    responses(
        (status = OK, body = uuid::Uuid, description = "Configuration saved"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn write_new_configuration(
    State(app_state): State<AppState>,
    Json(configuration): Json<StoredEngineMidiConfiguration>,
) -> Result<Json<uuid::Uuid>, (StatusCode, String)> {
    let id = app_state
        .engine_midi_configurations
        .write()
        .await
        .save_new(configuration)
        .await
        .map_err(internal_error)?;
    Ok(Json(id))
}

#[utoipa::path(
    post,
    path = "/refresh",
    operation_id = "refresh_engine_midi_configurations",
    responses(
        (status = NO_CONTENT, description = "Configurations refreshed"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn refresh_configurations(
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut configurations = app_state.engine_midi_configurations.write().await;
    *configurations =
        EngineMidiConfigurations::new(configurations.dir.clone()).await.map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/active",
    operation_id = "active_engine_midi_configuration",
    responses((status = OK, body = CtrEngineMidiConfiguration, description = "Active configuration"))
)]
async fn active_configuration(
    State(app_state): State<AppState>,
) -> Json<CtrEngineMidiConfiguration> {
    Json(app_state.active_engine_midi_configuration.read().await.clone())
}

#[utoipa::path(
    post,
    path = "/active/new",
    operation_id = "new_active_engine_midi_configuration",
    responses((status = NO_CONTENT, description = "Empty active configuration created"))
)]
async fn new_active_configuration(State(app_state): State<AppState>) -> StatusCode {
    replace_active_configuration(&app_state, StoredEngineMidiConfiguration::default(), None).await;
    StatusCode::NO_CONTENT
}

#[utoipa::path(
    put,
    path = "/active/name",
    operation_id = "set_active_engine_midi_configuration_name",
    request_body(content = String, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Name set"))
)]
async fn set_active_configuration_name(
    State(app_state): State<AppState>,
    Json(name): Json<String>,
) -> StatusCode {
    app_state.active_engine_midi_configuration.write().await.name = name;
    StatusCode::NO_CONTENT
}

#[utoipa::path(
    post,
    path = "/active/import",
    operation_id = "import_active_engine_midi_configuration",
    request_body(content = StoredEngineMidiConfiguration, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Configuration loaded"))
)]
async fn import_active_configuration(
    State(app_state): State<AppState>,
    Json(configuration): Json<StoredEngineMidiConfiguration>,
) -> StatusCode {
    replace_active_configuration(&app_state, configuration, None).await;
    StatusCode::NO_CONTENT
}

#[utoipa::path(
    get,
    path = "/active/export",
    operation_id = "export_active_engine_midi_configuration",
    responses((status = OK, body = StoredEngineMidiConfiguration, description = "Active configuration"))
)]
async fn export_active_configuration(
    State(app_state): State<AppState>,
) -> Json<StoredEngineMidiConfiguration> {
    Json(snapshot_active_configuration(&app_state).await)
}

#[utoipa::path(
    post,
    path = "/load-active/{configuration_id}",
    operation_id = "load_active_engine_midi_configuration",
    params(ConfigurationPath),
    responses(
        (status = NO_CONTENT, description = "Configuration loaded"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn load_active_configuration(
    State(app_state): State<AppState>,
    Path(path): Path<ConfigurationPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    let configuration = app_state
        .engine_midi_configurations
        .write()
        .await
        .load(path.configuration_id)
        .await
        .map_err(internal_error)?;
    replace_active_configuration(&app_state, configuration, Some(path.configuration_id)).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/save-active",
    operation_id = "save_active_engine_midi_configuration",
    responses(
        (status = OK, body = uuid::Uuid, description = "Active configuration saved"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn save_active_configuration(
    State(app_state): State<AppState>,
) -> Result<Json<uuid::Uuid>, (StatusCode, String)> {
    let configuration = snapshot_active_configuration(&app_state).await;
    let storage = app_state.active_engine_midi_configuration.read().await.storage.clone();
    let mut configurations = app_state.engine_midi_configurations.write().await;
    let id = if let Some(storage) = storage {
        configurations.save(storage.id, configuration).await.map_err(internal_error)?;
        storage.id
    } else {
        configurations.save_new(configuration).await.map_err(internal_error)?
    };
    drop(configurations);

    app_state.active_engine_midi_configuration.write().await.storage =
        Some(CtrEngineMidiConfigurationStorage { id });
    app_state.save_last_engine_midi_configuration(Some(id)).await;
    Ok(Json(id))
}

#[utoipa::path(
    delete,
    path = "/{configuration_id}",
    operation_id = "delete_engine_midi_configuration",
    params(ConfigurationPath),
    responses(
        (status = NO_CONTENT, description = "Configuration deleted"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn delete_configuration(
    State(app_state): State<AppState>,
    Path(path): Path<ConfigurationPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    let is_active = app_state
        .active_engine_midi_configuration
        .read()
        .await
        .storage
        .as_ref()
        .is_some_and(|storage| storage.id == path.configuration_id);
    app_state
        .engine_midi_configurations
        .write()
        .await
        .delete(path.configuration_id)
        .await
        .map_err(internal_error)?;
    if is_active {
        app_state.active_engine_midi_configuration.write().await.storage = None;
        app_state.save_last_engine_midi_configuration(None).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/{configuration_id}",
    operation_id = "read_engine_midi_configuration",
    params(ConfigurationPath),
    responses(
        (status = OK, body = String, content_type = "application/octet-stream", description = "Original configuration file"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn read_configuration(
    State(app_state): State<AppState>,
    Path(path): Path<ConfigurationPath>,
) -> Result<([(header::HeaderName, String); 2], Vec<u8>), (StatusCode, String)> {
    let content = app_state
        .engine_midi_configurations
        .read()
        .await
        .read_file(path.configuration_id)
        .await
        .map_err(internal_error)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".into()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}.json\"", path.configuration_id),
            ),
        ],
        content,
    ))
}

async fn snapshot_active_configuration(app_state: &AppState) -> StoredEngineMidiConfiguration {
    let name = app_state.active_engine_midi_configuration.read().await.name.clone();
    let midi_app_controls: StoredMidiEngineControls =
        app_state.engine_handle.midi_engine_controls().await.into();
    let input_auto_connections =
        app_state.midi_runtime.lock().await.input_auto_connections().clone();
    StoredEngineMidiConfiguration { name, midi_app_controls, input_auto_connections }
}

async fn replace_active_configuration(
    app_state: &AppState,
    configuration: StoredEngineMidiConfiguration,
    id: Option<uuid::Uuid>,
) {
    let StoredEngineMidiConfiguration { name, midi_app_controls, input_auto_connections } =
        configuration;
    let (controls, _) = prepare_engine(midi_app_controls);
    app_state.engine_handle.set_midi_engine_controls(controls).await;
    app_state.midi_runtime.lock().await.replace_input_auto_connections(input_auto_connections);
    *app_state.active_engine_midi_configuration.write().await = CtrEngineMidiConfiguration {
        name,
        storage: id.map(|id| CtrEngineMidiConfigurationStorage { id }),
    };
    app_state.save_last_engine_midi_configuration(id).await;
}

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
