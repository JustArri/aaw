use crate::{
    app_state::AppState, engine::graph::LocalId, midi::MidiEvent, server::WsHandlerAction,
};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::Response,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum Notification<'a> {
    MidiEvent(&'a MidiEvent),
    ProjectChange(&'a ProjectChange),
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectChange {
    pub revision: u32,

    #[serde(flatten)]
    pub kind: ProjectChangeKind,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum ProjectChangeKind {
    Full,
    Id,
    Name,
    DummySynth { id: LocalId },
    DummySynthName { id: LocalId },
    FluidSynth { id: LocalId },
    FluidSynthProgram { id: LocalId },
    FluidSynthName { id: LocalId },
    SfizzSynth { id: LocalId },
    SfizzSynthName { id: LocalId },
    GlobalTransposition,
    InstrumentSlot { id: LocalId },
    InstrumentSynth { id: LocalId },
    InstrumentEnabled { id: LocalId },
    InstrumentKeyTransform { id: LocalId },
    InstrumentVelocityTransform { id: LocalId },
    InstrumentChannelFilter { id: LocalId },
    InstrumentCcFilter { id: LocalId },
    InstrumentUseGlobalTransposition { id: LocalId },
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/notifications",
    operation_id = "notifications",
    description =
        "Upgrades the HTTP connection to a WebSocket. \
         WebSocket messages are JSON serialized Notification model.",
    responses(
        (status = SWITCHING_PROTOCOLS, description = "WebSocket connection established"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

//--------------------------------------------------------------------------------------------------

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut midi_rx = state.midi_tx.subscribe();
    let mut project_change_rx = state.project_change_tx.subscribe();

    loop {
        let action = tokio::select! {
            rx_result = midi_rx.recv() => {
                handle_broadcast_rx_result(rx_result, &mut socket, |event| {
                    Notification::MidiEvent(event)
                })
                .await
            }
            rx_result = project_change_rx.recv() => {
                handle_broadcast_rx_result(rx_result, &mut socket, |change| {
                    Notification::ProjectChange(change)
                })
                .await
            }
            message = socket.recv() => {
                handle_socket_rx_result(message)
            }
        };

        if matches!(action, WsHandlerAction::Exit) {
            break;
        }
    }
}

async fn handle_broadcast_rx_result<T, F>(
    rx_result: Result<Arc<T>, RecvError>,
    socket: &mut WebSocket,
    to_notif: F,
) -> WsHandlerAction
where
    F: for<'a> FnOnce(&'a T) -> Notification<'a>,
{
    match rx_result {
        Ok(data) => {
            let notif = to_notif(&data);
            tracing::trace!(target: "notif", ?notif);
            let json = match serde_json::to_string(&notif) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!(target: "web", ?err, "failed to serialize notification");
                    return WsHandlerAction::Continue;
                }
            };

            if socket.send(Message::Text(json.into())).await.is_err() {
                WsHandlerAction::Exit
            } else {
                WsHandlerAction::Continue
            }
        }
        Err(RecvError::Lagged(count)) => {
            tracing::warn!(target: "web", count, "notification websocket receiver lagged");
            WsHandlerAction::Continue
        }
        Err(RecvError::Closed) => WsHandlerAction::Exit,
    }
}

fn handle_socket_rx_result(message: Option<Result<Message, axum::Error>>) -> WsHandlerAction {
    match message {
        Some(Ok(Message::Close(_))) => WsHandlerAction::Exit,
        Some(Ok(_)) => WsHandlerAction::Continue,
        Some(Err(err)) => {
            tracing::error!(target: "web", ?err, "notification websocket error");
            WsHandlerAction::Exit
        }
        None => WsHandlerAction::Exit,
    }
}
