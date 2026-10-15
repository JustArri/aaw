use crate::{
    engine::{
        self, Command, dummy_graph, fluid_graph,
        graph::LocalId,
        instrument_graph::{self, SynthId},
        rt, sfizz_graph,
    },
    midi::{MidiEvent, MidiFilter, MidiKeyTransform},
    rt_channels::{NotifiedRtSharedProducer, RtConsumer},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
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

    #[error("engine paused, command discarded")]
    EnginePaused,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Project {
    pub instrument_graph: instrument_graph::ctr::Graph,
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
        let mut project = self.project.write().await;

        if !self.command(Command::ReplaceProject { project: Box::new(rt) }).await? {
            return Err(EngineError::Rejected);
        }

        self.pending_replies.lock().await.clear();
        *project = ctr;
        Ok(())
    }

    pub async fn set_global_transposition(&self, transposition: i8) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetGlobalTransposition { transposition }).await? {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.set_global_transposition(transposition);
        Ok(())
    }

    pub async fn instrument_slot_append_new(&self) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrumentSlotAppendNew).await? {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.append_new();
        Ok(())
    }

    pub async fn instrument_slot_remove(&self, slot_id: LocalId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrumentSlotRemove { slot_id }).await? {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.remove(slot_id);
        Ok(())
    }

    pub async fn set_synth(
        &self,
        slot_id: LocalId,
        synth_id: Option<SynthId>,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetSynth { slot_id, synth_id }).await? {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.set_synth(slot_id, synth_id);
        Ok(())
    }

    pub async fn set_enabled(&self, slot_id: LocalId, flag: bool) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetEnabled { slot_id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.set_enabled(slot_id, flag);
        Ok(())
    }

    pub async fn set_key_transform(
        &self,
        slot_id: LocalId,
        transform: MidiKeyTransform,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self
            .command(Command::SetKeyTransform { slot_id, transform: transform.compile() })
            .await?
        {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.set_key_transform(slot_id, transform);
        Ok(())
    }

    pub async fn set_channel_filter(
        &self,
        slot_id: LocalId,
        filter: MidiFilter,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetChannelFilter { slot_id, mask: filter.as_u16() }).await? {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.set_channel_filter(slot_id, filter);
        Ok(())
    }

    pub async fn set_cc_filter(
        &self,
        slot_id: LocalId,
        filter: MidiFilter,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self
            .command(Command::SetControlChangeFilter { slot_id, mask: filter.as_u128() })
            .await?
        {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.set_cc_filter(slot_id, filter);
        Ok(())
    }

    pub async fn set_use_global_transposition(
        &self,
        slot_id: LocalId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetUseGlobalTransposition { slot_id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        project.instrument_graph.set_use_global_transposition(slot_id, flag);
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
