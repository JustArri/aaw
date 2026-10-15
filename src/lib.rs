use crate::{
    midi::{ControlChange, MidiEvent, MidiEventKind, MidiRuntime},
    rt_channels::NotifiedRtConsumer,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{Mutex, broadcast},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub mod audio;
pub mod effects;
pub mod engine;
pub mod ffi;
pub mod memory;
pub mod midi;
pub mod rt_channels;
pub mod server;
pub mod synths;
pub mod system;

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("APP_VERSION");

pub fn subscribe_tracing(default_filter: &str) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("AAW_LOG")
                .unwrap_or_else(|_| default_filter.into()),
        )
        .init();
}

pub fn spawn_web_server_task(
    stop: CancellationToken,
    listener: TcpListener,
    root_router: axum::Router,
) -> JoinHandle<Result<(), std::io::Error>> {
    tokio::spawn(async move {
        axum::serve(listener, root_router)
            .with_graceful_shutdown(async move {
                let _ = stop.cancelled().await;
            })
            .await
    })
}

pub fn spawn_midi_runtime_task(
    stop: CancellationToken,
    midi_runtime: Arc<Mutex<MidiRuntime>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if stop.is_cancelled() {
                break;
            }

            if let Ok(midi_runtime) = &mut midi_runtime.try_lock() {
                if let Err(err) = midi_runtime.refresh_input_ports() {
                    error!(%err, "failed to refresh MIDI input ports");
                } else if let Err(err) = midi_runtime.update_input_connections() {
                    error!(%err, "failed to update MIDI input connections");
                }
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    })
}

pub fn spawn_engine_runtime_task(
    stop: CancellationToken,
    mut runtime: engine::ctr::Runtime,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = stop.cancelled() => break,

                _ = interval.tick() => runtime.process_replies().await,

                _ = runtime.cmd_notified(), if runtime.stream_error() => {
                    tracing::info!(target: "engine-runtime", "idle processing commands");
                    runtime.engine.lock().await.idle_process_commands();
                }

                _ = runtime.midi_notified(), if runtime.stream_error() => {
                    tracing::info!(target: "engine-runtime", "idle processing midi events");
                    runtime.engine.lock().await.idle_process_midi_events();
                }
            }
        }
    })
}

pub fn spawn_midi_relay_task(
    stop: CancellationToken,
    mut midi_rx: NotifiedRtConsumer<MidiEvent>,
    midi_tx: broadcast::Sender<Arc<MidiEvent>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop.cancelled() => break,

                event = midi_rx.recv() => {
                    info!(target: "midi-event", "{event:?}");

                    if let MidiEventKind::ControlChange { controller, .. } = event.kind
                        && controller == ControlChange::AllNotesOff as u8
                    {
                        info!("[midi panic]");
                        stop.cancel();
                        break;
                    }

                    let _ = midi_tx.send(Arc::new(event));
                }
            }
        }
    })
}
