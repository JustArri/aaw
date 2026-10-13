use crate::{
    engine::{self, Command, dummy_graph, fluid_graph, sfizz_graph, synth_graph}, midi::MidiEvent, rt_channels::{RtConsumer, RtSharedProducer},
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
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("command rejected")]
    Rejected,

    #[error("command queue full")]
    QueueFull,

    #[error("command timed out")]
    Timeout,
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
    pub reply_buffer: Arc<RwLock<HashMap<u32, bool>>>,
    pub midi_tx: RtSharedProducer<MidiEvent>,
    pub next_cmd_id: Arc<AtomicU32>,
    pub engine: Arc<RwLock<Engine>>,
}

impl EngineHandle {
    pub async fn command(&self, command: Command) -> Result<bool, EngineError> {
        let cmd_id = self.next_cmd_id.fetch_add(1, Ordering::Relaxed);
        self.cmd_tx.push((cmd_id, command)).await.map_err(|_| EngineError::QueueFull)?;
        let result = self.wait_for_reply(cmd_id).await;
        Ok(result)
    }

    pub async fn project(&self) -> engine::stored::Project {
        (&*self.engine.read().await).into()
    }

    async fn wait_for_reply(&self, cmd_id: u32) -> bool {
        loop {
            if self.reply_buffer.read().await.contains_key(&cmd_id)
                && let Some(result) = self.reply_buffer.write().await.remove(&cmd_id)
            {
                return result;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

pub struct Runtime {
    pub reply_buffer: Arc<RwLock<HashMap<u32, bool>>>,
    pub reply_rx: RtConsumer<(u32, bool)>,
}

impl Runtime {
    pub async fn update(&mut self) {
        while let Ok((cmd_id, reply)) = self.reply_rx.pop() {
            self.reply_buffer.write().await.insert(cmd_id, reply);
        }
    }
}
