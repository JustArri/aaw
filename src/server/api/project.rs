use crate::{
    engine::{
        self,
        ctr::EngineError,
        graph::LocalId,
        instrument_graph::{self, SynthId},
    },
    server::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::collections::HashMap;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(project))
        .routes(routes!(set_project))
        .routes(routes!(global_transposition))
        .routes(routes!(set_global_transposition))
        .routes(routes!(instrument_slots))
        .routes(routes!(instrument_slot_append_new))
        .routes(routes!(instrument_slot))
        .routes(routes!(instrument_slot_remove))
        .routes(routes!(set_synth))
        .routes(routes!(synth))
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = OK, body = engine::stored::Project, description = "OK"),
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
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_project(
    State(app_state): State<AppState>,
    Json(project): Json<engine::stored::Project>,
) -> Result<StatusCode, (StatusCode, String)> {
    let (ctr_project, rt_project) =
        engine::load(project, app_state.min_instrument_slots, app_state.min_synths_per_graph)
            .await
            .map_err(|err| {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to load project: {err}"))
            })?;

    app_state.engine_handle.replace_project(ctr_project, rt_project).await.map_err(|err| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to replace project: {err}"))
    })?;

    Ok(StatusCode::NO_CONTENT)
}

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

#[utoipa::path(
    put,
    path = "/graph/instrument/global-transposition",
    request_body(
        content = i8,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Global transposition set"),
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

#[utoipa::path(
    get,
    path = "/graph/instrument/slots",
    responses(
        (status = OK, body = HashMap<LocalId, instrument_graph::ctr::Slot>, description = "OK"),
    )
)]
async fn instrument_slots(
    State(app_state): State<AppState>,
) -> Json<HashMap<LocalId, instrument_graph::ctr::Slot>> {
    Json(app_state.engine_handle.project.read().await.instrument_graph.slots.clone().into())
}

#[utoipa::path(
    post,
    path = "/graph/instrument/slots",
    responses(
        (status = NO_CONTENT, description = "Instrument slot appended"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_slot_append_new(
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.instrument_slot_append_new().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}",
    params(InstrumentSlotPath),
    responses(
        (status = OK, body = instrument_graph::ctr::Slot, description = "OK"),
        (status = NOT_FOUND, description = "Slot not found"),
    )
)]
async fn instrument_slot(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<instrument_graph::ctr::Slot>, StatusCode> {
    let value = app_state
        .engine_handle
        .project
        .read()
        .await
        .instrument_graph
        .slots
        .get(path.slot_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(value))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
struct InstrumentSlotPath {
    #[param(value_type = usize)]
    slot_id: LocalId,
}

#[utoipa::path(
    delete,
    path = "/graph/instrument/slots/{slot_id}",
    params(InstrumentSlotPath),
    responses(
        (status = NO_CONTENT, description = "Instrument slot removed"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_slot_remove(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.instrument_slot_remove(path.slot_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/graph/instrument/slots/{slot_id}/synth",
    params(InstrumentSlotPath),
    request_body = Option<SynthId>,
    responses(
        (status = NO_CONTENT, description = "Instrument slot appended"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_synth(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
    Json(synth): Json<Option<SynthId>>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.set_synth(path.slot_id, synth).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/graph/instrument/slots/{slot_id}/synth",
    params(InstrumentSlotPath),
    responses(
        (status = OK, body = Option<SynthId>, description = "OK"),
        (status = NOT_FOUND, description = "Slot not found"),
    )
)]
async fn synth(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentSlotPath>,
) -> Result<Json<Option<SynthId>>, StatusCode> {
    let value = app_state
        .engine_handle
        .project
        .read()
        .await
        .instrument_graph
        .slots
        .get(path.slot_id)
        .map(|s| s.synth_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(value))
}

impl From<EngineError> for (StatusCode, String) {
    fn from(value: EngineError) -> Self {
        map_engine_error(value)
    }
}

fn map_engine_error(error: EngineError) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
