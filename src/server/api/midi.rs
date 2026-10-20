use crate::{
    app_state::AppState,
    midi::{InputAutoConnection, InputSlotError, MidiEvent},
    rt_channels::NotifiedRtSharedProducer,
    server::WsHandlerAction,
};
use axum::{
    Json,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(midi_input_ports))
        .routes(routes!(midi_input_slots))
        .routes(routes!(midi_input_auto_connections))
        .routes(routes!(midi_connect_input_slot))
        .routes(routes!(midi_disconnect_input_slot))
        .routes(routes!(midi_insert_input_auto_connection))
        .routes(routes!(midi_delete_input_auto_connection))
        .routes(routes!(midi_send_event))
        .routes(routes!(midi_open_event_stream))
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

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/input/ports",
    responses(
        (status = OK, body = Vec<MidiPort>, description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn midi_input_ports(State(app_state): State<AppState>) -> Json<Vec<MidiPort>> {
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

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/input/slots",
    responses(
        (status = OK, body = Vec<MidiInputSlot>, description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn midi_input_slots(
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

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct InputSlotPath {
    slot_id: usize,
}

#[derive(Deserialize, ToSchema)]
pub struct ConnectInputSlotRequest {
    port_id: String,
}

//--------------------------------------------------------------------------------------------------

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
async fn midi_connect_input_slot(
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

//--------------------------------------------------------------------------------------------------

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
async fn midi_disconnect_input_slot(
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

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/input/auto-connections",
    responses(
        (status = OK, body = Vec<MidiInputAutoConnection>, description = "OK"),
    )
)]
async fn midi_input_auto_connections(
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

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/input/auto-connections",
    request_body = MidiInputAutoConnection,
    responses(
        (status = NO_CONTENT, description = "Inserted"),
    )
)]
async fn midi_insert_input_auto_connection(
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

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, ToSchema)]
pub struct DeleteMidiAutoConnectionRequest {
    pub port_id: String,
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/input/auto-connections",
    request_body = DeleteMidiAutoConnectionRequest,
    responses(
        (status = NO_CONTENT, description = "Deleted"),
    )
)]
async fn midi_delete_input_auto_connection(
    State(app_state): State<AppState>,
    Json(req): Json<DeleteMidiAutoConnectionRequest>,
) -> StatusCode {
    app_state.midi_runtime.lock().await.input_auto_connections_mut().remove(&req.port_id);
    StatusCode::NO_CONTENT
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/events",
    request_body = MidiEvent,
    responses(
        (status = NO_CONTENT, description = "Connected"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn midi_send_event(
    State(app_state): State<AppState>,
    Json(event): Json<MidiEvent>,
) -> Result<StatusCode, (StatusCode, String)> {
    for sender in app_state.midi_event_senders.iter() {
        sender
            .push(event)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "event queue full".into()))?;
    }

    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/events",
    operation_id = "midi-event-stream",
    description =
        "Upgrades the HTTP connection to a WebSocket. \
         WebSocket messages are JSON serialized MidiEvent model.",
    responses(
        (status = SWITCHING_PROTOCOLS, description = "WebSocket connection established"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
pub async fn midi_open_event_stream(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<axum::response::Response, StatusCode> {
    Ok(ws.on_upgrade(move |socket| handle_event_stream_socket(socket, state)))
}

//--------------------------------------------------------------------------------------------------

async fn handle_event_stream_socket(mut socket: WebSocket, state: AppState) {
    let mut midi_rx = state.midi_tx.subscribe();
    let midi_senders = state.midi_event_senders;

    loop {
        let action = tokio::select! {
            midi_rx_result = midi_rx.recv() => {
                handle_midi_rx_result(midi_rx_result, &mut socket).await
            }
            message = socket.recv() => {
                handle_socket_rx_result(message, midi_senders.as_ref()).await
            }
        };

        if matches!(action, WsHandlerAction::Exit) {
            break;
        }
    }
}

async fn handle_midi_rx_result(
    midi_event_result: Result<Arc<MidiEvent>, broadcast::error::RecvError>,
    socket: &mut WebSocket,
) -> WsHandlerAction {
    match midi_event_result {
        Ok(midi_event) => {
            let json = match serde_json::to_string(midi_event.as_ref()) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!(target: "web", ?err, "failed to serialize midi event to send");
                    return WsHandlerAction::Continue;
                }
            };

            if socket.send(Message::Text(json.into())).await.is_err() {
                WsHandlerAction::Exit
            } else {
                WsHandlerAction::Continue
            }
        }
        Err(broadcast::error::RecvError::Lagged(count)) => {
            tracing::warn!(target: "web", count, "midi event stream websocket receiver lagged");
            WsHandlerAction::Continue
        }
        Err(broadcast::error::RecvError::Closed) => WsHandlerAction::Exit,
    }
}

async fn handle_socket_rx_result(
    message: Option<Result<Message, axum::Error>>,
    midi_senders: &[NotifiedRtSharedProducer<MidiEvent>],
) -> WsHandlerAction {
    match message {
        Some(Ok(Message::Close(_))) => WsHandlerAction::Exit,
        Some(Ok(Message::Text(text))) => {
            match serde_json::from_slice::<MidiEvent>(text.as_bytes()) {
                Ok(midi_event) => {
                    for sender in midi_senders {
                        let _ = sender.push(midi_event).await;
                    }
                }
                Err(err) => {
                    tracing::error!(target: "web", ?err, "failed to serialize received midi event");
                }
            }
            WsHandlerAction::Continue
        }
        Some(Ok(_)) => WsHandlerAction::Continue,
        Some(Err(err)) => {
            tracing::error!(target: "web", ?err, "notification websocket error");
            WsHandlerAction::Exit
        }
        None => WsHandlerAction::Exit,
    }
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
