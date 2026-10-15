use crate::{
    engine::{self, Command, dummy_graph, fluid_graph, rt, sfizz_graph, synth_graph}, midi::MidiEvent, rt_channels::{NotifiedRtSharedProducer, RtConsumer},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, sync::{
        Arc, atomic::{AtomicBool, AtomicU32, Ordering},
    }, time::Duration,
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

    #[error("engine paused, command discarded")]
    EnginePaused,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Project {
    pub synth_graph: synth_graph::ctr::Graph,
    pub dummy_graph: dummy_graph::ctr::Graph,
    pub fluid_graph: fluid_graph::ctr::Graph,
    pub sfizz_graph: sfizz_graph::ctr::Graph,
}

#[derive(Clone)]
pub struct EngineHandle {
    pub cmd_tx: NotifiedRtSharedProducer<(u32, Command)>,
    pub pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
    pub midi_tx: NotifiedRtSharedProducer<MidiEvent>,
    pub next_cmd_id: Arc<AtomicU32>,
    pub project: Arc<RwLock<Project>>,
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

    pub async fn save_project(&self) -> engine::stored::Project {
        (&*self.project.read().await).into()
    }

    pub async fn replace_project(
        &self,
        ctr: engine::ctr::Project,
        rt: engine::rt::Project,
    ) -> Result<(), EngineError> {
        self.command(Command::ReplaceProject { project: Box::new(rt) }).await?;
        self.pending_replies.lock().await.clear();
        *self.project.write().await = ctr;
        Ok(())
    }
}

pub struct Runtime {
    pub pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
    pub reply_rx: RtConsumer<(u32, bool)>,
    pub stream_error: Arc<AtomicBool>,
    pub cmd_notify: Arc<tokio::sync::Notify>,
    pub midi_notify: Arc<tokio::sync::Notify>,
    pub engine: Arc<Mutex<rt::Engine>>,
}

impl Runtime {
    pub async fn process_replies(&mut self) {
        while let Ok((cmd_id, reply)) = self.reply_rx.pop() {
            if let Some(reply_tx) = self.pending_replies.lock().await.remove(&cmd_id) {
                let _ = reply_tx.send(reply);
            }
        }
    }

    pub fn stream_error(&self) -> bool {
        self.stream_error.load(Ordering::Relaxed)
    }

    pub fn cmd_notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.cmd_notify.notified()
    }

    pub fn midi_notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.midi_notify.notified()
    }
}
