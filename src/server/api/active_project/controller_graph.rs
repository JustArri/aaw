use crate::{
    app_state::AppState,
    engine::{
        controller_graph::CtrController,
        graph::{ControllerAddress, ControllerId, InstrumentId},
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

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(set_global_transposition))
        .routes(routes!(global_transposition))
        .routes(routes!(controller_list, controller_append))
        .routes(routes!(controller, controller_replace, controller_remove))
}

#[derive(Debug, Clone, Copy, Deserialize, IntoParams)]
struct InstrumentPath {
    #[param(value_type = usize)]
    instrument_id: InstrumentId,
}

#[derive(Debug, Clone, Copy, Deserialize, IntoParams)]
struct ControllerPath {
    #[param(value_type = usize)]
    instrument_id: InstrumentId,

    #[param(value_type = usize)]
    controller_id: ControllerId,
}

impl From<ControllerPath> for ControllerAddress {
    fn from(path: ControllerPath) -> Self {
        Self { instrument_id: path.instrument_id, controller_id: path.controller_id }
    }
}

#[utoipa::path(
    put,
    path = "/global-transposition",
    request_body(content = i8, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Global transposition set"))
)]
async fn set_global_transposition(
    State(app_state): State<AppState>,
    Json(transposition): Json<i8>,
) -> StatusCode {
    app_state.engine_handle.set_global_transposition(transposition).await;
    StatusCode::NO_CONTENT
}

#[utoipa::path(
    get,
    path = "/global-transposition",
    responses((status = OK, body = i8, content_type = "application/json", description = "Global transposition"))
)]
async fn global_transposition(State(app_state): State<AppState>) -> Json<i8> {
    Json(app_state.engine_handle.project().await.instrument_graph.global_transposition)
}

#[utoipa::path(
    get,
    path = "/instruments/{instrument_id}/controllers",
    params(InstrumentPath),
    responses((status = OK, body = Vec<(ControllerId, CtrController)>, description = "Instrument controllers"))
)]
async fn controller_list(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<Vec<(ControllerId, CtrController)>>, StatusCode> {
    let project = app_state.engine_handle.project().await;
    let instrument = project
        .instrument_graph
        .instruments
        .get(path.instrument_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(
        (&instrument.controllers)
            .into_iter()
            .map(|(id, controller)| (*id, controller.clone()))
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/instruments/{instrument_id}/controllers",
    params(InstrumentPath),
    responses((status = OK, body = ControllerId, description = "Created controller ID"))
)]
async fn controller_append(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<ControllerId>, (StatusCode, String)> {
    Ok(Json(app_state.engine_handle.controller_append(path.instrument_id).await?))
}

#[utoipa::path(
    get,
    path = "/instruments/{instrument_id}/controllers/{controller_id}",
    params(ControllerPath),
    responses(
        (status = OK, body = CtrController, description = "Controller"),
        (status = NOT_FOUND, description = "Controller not found")
    )
)]
async fn controller(
    State(app_state): State<AppState>,
    Path(path): Path<ControllerPath>,
) -> Result<Json<CtrController>, StatusCode> {
    let project = app_state.engine_handle.project().await;
    let controller = project
        .instrument_graph
        .controller(path.instrument_id, path.controller_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(controller.clone()))
}

#[utoipa::path(
    put,
    path = "/instruments/{instrument_id}/controllers/{controller_id}",
    params(ControllerPath),
    request_body(content = CtrController, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Controller replaced"))
)]
async fn controller_replace(
    State(app_state): State<AppState>,
    Path(path): Path<ControllerPath>,
    Json(controller): Json<CtrController>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.controller_replace(path.into(), controller).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/instruments/{instrument_id}/controllers/{controller_id}",
    params(ControllerPath),
    responses((status = NO_CONTENT, description = "Controller removed"))
)]
async fn controller_remove(
    State(app_state): State<AppState>,
    Path(path): Path<ControllerPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.controller_remove(path.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}
