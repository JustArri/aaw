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
    synths::SoundFontProgram,
};
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::{self, Stream};
use serde::Serialize;
use std::{convert::Infallible, sync::Arc, time::Duration};
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
    description = "Server-sent events. Each event's data is a JSON serialized [Notification](#/components/schemas/Notification). Events are not replayed after a disconnect.",
    responses(
        (status = OK, body = String, content_type = "text/event-stream", description = "Stream of JSON Notification events"),
    )
)]
pub async fn handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let midi_rx = state.midi_tx.subscribe();
    let project_change_rx = state.engine_handle.subscribe_project_changes();
    let midi_app_controls_change_rx = state.engine_handle.subscribe_midi_app_controls_changes();
    let transport_position_rx = state.transport_position_tx.subscribe();
    let transport_playing = state.engine_handle.project().await.transport.playing;

    let stream_state = NotificationStream {
        state,
        midi_rx,
        project_change_rx,
        midi_app_controls_change_rx,
        transport_position_rx,
        transport_playing,
        pending_position: transport_playing,
    };
    let stream = stream::unfold(stream_state, |mut stream| async move {
        loop {
            if stream.pending_position {
                stream.pending_position = false;
                let notification =
                    Notification::TransportPosition(stream.state.engine_metrics.transport_tick());
                if let Some(event) = notification_event(notification) {
                    return Some((Ok(event), stream));
                }
            }

            let event = tokio::select! {
                result = stream.midi_rx.recv() => {
                    notification_from_broadcast(result, |event| Notification::MidiEvent(event))
                }
                result = stream.midi_app_controls_change_rx.recv() => {
                    notification_from_broadcast(result, |controls| {
                        Notification::EngineMidiAppControls(controls)
                    })
                }
                result = stream.transport_position_rx.recv() => {
                    match result {
                        Ok(tick) => Some(notification_event(Notification::TransportPosition(tick))),
                        Err(error) => broadcast_error(error),
                    }
                }
                result = stream.project_change_rx.recv() => {
                    match result {
                        Ok(change) => {
                            let was_playing = stream.transport_playing;
                            match &change.kind {
                                ProjectChangeKind::Transport { transport } => {
                                    stream.transport_playing = transport.playing;
                                }
                                ProjectChangeKind::Full => {
                                    stream.transport_playing = stream
                                        .state
                                        .engine_handle
                                        .project()
                                        .await
                                        .transport
                                        .playing;
                                }
                                _ => {}
                            }
                            stream.pending_position = !was_playing && stream.transport_playing;
                            Some(notification_event(Notification::ProjectChange(&change)))
                        }
                        Err(error) => broadcast_error(error),
                    }
                }
            };

            match event {
                Some(Some(event)) => return Some((Ok(event), stream)),
                Some(None) => continue,
                None => return None,
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

struct NotificationStream {
    state: AppState,
    midi_rx: broadcast::Receiver<Arc<MidiEvent>>,
    project_change_rx: broadcast::Receiver<Arc<ProjectChange>>,
    midi_app_controls_change_rx: broadcast::Receiver<Arc<CtrMidiEngineControls>>,
    transport_position_rx: broadcast::Receiver<f64>,
    transport_playing: bool,
    pending_position: bool,
}

fn notification_from_broadcast<T>(
    result: Result<Arc<T>, RecvError>,
    to_notification: impl for<'a> FnOnce(&'a T) -> Notification<'a>,
) -> Option<Option<Event>> {
    match result {
        Ok(data) => Some(notification_event(to_notification(&data))),
        Err(error) => broadcast_error(error),
    }
}

fn broadcast_error(error: RecvError) -> Option<Option<Event>> {
    if let RecvError::Lagged(count) = error {
        tracing::warn!(target: "web", count, "notification stream receiver lagged; disconnecting");
    }
    None
}

fn notification_event(notification: Notification<'_>) -> Option<Event> {
    tracing::trace!(target: "notif", ?notification);
    match serde_json::to_string(&notification) {
        Ok(json) => Some(Event::default().data(json)),
        Err(error) => {
            tracing::error!(target: "web", ?error, "failed to serialize notification");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn notification_is_framed_as_an_sse_event() {
        let event = notification_event(Notification::TransportPosition(42.0)).unwrap();
        let response = Sse::new(stream::iter([Ok::<_, Infallible>(event)])).into_response();
        assert_eq!(response.headers()["content-type"], "text/event-stream");
        let mut body = response.into_body().into_data_stream();
        let bytes = body.next().await.unwrap().unwrap();
        assert_eq!(&bytes[..], b"data: {\"type\":\"TransportPosition\",\"data\":42.0}\n\n");
    }

    #[test]
    fn openapi_describes_notification_stream() {
        let openapi = serde_json::to_value(crate::server::api_router(".").into_openapi()).unwrap();
        let response = &openapi["paths"]["/api/notifications"]["get"]["responses"]["200"];
        assert_eq!(response["content"]["text/event-stream"]["schema"]["type"], "string");
        assert!(openapi["components"]["schemas"]["Notification"].is_object());
    }
}
