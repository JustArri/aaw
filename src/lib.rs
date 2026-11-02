use crate::{
    engine::EngineHandle,
    midi::{MidiEvent, MidiRuntime},
    rt_channels::NotifiedRtConsumer,
};
use serde::Serialize;
use serde_json::{Serializer, ser::PrettyFormatter};
use std::{sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{Mutex, broadcast},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub mod app_state;
pub mod assets;
pub mod audio;
pub mod engine;
pub mod ffi;
pub mod files;
pub mod memory;
pub mod midi;
pub mod projects;
pub mod rt_channels;
pub mod server;
pub mod synths;
pub mod system;

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("APP_VERSION");

pub const AAW_ROOT_DIR_ENV: &str = "AAW_ROOT_DIR";
pub const CFG_AUDIO_OUTPUT: (&str, &str) = ("config/audio-output.json", "audio output config");
pub const CFG_LAST_PROJECT_SUBJECT: &str = "last project";
pub const CFG_LAST_PROJECT: (&str, &str) = ("config/last-project.json", CFG_LAST_PROJECT_SUBJECT);
pub const CFG_MIDI_AC: (&str, &str) =
    ("config/midi-auto-connections.json", "midi auto-connections");
pub const CFG_PREFERENCES: (&str, &str) = ("config/preferences.json", "preferences");
pub const PROJECTS_DIR: &str = "projects";
pub const ASSETS_DIR: &str = "assets";
pub const RECORDINGS_DIR: &str = "recordings";

pub const DEFAULT_CHANNEL_COUNT: usize = 2;
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;
pub const MAX_BLOCK_SIZE: usize = 8192;

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

pub fn spawn_midi_relay_task(
    stop: CancellationToken,
    engine_handle: EngineHandle,
    mut midi_rx: NotifiedRtConsumer<MidiEvent>,
    midi_tx: broadcast::Sender<Arc<MidiEvent>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop.cancelled() => break,

                event = midi_rx.recv() => {
                    info!(target: "midi-event", "{event:?}");
                    engine_handle.process_midi_event(event).await;
                    let _ = midi_tx.send(Arc::new(event));
                }
            }
        }
    })
}

pub fn to_pretty_string<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let mut buf = Vec::new();
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut serializer = Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut serializer)?;
    Ok(String::from_utf8_lossy(&buf).into())
}
