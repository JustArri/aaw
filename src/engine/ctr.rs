use crate::{
    engine::{
        Command, Engine,
        effects::{Effects, RtEffects},
        graph::{InstrumentId, SynthId},
        project::{CtrProject, CtrProjectStorage, RtProject, StoredProject},
        synth_graph::{CtrSynth, CtrSynthParameters, RtSynth, RtSynthKind},
        user_presets::CtrUserPresets,
    },
    midi::{MidiEvent, MidiFilter, MidiKeyTransform, MidiVelocityTransform},
    rt_channels::{NotifiedRtSharedProducer, RtConsumer},
    server::api::notifications::{ProjectChange, ProjectChangeKind},
    synths::SoundFontProgram,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};
use tokio::sync::{Mutex, RwLock, RwLockReadGuard, broadcast, oneshot};

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
        project.storage = storage.clone();
        let revision = project.revision;

        self.broadcast_project_change(revision, ProjectChangeKind::Storage { storage });
    }

    pub async fn set_project_name(&self, name: String) {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.name = name.clone();

        self.broadcast_project_change(revision, ProjectChangeKind::Name { name });
    }

    pub async fn save_project(&self) -> StoredProject {
        (&*self.project.read().await).into()
    }

    pub async fn replace_project(&self, ctr: CtrProject, rt: RtProject) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::ReplaceProject { project: Box::new(rt) }).await? {
            return Err(EngineError::Rejected);
        }

        self.pending_replies.lock().await.clear();
        *project = ctr;

        self.broadcast_project_change(project.revision, ProjectChangeKind::Full);

        Ok(())
    }

    pub async fn set_effects(
        &self,
        effects: Effects,
        rt_effects: RtEffects,
    ) -> Result<(), EngineError> {
        if !self.command(Command::SetEffects { effects: rt_effects }).await? {
            return Err(EngineError::Rejected);
        }

        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.synth_graph.set_effects(effects.clone());

        self.broadcast_project_change(revision, ProjectChangeKind::Effects { effects });

        Ok(())
    }

    pub async fn synth_push(&self, ctr: CtrSynth, rt: RtSynth) -> Result<SynthId, EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SynthPush { synth: rt }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        let id = project.synth_graph.push(ctr);

        self.broadcast_project_change(revision, ProjectChangeKind::Synth { id, removed: false });

        Ok(id)
    }

    pub async fn synth_replace(
        &self,
        id: SynthId,
        parameters: CtrSynthParameters,
        kind: RtSynthKind,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SynthReplace { id, kind }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.synth_graph.replace(id, parameters.clone());

        self.broadcast_project_change(revision, ProjectChangeKind::SynthKind { id, parameters });

        Ok(())
    }

    pub async fn synth_remove(&self, id: SynthId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SynthRemove { id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.synth_graph.remove(id);
        project.instrument_graph.remove_synth_references(id);

        self.broadcast_project_change(revision, ProjectChangeKind::Synth { id, removed: true });

        Ok(())
    }

    pub async fn synth_select_soundfont_program(
        &self,
        id: SynthId,
        program: SoundFontProgram,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SynthSelectSoundfontProgram { id, program }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.synth_graph.synth_select_soundfont_program(id, program);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::SynthSoundfontProgram { id, program },
        );

        Ok(())
    }

    pub async fn synth_set_name(&self, id: SynthId, name: String) -> Result<(), EngineError> {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.synth_graph.synth_set_name(id, name.clone());

        self.broadcast_project_change(revision, ProjectChangeKind::SynthName { id, name });

        Ok(())
    }

    pub async fn synth_set_effects(
        &self,
        id: SynthId,
        effects: Effects,
        rt_effects: RtEffects,
    ) -> Result<(), EngineError> {
        if !self.command(Command::SynthSetEffects { id, effects: rt_effects }).await? {
            return Err(EngineError::Rejected);
        }

        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.synth_graph.synth_set_effects(id, effects.clone());

        self.broadcast_project_change(revision, ProjectChangeKind::SynthEffects { id, effects });

        Ok(())
    }

    pub async fn set_global_transposition(&self, transposition: i8) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetGlobalTransposition { transposition }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_global_transposition(transposition);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::GlobalTransposition { transposition },
        );

        Ok(())
    }

    pub async fn instrument_append_new(&self) -> Result<InstrumentId, EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrAppendNew).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        let id = project.instrument_graph.append_new();

        self.broadcast_project_change(revision, ProjectChangeKind::Instrument { id });

        Ok(id)
    }

    pub async fn instrument_remove(&self, id: InstrumentId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrRemove { id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.remove(id);

        self.broadcast_project_change(revision, ProjectChangeKind::Instrument { id });

        Ok(())
    }

    pub async fn instrument_set_synth(
        &self,
        id: InstrumentId,
        synth_id: Option<SynthId>,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetSynth { id, synth_id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_synth(id, synth_id);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentSynth { id, synth_id },
        );

        Ok(())
    }

    pub async fn instrument_set_enabled(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetEnabled { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_enabled(id, flag);

        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentEnabled { id, flag });

        Ok(())
    }

    pub async fn instrument_set_ignore_key_transform(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetIgnoreKeyTransform { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_ignore_key_transform(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentIgnoreKeyTransform { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_key_transform(
        &self,
        id: InstrumentId,
        transform: MidiKeyTransform,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self
            .command(Command::InstrSetKeyTransform { id, transform: transform.compile() })
            .await?
        {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_key_transform(id, transform.clone());

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentKeyTransform { id, transform },
        );

        Ok(())
    }

    pub async fn instrument_set_ignore_velocity_transform(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetIgnoreVelocityTransform { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_ignore_velocity_transform(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentIgnoreVelocityTransform { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_velocity_transform(
        &self,
        id: InstrumentId,
        transform: MidiVelocityTransform,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self
            .command(Command::InstrSetVelocityTransform { id, transform: transform.compile() })
            .await?
        {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_velocity_transform(id, transform.clone());

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentVelocityTransform { id, transform },
        );

        Ok(())
    }

    pub async fn instrument_set_channel_filter(
        &self,
        id: InstrumentId,
        filter: MidiFilter,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetChannelFilter { id, mask: filter.as_u16() }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_channel_filter(id, filter.clone());

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentChannelFilter { id, filter },
        );

        Ok(())
    }

    pub async fn instrument_set_cc_filter(
        &self,
        id: InstrumentId,
        filter: MidiFilter,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self
            .command(Command::InstrSetControlChangeFilter { id, mask: filter.as_u128() })
            .await?
        {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_cc_filter(id, filter.clone());

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentCcFilter { id, filter },
        );

        Ok(())
    }

    pub async fn instrument_set_ignore_pitch_bend(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetIgnorePitchBend { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_ignore_pitch_bend(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentIgnorePitchBend { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_ignore_program_change(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetIgnoreProgramChange { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_ignore_program_change(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentIgnoreProgramChange { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_ignore_channel_aftertouch(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetIgnoreChannelAftertouch { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_ignore_channel_aftertouch(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentIgnoreChannelAftertouch { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_ignore_polyphonic_aftertouch(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetIgnorePolyphonicAftertouch { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_ignore_polyphonic_aftertouch(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentIgnorePolyphonicAftertouch { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_use_global_transposition(
        &self,
        id: InstrumentId,
        flag: bool,
    ) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrSetUseGlobalTransposition { id, flag }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = project.next_revision();
        project.instrument_graph.set_use_global_transposition(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentUseGlobalTransposition { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_name(&self, id: InstrumentId, name: String) {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.instrument_graph.set_name(id, name.clone());
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentName { id, name });
    }

    pub async fn set_user_presets(&self, presets: CtrUserPresets) {
        let mut project = self.project.write().await;
        let revision = project.next_revision();
        project.user_presets = presets;

        self.broadcast_project_change(revision, ProjectChangeKind::UserPresets);
    }

    // pub async fn activate_user_preset(&self, instrument_ids: &[LocalId]) -> Result<(), EngineError> {
    //     let project = self.project.read().await;

    //     let Some(preset) = project.user_presets.entries.get(id) else {
    //         return Err(EngineError::Rejected);
    //     };

    //     for id in project.instrument_graph.instruments.ids() {
    //         let enabled = preset.instruments.contains(&(*id).into());
    //         self.instrument_set_enabled(*id, enabled).await?;
    //     }

    //     Ok(())
    // }

    async fn command(&self, command: Command) -> Result<bool, EngineError> {
        let cmd_id = self.next_cmd_id.fetch_add(1, Ordering::Relaxed);
        let (reply_tx, reply_rx) = oneshot::channel();

        self.pending_replies.lock().await.insert(cmd_id, reply_tx);

        if self.cmd_tx.push((cmd_id, command)).await.is_err() {
            self.pending_replies.lock().await.remove(&cmd_id);
            return Err(EngineError::QueueFull);
        }

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
    pub engine: Arc<Mutex<Engine>>,
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
