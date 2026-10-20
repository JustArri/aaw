use crate::{
    app_state::AppState,
    engine::{
        self, CtrProject, EngineError, StoredProject, dummy_graph, fluid_graph,
        graph::LocalId,
        instrument_graph::{self, SynthId},
        sfizz_graph,
    },
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
    synths::{self, SfzInfo, SoundFontInfo, SoundFontProgram},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(active_project))
        .routes(routes!(import_active_project))
        .routes(routes!(export_active_project))
        .routes(routes!(active_project_id))
        .routes(routes!(delete_active_project_id))
        .routes(routes!(set_active_project_name))
        .routes(routes!(active_project_name))
        .routes(routes!(dummy_synths))
        .routes(routes!(dummy_push))
        .routes(routes!(dummy_remove))
        .routes(routes!(dummy_set_name))
        .routes(routes!(dummy_name))
        .routes(routes!(fluid_synths))
        .routes(routes!(fluid_push))
        .routes(routes!(fluid_replace))
        .routes(routes!(fluid_remove))
        .routes(routes!(fluid_info))
        .routes(routes!(fluid_set_program))
        .routes(routes!(fluid_program))
        .routes(routes!(fluid_set_name))
        .routes(routes!(fluid_name))
        .routes(routes!(sfizz_synths))
        .routes(routes!(sfizz_push))
        .routes(routes!(sfizz_replace))
        .routes(routes!(sfizz_remove))
        .routes(routes!(sfizz_info))
        .routes(routes!(sfizz_set_name))
        .routes(routes!(sfizz_name))
        .routes(routes!(set_global_transposition))
        .routes(routes!(global_transposition))
        .routes(routes!(instrument_slots))
        .routes(routes!(instrument_slot_append_new))
        .routes(routes!(instrument_slot))
        .routes(routes!(instrument_slot_remove))
        .routes(routes!(set_synth))
        .routes(routes!(synth))
        .routes(routes!(set_enabled))
        .routes(routes!(enabled))
        .routes(routes!(set_key_transform))
        .routes(routes!(key_transform))
        .routes(routes!(set_velocity_transform))
        .routes(routes!(velocity_transform))
        .routes(routes!(set_channel_filter))
        .routes(routes!(channel_filter))
        .routes(routes!(set_cc_filter))
        .routes(routes!(cc_filter))
        .routes(routes!(set_use_global_transposition))
        .routes(routes!(use_global_transposition))
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
    Json(app_state.engine_handle.project.read().await.clone())
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
    let sample_rate = app_state.sample_rate().await;
    let channel_count = app_state.channel_count().await;

    let (ctr_project, rt_project) = engine::load(
        &app_state.assets_dir,
        project,
        app_state.min_instrument_slots,
        app_state.min_synths_per_graph,
        channel_count,
        sample_rate,
    )
    .await?;
    *app_state.active_project_id.write().await = None;
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
    get,
    path = "/id",
    responses(
        (
            status = OK,
            body = Option<uuid::Uuid>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn active_project_id(State(app_state): State<AppState>) -> Json<Option<uuid::Uuid>> {
    Json(*app_state.active_project_id.read().await)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/id",
    responses(
        (status = NO_CONTENT, description = "Project ID deleted"),
    )
)]
async fn delete_active_project_id(State(app_state): State<AppState>) -> StatusCode {
    *app_state.active_project_id.write().await = None;
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
    Json(app_state.engine_handle.project.read().await.name.clone())
}

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)] //TODO: remove all parameter_in
struct SynthPath {
    #[param(value_type = usize)]
    synth_id: LocalId,
}

#[derive(Deserialize, ToSchema)]
pub struct SynthPushRequest {
    #[schema(value_type = String)]
    path: PathBuf,
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/dummy/synths",
    responses(
        (
            status = OK,
            body = HashMap<LocalId, dummy_graph::CtrSynth>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn dummy_synths(
    State(app_state): State<AppState>,
) -> Json<HashMap<LocalId, dummy_graph::CtrSynth>> {
    Json(app_state.engine_handle.project.read().await.dummy_graph.synths.clone().into())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/graph/dummy/synths",
    responses(
        (status = OK, body = LocalId, content_type = "application/json", description = "Created"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn dummy_push(
    State(app_state): State<AppState>,
) -> Result<Json<LocalId>, (StatusCode, String)> {
    let sample_rate = app_state.sample_rate().await;
    let (ctr_synth, rt_synth) = dummy_graph::prepare_synth("Dummy Synth".into(), sample_rate);
    Ok(Json(app_state.engine_handle.dummy_push(ctr_synth, rt_synth).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/graph/dummy/synths/{synth_id}",
    params(SynthPath),
    responses(
        (status = NO_CONTENT, description = "Removed"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn dummy_remove(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_dummy_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.dummy_remove(path.synth_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/dummy/synths/{synth_id}/name",
    params(SynthPath),
    request_body(
        content = String,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn dummy_set_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(name): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_dummy_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.dummy_set_name(path.synth_id, name).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/dummy/synths/{synth_id}/name",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = String,
            content_type = "application/json",
            description = "Value set"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn dummy_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<String>, (StatusCode, String)> {
    Ok(Json(with_dummy_synth(&app_state, path.synth_id, |s| s.name.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/fluid/synths",
    responses(
        (
            status = OK,
            body = HashMap<LocalId, fluid_graph::CtrSynth>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn fluid_synths(
    State(app_state): State<AppState>,
) -> Json<HashMap<LocalId, fluid_graph::CtrSynth>> {
    Json(app_state.engine_handle.project.read().await.fluid_graph.synths.clone().into())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/graph/fluid/synths",
    request_body(
        content = SynthPushRequest,
        content_type = "application/json"
    ),
    responses(
        (status = OK, body = LocalId, content_type = "application/json", description = "Created"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn fluid_push(
    State(app_state): State<AppState>,
    Json(req): Json<SynthPushRequest>,
) -> Result<Json<LocalId>, (StatusCode, String)> {
    let sample_rate = app_state.sample_rate().await;
    let name = req
        .path
        .file_stem()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or("Fluid Synth".into());
    let (ctr_synth, rt_synth) =
        fluid_graph::prepare_synth(&app_state.assets_dir, name, req.path, sample_rate).await?;
    Ok(Json(app_state.engine_handle.fluid_push(ctr_synth, rt_synth).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/fluid/synths/{synth_id}",
    params(SynthPath),
    request_body(
        content = SynthPushRequest,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Created"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn fluid_replace(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(req): Json<SynthPushRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_fluid_synth(&app_state, path.synth_id, |_| ()).await?;
    let sample_rate = app_state.sample_rate().await;
    let name = req
        .path
        .file_stem()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or("Fluid Synth".into());
    let (ctr_synth, rt_synth) =
        fluid_graph::prepare_synth(&app_state.assets_dir, name, req.path, sample_rate).await?;
    app_state.engine_handle.fluid_replace(path.synth_id, ctr_synth, rt_synth).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/graph/fluid/synths/{synth_id}",
    params(SynthPath),
    responses(
        (status = NO_CONTENT, description = "Removed"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn fluid_remove(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_fluid_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.fluid_remove(path.synth_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/fluid/synths/{synth_id}/info",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = SoundFontInfo,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn fluid_info(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<SoundFontInfo>, (StatusCode, String)> {
    Ok(Json(with_fluid_synth(&app_state, path.synth_id, |s| s.info.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/fluid/synths/{synth_id}/program",
    params(SynthPath),
    request_body(
        content = SoundFontProgram,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn fluid_set_program(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(program): Json<SoundFontProgram>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_fluid_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.fluid_select_program(path.synth_id, program).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/fluid/synths/{synth_id}/program",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = SoundFontProgram,
            content_type = "application/json",
            description = "Value set"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn fluid_program(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<SoundFontProgram>, (StatusCode, String)> {
    Ok(Json(with_fluid_synth(&app_state, path.synth_id, |s| s.program).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/fluid/synths/{synth_id}/name",
    params(SynthPath),
    request_body(
        content = String,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn fluid_set_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(name): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_fluid_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.fluid_set_name(path.synth_id, name).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/fluid/synths/{synth_id}/name",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = String,
            content_type = "application/json",
            description = "Value set"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn fluid_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<String>, (StatusCode, String)> {
    Ok(Json(with_fluid_synth(&app_state, path.synth_id, |s| s.name.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/sfizz/synths",
    responses(
        (
            status = OK,
            body = HashMap<LocalId, sfizz_graph::CtrSynth>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn sfizz_synths(
    State(app_state): State<AppState>,
) -> Json<HashMap<LocalId, sfizz_graph::CtrSynth>> {
    Json(app_state.engine_handle.project.read().await.sfizz_graph.synths.clone().into())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/graph/sfizz/synths",
    request_body(
        content = SynthPushRequest,
        content_type = "application/json"
    ),
    responses(
        (status = OK, body = LocalId, content_type = "application/json", description = "Created"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn sfizz_push(
    State(app_state): State<AppState>,
    Json(req): Json<SynthPushRequest>,
) -> Result<Json<LocalId>, (StatusCode, String)> {
    let sample_rate = app_state.sample_rate().await;
    let name = req
        .path
        .file_stem()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or("Sfizz Synth".into());
    let (ctr_synth, rt_synth) =
        sfizz_graph::prepare_synth(&app_state.assets_dir, name, req.path, sample_rate).await?;
    Ok(Json(app_state.engine_handle.sfizz_push(ctr_synth, rt_synth).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/sfizz/synths/{synth_id}",
    params(SynthPath),
    request_body(
        content = SynthPushRequest,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Created"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn sfizz_replace(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(req): Json<SynthPushRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_sfizz_synth(&app_state, path.synth_id, |_| ()).await?;
    let sample_rate = app_state.sample_rate().await;
    let name = req
        .path
        .file_stem()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or("Sfizz Synth".into());
    let (ctr_synth, rt_synth) =
        sfizz_graph::prepare_synth(&app_state.assets_dir, name, req.path, sample_rate).await?;
    app_state.engine_handle.sfizz_replace(path.synth_id, ctr_synth, rt_synth).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/graph/sfizz/synths/{synth_id}",
    params(SynthPath),
    responses(
        (status = NO_CONTENT, description = "Removed"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn sfizz_remove(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_sfizz_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.sfizz_remove(path.synth_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/sfizz/synths/{synth_id}/info",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = SfzInfo,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn sfizz_info(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<SfzInfo>, (StatusCode, String)> {
    Ok(Json(with_sfizz_synth(&app_state, path.synth_id, |s| s.info.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/sfizz/synths/{synth_id}/name",
    params(SynthPath),
    request_body(
        content = String,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn sfizz_set_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(name): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_sfizz_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.sfizz_set_name(path.synth_id, name).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/sfizz/synths/{synth_id}/name",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = String,
            content_type = "application/json",
            description = "Value set"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn sfizz_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<String>, (StatusCode, String)> {
    Ok(Json(with_sfizz_synth(&app_state, path.synth_id, |s| s.name.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/global-transposition",
    request_body(
        content = i8,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_global_transposition(
    State(app_state): State<AppState>,
    Json(transposition): Json<i8>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.set_global_transposition(transposition).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/global-transposition",
    responses(
        (status = OK, body = i8, content_type = "application/json", description = "OK"),
    )
)]
async fn global_transposition(State(app_state): State<AppState>) -> Json<i8> {
    Json(app_state.engine_handle.project.read().await.instrument_graph.global_transposition)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots",
    responses(
        (
            status = OK,
            body = HashMap<LocalId, instrument_graph::CtrSlot>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn instrument_slots(
    State(app_state): State<AppState>,
) -> Json<HashMap<LocalId, instrument_graph::CtrSlot>> {
    Json(app_state.engine_handle.project.read().await.instrument_graph.slots.clone().into())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/graph/instrument/slots",
    responses(
        (status = OK, body = LocalId, content_type = "application/json", description = "Created"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_slot_append_new(
    State(app_state): State<AppState>,
) -> Result<Json<LocalId>, (StatusCode, String)> {
    Ok(Json(app_state.engine_handle.instrument_slot_append_new().await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}",
    params(InstrumentSlotPath),
    responses(
        (
            status = OK,
            body = instrument_graph::CtrSlot,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn instrument_slot(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<instrument_graph::CtrSlot>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, Clone::clone).await?))
}

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
struct InstrumentSlotPath {
    #[param(value_type = usize)]
    slot_id: LocalId,
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/graph/instrument/slots/{slot_id}",
    params(InstrumentSlotPath),
    responses(
        (status = NO_CONTENT, description = "Removed"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_slot_remove(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.instrument_slot_remove(path.slot_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/synth",
    params(InstrumentSlotPath),
    request_body(
        content = Option<SynthId>,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_synth(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(synth): Json<Option<SynthId>>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.set_synth(path.slot_id, synth).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/synth",
    params(InstrumentSlotPath),
    responses(
        (
            status = OK,
            body = Option<SynthId>,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn synth(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<Option<SynthId>>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, |s| s.synth_id).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/enabled",
    params(InstrumentSlotPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_enabled(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.set_enabled(path.slot_id, enabled).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/enabled",
    params(InstrumentSlotPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn enabled(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, |s| s.enabled).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/key-transform",
    params(InstrumentSlotPath),
    request_body(
        content = MidiKeyTransform,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_key_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(transform): Json<MidiKeyTransform>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.set_key_transform(path.slot_id, transform).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/key-transform",
    params(InstrumentSlotPath),
    responses(
        (
            status = OK,
            body = MidiKeyTransform,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn key_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<MidiKeyTransform>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, |s| s.key_transform.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/velocity-transform",
    params(InstrumentSlotPath),
    request_body(
        content = MidiVelocityTransform,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_velocity_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(transform): Json<MidiVelocityTransform>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.set_velocity_transform(path.slot_id, transform).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/velocity-transform",
    params(InstrumentSlotPath),
    responses(
        (
            status = OK,
            body = MidiVelocityTransform,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn velocity_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<MidiVelocityTransform>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, |s| s.velocity_transform.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/channel-filter",
    params(InstrumentSlotPath),
    request_body(
        content = MidiFilter,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_channel_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(filter): Json<MidiFilter>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.set_channel_filter(path.slot_id, filter).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/channel-filter",
    params(InstrumentSlotPath),
    responses(
        (status = OK, body = MidiFilter, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn channel_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<MidiFilter>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, |s| s.channel_filter.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/cc-filter",
    params(InstrumentSlotPath),
    request_body(
        content = MidiFilter,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_cc_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(filter): Json<MidiFilter>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.set_cc_filter(path.slot_id, filter).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/cc-filter",
    params(InstrumentSlotPath),
    responses(
        (status = OK, body = MidiFilter, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn cc_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<MidiFilter>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, |s| s.cc_filter.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/use-global-transposition",
    params(InstrumentSlotPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_use_global_transposition(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_slot(&app_state, path.slot_id, |_| ()).await?;
    app_state.engine_handle.set_use_global_transposition(path.slot_id, enabled).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/use-global-transposition",
    params(InstrumentSlotPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Slot not found"),
    )
)]
async fn use_global_transposition(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(with_slot(&app_state, path.slot_id, |s| s.use_global_transposition).await?))
}

//--------------------------------------------------------------------------------------------------

impl From<EngineError> for (StatusCode, String) {
    fn from(value: EngineError) -> Self {
        map_engine_error(value)
    }
}

fn map_engine_error(error: EngineError) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

impl From<synths::Error> for (StatusCode, String) {
    fn from(value: synths::Error) -> Self {
        map_synth_error(value)
    }
}

fn map_synth_error(error: synths::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub async fn with_slot<T>(
    app_state: &AppState,
    slot_id: LocalId,
    f: impl FnOnce(&instrument_graph::CtrSlot) -> T,
) -> Result<T, (StatusCode, String)> {
    let project = app_state.engine_handle.project.read().await;
    let result_opt = project.instrument_graph.slots.get(slot_id).map(f);
    result_opt.ok_or((StatusCode::NOT_FOUND, "not found".into()))
}

pub async fn with_dummy_synth<T>(
    app_state: &AppState,
    synth_id: LocalId,
    f: impl FnOnce(&dummy_graph::CtrSynth) -> T,
) -> Result<T, (StatusCode, String)> {
    let project = app_state.engine_handle.project.read().await;
    let result_opt = project.dummy_graph.synths.get(synth_id).map(f);
    result_opt.ok_or((StatusCode::NOT_FOUND, "not found".into()))
}

pub async fn with_fluid_synth<T>(
    app_state: &AppState,
    synth_id: LocalId,
    f: impl FnOnce(&fluid_graph::CtrSynth) -> T,
) -> Result<T, (StatusCode, String)> {
    let project = app_state.engine_handle.project.read().await;
    let result_opt = project.fluid_graph.synths.get(synth_id).map(f);
    result_opt.ok_or((StatusCode::NOT_FOUND, "not found".into()))
}

pub async fn with_sfizz_synth<T>(
    app_state: &AppState,
    synth_id: LocalId,
    f: impl FnOnce(&sfizz_graph::CtrSynth) -> T,
) -> Result<T, (StatusCode, String)> {
    let project = app_state.engine_handle.project.read().await;
    let result_opt = project.sfizz_graph.synths.get(synth_id).map(f);
    result_opt.ok_or((StatusCode::NOT_FOUND, "not found".into()))
}
