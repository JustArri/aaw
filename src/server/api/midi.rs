use crate::{
    midi::{InputAutoConnection, InputSlotError},
    server::AppState,
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
        .routes(routes!(input_ports))
        .routes(routes!(input_slots))
        .routes(routes!(input_auto_connections))
        .routes(routes!(insert_input_auto_connection))
        .routes(routes!(delete_input_auto_connection))
        .routes(routes!(connect_input_slot))
        .routes(routes!(disconnect_input_slot))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MidiPort {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum MidiInputSlot {
    Connected { port_id: String },
    Disconnected,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MidiInputAutoConnection {
    pub port_id: String,

    #[schema(required)]
    pub preferred_slot_id: Option<usize>,

    pub slot_id_fixed: bool,
}

#[utoipa::path(
    get,
    path = "/input/ports",
    responses(
        (status = OK, body = Vec<MidiPort>, description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn input_ports(State(app_state): State<AppState>) -> Json<Vec<MidiPort>> {
    let ports = app_state
        .midi_runtime
        .lock()
        .await
        .available_input_ports()
        .iter()
        .map(|p| MidiPort { id: p.id.clone(), name: p.name.clone() })
        .collect();
    Json(ports)
}

#[utoipa::path(
    get,
    path = "/input/slots",
    responses(
        (status = OK, body = Vec<MidiInputSlot>, description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn input_slots(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<MidiInputSlot>>, StatusCode> {
    use crate::midi::InputSlot;

    let runtime = &app_state.midi_runtime.lock().await;
    let ports: Option<Vec<_>> = runtime
        .inputs()
        .iter()
        .map(|slot| match slot {
            InputSlot::Connected(s) => {
                Some(MidiInputSlot::Connected { port_id: s.port_id().clone() })
            }
            InputSlot::Disconnected(_) => Some(MidiInputSlot::Disconnected),
            InputSlot::Empty => None,
        })
        .collect();

    Ok(Json(ports.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct InputSlotPath {
    slot_id: usize,
}

#[derive(Deserialize, ToSchema)]
pub struct ConnectInputSlotRequest {
    port_id: String,
}

#[utoipa::path(
    post,
    path = "/input/slots/connect/{slot_id}",
    params(InputSlotPath),
    request_body = ConnectInputSlotRequest,
    responses(
        (status = NO_CONTENT, description = "Connected"),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn connect_input_slot(
    State(app_state): State<AppState>,
    Path(path): Path<InputSlotPath>,
    Json(req): Json<ConnectInputSlotRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state
        .midi_runtime
        .lock()
        .await
        .connect_input(path.slot_id, &req.port_id)
        .map_err(map_input_slot_err)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/input/slots/disconnect/{slot_id}",
    params(InputSlotPath),
    responses(
        (status = NO_CONTENT, description = "Disconnected"),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn disconnect_input_slot(
    State(app_state): State<AppState>,
    Path(path): Path<InputSlotPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state
        .midi_runtime
        .lock()
        .await
        .disconnect_input(path.slot_id)
        .map_err(map_input_slot_err)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/input/auto-connections",
    responses(
        (status = OK, body = Vec<MidiInputAutoConnection>, description = "OK"),
    )
)]
async fn input_auto_connections(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<MidiInputAutoConnection>>, StatusCode> {
    let auto_connections = app_state
        .midi_runtime
        .lock()
        .await
        .input_auto_connections()
        .iter()
        .map(|ac| MidiInputAutoConnection {
            port_id: ac.0.into(),
            preferred_slot_id: ac.1.preferred_slot_id,
            slot_id_fixed: ac.1.slot_id_fixed,
        })
        .collect();
    Ok(Json(auto_connections))
}

#[utoipa::path(
    put,
    path = "/input/auto-connections",
    request_body = MidiInputAutoConnection,
    responses(
        (status = NO_CONTENT, description = "Inserted"),
    )
)]
async fn insert_input_auto_connection(
    State(app_state): State<AppState>,
    Json(req): Json<MidiInputAutoConnection>,
) -> StatusCode {
    app_state.midi_runtime.lock().await.input_auto_connections_mut().insert(
        req.port_id,
        InputAutoConnection {
            preferred_slot_id: req.preferred_slot_id,
            slot_id_fixed: req.slot_id_fixed,
        },
    );
    StatusCode::NO_CONTENT
}

#[derive(Deserialize, ToSchema)]
pub struct DeleteMidiAutoConnectionRequest {
    pub port_id: String,
}

#[utoipa::path(
    delete,
    path = "/input/auto-connections",
    request_body = DeleteMidiAutoConnectionRequest,
    responses(
        (status = NO_CONTENT, description = "Deleted"),
    )
)]
async fn delete_input_auto_connection(
    State(app_state): State<AppState>,
    Json(req): Json<DeleteMidiAutoConnectionRequest>,
) -> StatusCode {
    app_state.midi_runtime.lock().await.input_auto_connections_mut().remove(&req.port_id);
    StatusCode::NO_CONTENT
}

fn map_input_slot_err(err: InputSlotError) -> (StatusCode, String) {
    let code = match &err {
        InputSlotError::AlreadyDisconnected => StatusCode::BAD_REQUEST,
        InputSlotError::AlreadyConnected => StatusCode::BAD_REQUEST,
        InputSlotError::NotConnected => StatusCode::BAD_REQUEST,
        InputSlotError::NotDisconnected => StatusCode::BAD_REQUEST,
        InputSlotError::InvalidId(_) => StatusCode::NOT_FOUND,
        InputSlotError::Connect(midir::ConnectErrorKind::InvalidPort) => StatusCode::BAD_REQUEST,
        InputSlotError::Connect(midir::ConnectErrorKind::Other(_)) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        InputSlotError::PortInfo(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, err.to_string())
}
