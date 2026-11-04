use crate::{
    app_state::AppState,
    engine::{
        EngineHandle,
        drum_machine::MusicalTransport,
        effects::Effects,
        graph::{ControllerAddress, InstrumentId},
        instrument_graph::CtrInstrumentKind,
        metrics::EngineMetrics,
        midi_app_controls::{CtrMidiEngineControls, CtrMidiProjectControls},
        project::CtrProjectStorage,
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
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::broadcast::{self, error::RecvError},
    task::JoinHandle,
    time::{Instant, MissedTickBehavior},
};
use tokio_util::sync::CancellationToken;
use utoipa::ToSchema;

// Clients extrapolate between updates; this only bounds browser/audio clock drift.
const TRANSPORT_POSITION_CORRECTION_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum Notification<'a> {
    MidiEvent(&'a MidiEvent),
    ProjectChange(&'a ProjectChange),
    EngineMidiAppControls(&'a CtrMidiEngineControls),
    TransportPosition(f64),
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
    GlobalEffects { effects: Effects },
    GlobalGain { gain: f32 },
    Instrument { id: InstrumentId, removed: bool },
    InstrumentKind { id: InstrumentId, kind: CtrInstrumentKind },
    InstrumentName { id: InstrumentId, name: String },
    InstrumentSoundfontProgram { id: InstrumentId, program: SoundFontProgram },
    InstrumentClapParameter { id: InstrumentId, parameter_id: u32, value: f64 },
    InstrumentEffects { id: InstrumentId, effects: Effects },
    InstrumentGain { id: InstrumentId, gain: f32 },
    GlobalTransposition { transposition: i8 },
    Controller { address: ControllerAddress, removed: bool },
    ControllerEnabled { address: ControllerAddress, flag: bool },
    ControllerSourceFilter { address: ControllerAddress, filter: MidiFilter },
    ControllerIgnoreKeyTransform { address: ControllerAddress, flag: bool },
    ControllerKeyTransform { address: ControllerAddress, transform: MidiKeyTransform },
    ControllerIgnoreVelocityTransform { address: ControllerAddress, flag: bool },
    ControllerVelocityTransform { address: ControllerAddress, transform: MidiVelocityTransform },
    ControllerChannelFilter { address: ControllerAddress, filter: MidiFilter },
    ControllerCcFilter { address: ControllerAddress, filter: MidiFilter },
    ControllerIgnorePitchBend { address: ControllerAddress, flag: bool },
    ControllerIgnoreProgramChange { address: ControllerAddress, flag: bool },
    ControllerIgnoreChannelAftertouch { address: ControllerAddress, flag: bool },
    ControllerIgnorePolyphonicAftertouch { address: ControllerAddress, flag: bool },
    ControllerUseGlobalTransposition { address: ControllerAddress, flag: bool },
    ControllerName { address: ControllerAddress, name: String },
    UserPresets,
    MidiAppControls { controls: CtrMidiProjectControls },
    Transport { transport: MusicalTransport },
}

//--------------------------------------------------------------------------------------------------

pub fn spawn_transport_position_task(
    stop: CancellationToken,
    engine_handle: EngineHandle,
    engine_metrics: Arc<EngineMetrics>,
    transport_position_tx: broadcast::Sender<f64>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let first_correction = Instant::now() + TRANSPORT_POSITION_CORRECTION_INTERVAL;
        let mut interval =
            tokio::time::interval_at(first_correction, TRANSPORT_POSITION_CORRECTION_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = stop.cancelled() => break,
                _ = interval.tick() => {
                    if engine_handle.project().await.transport.playing {
                        let _ = transport_position_tx.send(engine_metrics.transport_tick());
                    }
                }
            }
        }
    })
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
    let mut midi_app_controls_change_rx = state.engine_handle.subscribe_midi_app_controls_changes();
    let mut transport_position_rx = state.transport_position_tx.subscribe();
    let mut transport_playing = state.engine_handle.project().await.transport.playing;

    if transport_playing
        && matches!(
            send_transport_position(&mut socket, state.engine_metrics.transport_tick()).await,
            WsHandlerAction::Exit
        )
    {
        return;
    }

    loop {
        let action = tokio::select! {
            rx_result = midi_rx.recv() => {
                handle_broadcast_rx_result(rx_result, &mut socket, |event| {
                    Notification::MidiEvent(event)
                })
                .await
            }
            rx_result = project_change_rx.recv() => {
                handle_project_change_rx_result(
                    rx_result,
                    &mut socket,
                    &state,
                    &mut transport_playing,
                ).await
            }
            rx_result = midi_app_controls_change_rx.recv() => {
                handle_broadcast_rx_result(rx_result, &mut socket, |controls| {
                    Notification::EngineMidiAppControls(controls)
                })
                .await
            }
            rx_result = transport_position_rx.recv() => {
                match rx_result {
                    Ok(tick) => send_transport_position(&mut socket, tick).await,
                    Err(RecvError::Lagged(count)) => {
                        tracing::warn!(target: "web", count, "transport position websocket receiver lagged");
                        WsHandlerAction::Continue
                    }
                    Err(RecvError::Closed) => WsHandlerAction::Exit,
                }
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

async fn handle_project_change_rx_result(
    rx_result: Result<Arc<ProjectChange>, RecvError>,
    socket: &mut WebSocket,
    state: &AppState,
    transport_playing: &mut bool,
) -> WsHandlerAction {
    let change = match rx_result {
        Ok(change) => change,
        Err(RecvError::Lagged(count)) => {
            tracing::warn!(target: "web", count, "notification websocket receiver lagged");
            return WsHandlerAction::Continue;
        }
        Err(RecvError::Closed) => return WsHandlerAction::Exit,
    };

    let was_playing = *transport_playing;
    match &change.kind {
        ProjectChangeKind::Transport { transport } => {
            *transport_playing = transport.playing;
        }
        ProjectChangeKind::Full => {
            *transport_playing = state.engine_handle.project().await.transport.playing;
        }
        _ => {}
    }

    if matches!(
        send_notification(socket, Notification::ProjectChange(&change)).await,
        WsHandlerAction::Exit
    ) {
        return WsHandlerAction::Exit;
    }

    if !was_playing && *transport_playing {
        send_transport_position(socket, state.engine_metrics.transport_tick()).await
    } else {
        WsHandlerAction::Continue
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
            send_notification(socket, notif).await
        }
        Err(RecvError::Lagged(count)) => {
            tracing::warn!(target: "web", count, "notification websocket receiver lagged");
            WsHandlerAction::Continue
        }
        Err(RecvError::Closed) => WsHandlerAction::Exit,
    }
}

async fn send_notification(socket: &mut WebSocket, notif: Notification<'_>) -> WsHandlerAction {
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

async fn send_transport_position(socket: &mut WebSocket, tick: f64) -> WsHandlerAction {
    send_notification(socket, Notification::TransportPosition(tick)).await
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
