use crate::{midi::MidiEvent, server::AppState};
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
}

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

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut midi_rx = state.midi_tx.subscribe();

    loop {
        let action = tokio::select! {
            midi_rx_result = midi_rx.recv() => {
                handle_midi_rx_result(midi_rx_result, &mut socket).await
            }
            message = socket.recv() => {
                handle_socket_rx_result(message)
            }
        };

        if matches!(action, Action::Exit) {
            break;
        }
    }
}

async fn handle_midi_rx_result(
    midi_event_result: Result<Arc<MidiEvent>, RecvError>,
    socket: &mut WebSocket,
) -> Action {
    match midi_event_result {
        Ok(midi_event) => {
            let notif = Notification::MidiEvent(&midi_event);
            tracing::trace!(target: "notif", ?notif);
            let json = match serde_json::to_string(&notif) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!(target: "web", ?err, "failed to serialize notification");
                    return Action::Continue;
                }
            };

            if socket.send(Message::Text(json.into())).await.is_err() {
                Action::Exit
            } else {
                Action::Continue
            }
        }
        Err(RecvError::Lagged(count)) => {
            tracing::warn!(target: "web", count, "notification websocket receiver lagged");
            Action::Continue
        }
        Err(RecvError::Closed) => Action::Exit,
    }
}

fn handle_socket_rx_result(message: Option<Result<Message, axum::Error>>) -> Action {
    match message {
        Some(Ok(Message::Close(_))) => Action::Exit,
        Some(Ok(_)) => Action::Continue,
        Some(Err(err)) => {
            tracing::error!(target: "web", ?err, "notification websocket error");
            Action::Exit
        }
        None => Action::Exit,
    }
}

enum Action {
    Continue,
    Exit,
}
