use crate::{
    engine::{self, Command, dummy_graph, fluid_graph, sfizz_graph, synth_graph},
    midi::MidiEvent,
    rt_channels::{NotifiedRtSharedProducer, RtConsumer, RtSharedProducer},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Mutex, RwLock, oneshot};
use utoipa::ToSchema;

const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("command rejected")]
    Rejected,

    #[error("command queue full")]
    QueueFull,

    #[error("command timed out")]
    Timeout,

    #[error("command reply channel closed")]
    ReplyChannelClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Engine {
    pub synth_graph: synth_graph::ctr::Graph,
    pub dummy_graph: dummy_graph::ctr::Graph,
    pub fluid_graph: fluid_graph::ctr::Graph,
    pub sfizz_graph: sfizz_graph::ctr::Graph,
}

pub struct EngineHandle {
    pub cmd_tx: RtSharedProducer<(u32, Command)>,
    pub pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
    pub midi_tx: NotifiedRtSharedProducer<MidiEvent>,
    pub next_cmd_id: Arc<AtomicU32>,
    pub engine: Arc<RwLock<Engine>>,
}

impl EngineHandle {
    pub async fn command(&self, command: Command) -> Result<bool, EngineError> {
        let cmd_id = self.next_cmd_id.fetch_add(1, Ordering::Relaxed);
        let (reply_tx, reply_rx) = oneshot::channel();

        self.pending_replies.lock().await.insert(cmd_id, reply_tx);

        if self.cmd_tx.push((cmd_id, command)).await.is_err() {
            self.pending_replies.lock().await.remove(&cmd_id);
            return Err(EngineError::QueueFull);
        }

        match tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await {
            Ok(Ok(reply)) => Ok(reply),
            Ok(Err(_)) => Err(EngineError::ReplyChannelClosed),
            Err(_) => {
                self.pending_replies.lock().await.remove(&cmd_id);
                Err(EngineError::Timeout)
            }
        }
    }

    pub async fn project(&self) -> engine::stored::Project {
        (&*self.engine.read().await).into()
    }
}

pub struct Runtime {
    pub pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
    pub reply_rx: RtConsumer<(u32, bool)>,
}

impl Runtime {
    pub async fn update(&mut self) {
        while let Ok((cmd_id, reply)) = self.reply_rx.pop() {
            if let Some(reply_tx) = self.pending_replies.lock().await.remove(&cmd_id) {
                let _ = reply_tx.send(reply);
            }
        }
    }
}
