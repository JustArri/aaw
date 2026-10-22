use crate::{
    app_state::AppState,
    engine::{
        effects::Effects,
        graph::{InstrumentId, SynthId},
        project::CtrProjectStorage,
        synth_graph::CtrSynthParameters,
    },
    midi::{MidiEvent, MidiFilter, MidiKeyTransform, MidiVelocityTransform},
    server::WsHandlerAction,
    synths::SoundFontProgram,
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
    Storage { storage: Option<CtrProjectStorage> },
    Name { name: String },
    Effects { effects: Effects },
    Synth { id: SynthId, removed: bool },
    SynthKind { id: SynthId, parameters: CtrSynthParameters },
    SynthName { id: SynthId, name: String },
    SynthSoundfontProgram { id: SynthId, program: SoundFontProgram },
    SynthEffects { id: SynthId, effects: Effects },
    GlobalTransposition { transposition: i8 },
    Instrument { id: InstrumentId },
    InstrumentSynth { id: InstrumentId, synth_id: Option<SynthId> },
    InstrumentEnabled { id: InstrumentId, flag: bool },
    InstrumentIgnoreKeyTransform { id: InstrumentId, flag: bool },
    InstrumentKeyTransform { id: InstrumentId, transform: MidiKeyTransform },
    InstrumentIgnoreVelocityTransform { id: InstrumentId, flag: bool },
    InstrumentVelocityTransform { id: InstrumentId, transform: MidiVelocityTransform },
    InstrumentChannelFilter { id: InstrumentId, filter: MidiFilter },
    InstrumentCcFilter { id: InstrumentId, filter: MidiFilter },
    InstrumentIgnorePitchBend { id: InstrumentId, flag: bool },
    InstrumentIgnoreProgramChange { id: InstrumentId, flag: bool },
    InstrumentIgnoreChannelAftertouch { id: InstrumentId, flag: bool },
    InstrumentIgnorePolyphonicAftertouch { id: InstrumentId, flag: bool },
    InstrumentUseGlobalTransposition { id: InstrumentId, flag: bool },
    InstrumentName { id: InstrumentId, name: String },
    UserPresets,
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
    let mut project_change_rx = state.engine_handle.subscribe_project_changes();

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
