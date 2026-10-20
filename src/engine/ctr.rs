use crate::{
    engine::{
        self, Command, dummy_graph, fluid_graph,
        graph::LocalId,
        instrument_graph::{self, SynthId, SynthKind},
        rt, sfizz_graph,
    },
    midi::{MidiEvent, MidiFilter, MidiKeyTransform, MidiVelocityTransform},
    rt_channels::{NotifiedRtSharedProducer, RtConsumer},
    server::api::notifications::{ProjectChange, ProjectChangeKind},
    synths::{self, SoundFontProgram},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};
use tokio::sync::{Mutex, RwLock, RwLockReadGuard, broadcast, oneshot};
use utoipa::ToSchema;

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
pub struct CtrProjectStorage {
    pub id: uuid::Uuid,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = Project)]
pub struct CtrProject {
    #[schema(required)]
    pub storage: Option<CtrProjectStorage>,

    pub revision: u32,
    pub name: String,
    pub instrument_graph: instrument_graph::CtrGraph,
    pub dummy_graph: dummy_graph::CtrGraph,
    pub fluid_graph: fluid_graph::CtrGraph,
    pub sfizz_graph: sfizz_graph::CtrGraph,
}

impl CtrProject {
    fn next_revision(&mut self) -> u32 {
        (self.revision, _) = self.revision.overflowing_add(1);
        self.revision
    }
}

#[derive(Clone)]
pub struct EngineHandle {
    cmd_tx: NotifiedRtSharedProducer<(u32, Command)>,
    pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
    midi_tx: NotifiedRtSharedProducer<MidiEvent>,
    next_cmd_id: Arc<AtomicU32>,
    project: Arc<RwLock<CtrProject>>,
    project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
}

impl EngineHandle {
    pub fn new(
        cmd_tx: NotifiedRtSharedProducer<(u32, Command)>,
        pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
        midi_tx: NotifiedRtSharedProducer<MidiEvent>,
        next_cmd_id: Arc<AtomicU32>,
        project: Arc<RwLock<CtrProject>>,
        project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
    ) -> Self {
        Self { cmd_tx, pending_replies, midi_tx, next_cmd_id, project, project_change_tx }
    }

    pub fn midi_sender(&self) -> NotifiedRtSharedProducer<MidiEvent> {
        self.midi_tx.clone()
    }

    pub async fn project(&self) -> RwLockReadGuard<'_, CtrProject> {
        self.project.read().await
    }

    pub fn subscribe_project_changes(&self) -> broadcast::Receiver<Arc<ProjectChange>> {
        self.project_change_tx.subscribe()
    }

    pub async fn silence(&self) -> Result<(), EngineError> {
        if !self.command(Command::Silence).await? {
            return Err(EngineError::Rejected);
        }

        Ok(())
    }

    pub async fn change_project_storage(&self, storage: Option<CtrProjectStorage>) {
        let mut project = self.project.write().await;
        project.storage = storage;
        let revision = project.revision;
        self.broadcast_project_change(revision, ProjectChangeKind::Storage);
    }

    pub async fn set_project_name(&self, name: String) {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.name = name;
        self.broadcast_project_change(revision, ProjectChangeKind::Name);
    }

    pub async fn save_project(&self) -> engine::StoredProject {
        (&*self.project.read().await).into()
    }

    pub async fn replace_project(
        &self,
        ctr: engine::CtrProject,
        rt: engine::RtProject,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::ReplaceProject { project: Box::new(rt) }).await? {
            return Err(EngineError::Rejected);
        }

        self.pending_replies.lock().await.clear();
        *project = ctr;
        self.broadcast_project_change(project.revision, ProjectChangeKind::Full);
        Ok(())
    }

    pub async fn dummy_push(
        &self,
        ctr: dummy_graph::CtrSynth,
        rt: synths::Dummy,
    ) -> Result<LocalId, EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::DummyPush { synth: rt }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        let id = project.dummy_graph.push(ctr);
        self.broadcast_project_change(revision, ProjectChangeKind::DummySynth { id });
        Ok(id)
    }

    pub async fn dummy_remove(&self, synth_id: LocalId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::DummyRemove { synth_id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.dummy_graph.remove(synth_id);
        project
            .instrument_graph
            .remove_synth_references(SynthId { kind: SynthKind::Dummy, id: synth_id });
        self.broadcast_project_change(revision, ProjectChangeKind::DummySynth { id: synth_id });
        Ok(())
    }

    pub async fn dummy_set_name(&self, synth_id: LocalId, name: String) -> Result<(), EngineError> {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.dummy_graph.set_name(synth_id, name);
        self.broadcast_project_change(revision, ProjectChangeKind::DummySynthName { id: synth_id });
        Ok(())
    }

    pub async fn fluid_push(
        &self,
        ctr: fluid_graph::CtrSynth,
        rt: synths::Fluid,
    ) -> Result<LocalId, EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::FluidPush { synth: rt }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        let id = project.fluid_graph.push(ctr);
        self.broadcast_project_change(revision, ProjectChangeKind::FluidSynth { id });
        Ok(id)
    }

    pub async fn fluid_replace(
        &self,
        synth_id: LocalId,
        ctr: fluid_graph::CtrSynth,
        rt: synths::Fluid,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::FluidReplace { synth_id, synth: rt }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.fluid_graph.replace(synth_id, ctr);
        self.broadcast_project_change(revision, ProjectChangeKind::FluidSynth { id: synth_id });
        Ok(())
    }

    pub async fn fluid_remove(&self, synth_id: LocalId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::FluidRemove { synth_id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.fluid_graph.remove(synth_id);
        project
            .instrument_graph
            .remove_synth_references(SynthId { kind: SynthKind::Fluid, id: synth_id });
        self.broadcast_project_change(revision, ProjectChangeKind::FluidSynth { id: synth_id });
        Ok(())
    }

    pub async fn fluid_select_program(
        &self,
        synth_id: LocalId,
        program: SoundFontProgram,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::FluidSelectProgram { synth_id, program }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.fluid_graph.select_program(synth_id, program);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::FluidSynthProgram { id: synth_id },
        );
        Ok(())
    }

    pub async fn fluid_set_name(&self, synth_id: LocalId, name: String) -> Result<(), EngineError> {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.fluid_graph.set_name(synth_id, name);
        self.broadcast_project_change(revision, ProjectChangeKind::FluidSynthName { id: synth_id });
        Ok(())
    }

    pub async fn sfizz_push(
        &self,
        ctr: sfizz_graph::CtrSynth,
        rt: synths::Sfizz,
    ) -> Result<LocalId, EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SfizzPush { synth: rt }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        let id = project.sfizz_graph.push(ctr);
        self.broadcast_project_change(revision, ProjectChangeKind::SfizzSynth { id });
        Ok(id)
    }

    pub async fn sfizz_replace(
        &self,
        synth_id: LocalId,
        ctr: sfizz_graph::CtrSynth,
        rt: synths::Sfizz,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SfizzReplace { synth_id, synth: rt }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.sfizz_graph.replace(synth_id, ctr);
        self.broadcast_project_change(revision, ProjectChangeKind::SfizzSynth { id: synth_id });
        Ok(())
    }

    pub async fn sfizz_remove(&self, synth_id: LocalId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SfizzRemove { synth_id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.sfizz_graph.remove(synth_id);
        project
            .instrument_graph
            .remove_synth_references(SynthId { kind: SynthKind::Sfizz, id: synth_id });
        self.broadcast_project_change(revision, ProjectChangeKind::SfizzSynth { id: synth_id });
        Ok(())
    }

    pub async fn sfizz_set_name(&self, synth_id: LocalId, name: String) -> Result<(), EngineError> {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.sfizz_graph.set_name(synth_id, name);
        self.broadcast_project_change(revision, ProjectChangeKind::SfizzSynthName { id: synth_id });
        Ok(())
    }

    pub async fn set_global_transposition(&self, transposition: i8) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetGlobalTransposition { transposition }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_global_transposition(transposition);
        self.broadcast_project_change(revision, ProjectChangeKind::GlobalTransposition);
        Ok(())
    }

    pub async fn instrument_slot_append_new(&self) -> Result<LocalId, EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrumentSlotAppendNew).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        let id = project.instrument_graph.append_new();
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentSlot { id });
        Ok(id)
    }

    pub async fn instrument_slot_remove(&self, slot_id: LocalId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrumentSlotRemove { slot_id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.remove(slot_id);
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentSlot { id: slot_id });
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

        let revision = project.next_revision();
        project.instrument_graph.set_synth(slot_id, synth_id);
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentSynth { id: slot_id });
        Ok(())
    }

    pub async fn set_enabled(&self, slot_id: LocalId, flag: bool) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetEnabled { slot_id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_enabled(slot_id, flag);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentEnabled { id: slot_id },
        );
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

        let revision = project.next_revision();
        project.instrument_graph.set_key_transform(slot_id, transform);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentKeyTransform { id: slot_id },
        );
        Ok(())
    }

    pub async fn set_velocity_transform(
        &self,
        slot_id: LocalId,
        transform: MidiVelocityTransform,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self
            .command(Command::SetVelocityTransform { slot_id, transform: transform.compile() })
            .await?
        {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_velocity_transform(slot_id, transform);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentVelocityTransform { id: slot_id },
        );
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

        let revision = project.next_revision();
        project.instrument_graph.set_channel_filter(slot_id, filter);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentChannelFilter { id: slot_id },
        );
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

        let revision = project.next_revision();
        project.instrument_graph.set_cc_filter(slot_id, filter);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentCcFilter { id: slot_id },
        );
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

        let revision = project.next_revision();
        project.instrument_graph.set_use_global_transposition(slot_id, flag);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentUseGlobalTransposition { id: slot_id },
        );
        Ok(())
    }

    async fn command(&self, command: Command) -> Result<bool, EngineError> {
        let cmd_id = self.next_cmd_id.fetch_add(1, Ordering::Relaxed);
        let (reply_tx, reply_rx) = oneshot::channel();

        self.pending_replies.lock().await.insert(cmd_id, reply_tx);

        if self.cmd_tx.push((cmd_id, command)).await.is_err() {
            self.pending_replies.lock().await.remove(&cmd_id);
            return Err(EngineError::QueueFull);
        }

        // match tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await {
        //     Ok(Ok(reply)) => Ok(reply),
        //     Ok(Err(_)) => Err(EngineError::ReplyChannelClosed),
        //     Err(_) => {
        //         self.pending_replies.lock().await.remove(&cmd_id);
        //         Err(EngineError::Timeout)
        //     }
        // }

        match reply_rx.await {
            Ok(reply) => Ok(reply),
            Err(_) => Err(EngineError::ReplyChannelClosed),
        }
    }

    fn broadcast_project_change(&self, revision: u32, kind: ProjectChangeKind) {
        let change = ProjectChange { revision, kind };
        let _ = self.project_change_tx.send(Arc::new(change));
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
