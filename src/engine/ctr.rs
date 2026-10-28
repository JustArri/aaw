use crate::{
    MAX_BLOCK_SIZE,
    audio::AudioRecordingChunk,
    engine::{
        Command, Engine,
        effects::{Effects, RtEffects},
        graph::{InstrumentId, SynthId},
        instrument_graph::CtrInstrument,
        project::{CtrProject, CtrProjectStorage, RtProject, StoredProject},
        synth_graph::{CtrSynth, CtrSynthParameters, CtrSynthProgramChange, RtSynth, RtSynthKind},
        user_presets::CtrUserPresets,
    },
    midi::{ControlChange, MidiEvent, MidiFilter, MidiKeyTransform, MidiVelocityTransform},
    rt_channels::{NotifiedRtSharedProducer, RtConsumer},
    server::api::notifications::{ProjectChange, ProjectChangeKind},
    synths::SoundFontProgram,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::SeekFrom,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Duration,
};
use tokio::{
    fs::File,
    io::{self, AsyncSeekExt, AsyncWriteExt},
    sync::{Mutex, RwLock, RwLockReadGuard, broadcast, oneshot},
};
use tokio_util::sync::CancellationToken;
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

#[derive(Clone)]
pub struct EngineHandle {
    pub(super) cmd_tx: NotifiedRtSharedProducer<(u32, Command)>,
    pub(super) pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
    pub(super) midi_tx: NotifiedRtSharedProducer<MidiEvent>,
    pub(super) next_cmd_id: Arc<AtomicU32>,
    pub(super) project: Arc<RwLock<CtrProject>>,
    pub(super) project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
    pub(super) audio_rec_state: Arc<RwLock<AudioRecordingState>>,
    pub(super) audio_rec_params: Arc<Mutex<AudioRecordingParams>>,
}

impl EngineHandle {
    pub fn midi_sender(&self) -> NotifiedRtSharedProducer<MidiEvent> {
        self.midi_tx.clone()
    }

    pub async fn project(&self) -> RwLockReadGuard<'_, CtrProject> {
        self.project.read().await
    }

    pub async fn audio_recording_state(&self) -> AudioRecordingState {
        *self.audio_rec_state.read().await
    }

    pub async fn audio_recording_parameters(&self) -> AudioRecordingParams {
        self.audio_rec_params.lock().await.clone()
    }

    pub fn subscribe_project_changes(&self) -> broadcast::Receiver<Arc<ProjectChange>> {
        self.project_change_tx.subscribe()
    }

    pub async fn start_recording(
        &self,
        sample_rate: u32,
        file_path: PathBuf,
    ) -> Result<(), EngineError> {
        if *self.audio_rec_state.read().await != AudioRecordingState::Stopped {
            return Err(EngineError::Rejected);
        }

        let mut params = self.audio_rec_params.lock().await;
        params.sample_rate = sample_rate;
        params.file_path = file_path;
        drop(params);

        *self.audio_rec_state.write().await = AudioRecordingState::Running;

        Ok(())
    }

    pub async fn set_recording_paused(&self, flag: bool) -> Result<(), EngineError> {
        if *self.audio_rec_state.read().await == AudioRecordingState::Stopped {
            return Err(EngineError::Rejected);
        }

        *self.audio_rec_state.write().await =
            if flag { AudioRecordingState::Paused } else { AudioRecordingState::Running };

        Ok(())
    }

    pub async fn stop_recording(&self) -> Result<(), EngineError> {
        if *self.audio_rec_state.read().await == AudioRecordingState::Stopped {
            return Err(EngineError::Rejected);
        }

        *self.audio_rec_state.write().await = AudioRecordingState::Stopped;

        Ok(())
    }

    pub async fn silence(&self) -> Result<(), EngineError> {
        if !self.command(Command::Silence).await? {
            return Err(EngineError::Rejected);
        }

        Ok(())
    }

    pub async fn handle_control_change(&self, channel: u8, controller: u8, value: u8) {
        let filter_fn = |instrument: &CtrInstrument| -> bool {
            instrument.channel_filter.passes(channel) && instrument.cc_filter.passes(controller)
        };

        let project = &mut *self.project.write().await;

        project.instrument_graph.synth_iter(filter_fn).for_each(|id| {
            project.synth_graph.save_controller_value(id, controller, value);
        });
    }

    pub async fn handle_program_change(&self, channel: u8, program: u8) {
        let filter_fn = |instrument: &CtrInstrument| -> bool {
            !instrument.ignore_program_change && instrument.channel_filter.passes(channel)
        };

        let project = &mut *self.project.write().await;

        project.instrument_graph.synth_iter(filter_fn).for_each(|id| {
            if let Some(change) = project.synth_graph.change_program(id, program) {
                let revision = CtrProject::next_revision(&mut project.revision);
                match change {
                    CtrSynthProgramChange::SoundFont(program) => self.broadcast_project_change(
                        revision,
                        ProjectChangeKind::SynthSoundfontProgram { id, program },
                    ),
                }
            }
        });
    }

    pub async fn handle_bank_lsb_change(&self, channel: u8, bank_lsb: u8) {
        let filter_fn = |instrument: &CtrInstrument| -> bool {
            instrument.channel_filter.passes(channel)
                && instrument.cc_filter.passes(ControlChange::BankSelectLsb as u8)
        };

        let project = &mut *self.project.write().await;

        project.instrument_graph.synth_iter(filter_fn).for_each(|id| {
            if let Some(change) = project.synth_graph.change_bank_lsb(id, bank_lsb) {
                let revision = CtrProject::next_revision(&mut project.revision);
                match change {
                    CtrSynthProgramChange::SoundFont(program) => self.broadcast_project_change(
                        revision,
                        ProjectChangeKind::SynthSoundfontProgram { id, program },
                    ),
                }
            }
        });
    }

    pub async fn change_project_storage(&self, storage: Option<CtrProjectStorage>) {
        let mut project = self.project.write().await;
        project.storage = storage.clone();
        let revision = project.revision;

        self.broadcast_project_change(revision, ProjectChangeKind::Storage { storage });
    }

    pub async fn set_project_name(&self, name: String) {
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
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

    pub async fn set_global_effects(
        &self,
        effects: Effects,
        rt_effects: RtEffects,
    ) -> Result<(), EngineError> {
        if !self.command(Command::SetGlobalEffects { effects: rt_effects }).await? {
            return Err(EngineError::Rejected);
        }

        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.synth_graph.set_global_effects(effects.clone());

        self.broadcast_project_change(revision, ProjectChangeKind::GlobalEffects { effects });

        Ok(())
    }

    pub async fn set_global_gain(&self, gain: f32) -> Result<(), EngineError> {
        if !self.command(Command::SetGlobalGain { gain }).await? {
            return Err(EngineError::Rejected);
        }

        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.synth_graph.set_global_gain(gain);

        self.broadcast_project_change(revision, ProjectChangeKind::GlobalGain { gain });

        Ok(())
    }

    pub async fn synth_push(&self, ctr: CtrSynth, rt: RtSynth) -> Result<SynthId, EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SynthPush { synth: rt }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
        project.synth_graph.replace(id, parameters.clone());

        self.broadcast_project_change(revision, ProjectChangeKind::SynthKind { id, parameters });

        Ok(())
    }

    pub async fn synth_remove(&self, id: SynthId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SynthRemove { id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
        project.synth_graph.synth_select_soundfont_program(id, program);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::SynthSoundfontProgram { id, program },
        );

        Ok(())
    }

    pub async fn synth_set_name(&self, id: SynthId, name: String) -> Result<(), EngineError> {
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
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
        let revision = CtrProject::next_revision(&mut project.revision);
        project.synth_graph.synth_set_effects(id, effects.clone());

        self.broadcast_project_change(revision, ProjectChangeKind::SynthEffects { id, effects });

        Ok(())
    }

    pub async fn synth_set_gain(&self, id: SynthId, gain: f32) -> Result<(), EngineError> {
        if !self.command(Command::SynthSetGain { id, gain }).await? {
            return Err(EngineError::Rejected);
        }

        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.synth_graph.set_synth_gain(id, gain);

        self.broadcast_project_change(revision, ProjectChangeKind::SynthGain { id, gain });

        Ok(())
    }

    pub async fn set_global_transposition(&self, transposition: i8) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::SetGlobalTransposition { transposition }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
        let id = project.instrument_graph.append_new();

        self.broadcast_project_change(revision, ProjectChangeKind::Instrument { id });

        Ok(id)
    }

    pub async fn instrument_remove(&self, id: InstrumentId) -> Result<(), EngineError> {
        let mut project = self.project.write().await;

        if !self.command(Command::InstrRemove { id }).await? {
            return Err(EngineError::Rejected);
        }

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
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

        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.set_use_global_transposition(id, flag);

        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentUseGlobalTransposition { id, flag },
        );

        Ok(())
    }

    pub async fn instrument_set_name(&self, id: InstrumentId, name: String) {
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.set_name(id, name.clone());
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentName { id, name });
    }

    pub async fn set_user_presets(&self, presets: CtrUserPresets) {
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
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

#[derive(Debug, Clone)]
pub struct AudioRecordingParams {
    pub sample_rate: u32,
    pub file_path: PathBuf,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum AudioRecordingState {
    Running,
    Paused,

    #[default]
    Stopped,
}

pub struct Runtime {
    pub(super) pending_replies: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,
    pub(super) reply_rx: RtConsumer<(u32, bool)>,
    pub(super) recording_rx: RtConsumer<AudioRecordingChunk>,

    pub(super) stream_error: Arc<AtomicBool>,
    pub(super) cmd_notify: Arc<tokio::sync::Notify>,
    pub(super) midi_notify: Arc<tokio::sync::Notify>,
    pub(super) engine: Arc<Mutex<Engine>>,

    pub(super) audio_rec_state: Arc<RwLock<AudioRecordingState>>,
    pub(super) audio_rec_params: Arc<Mutex<AudioRecordingParams>>,

    pub(super) audio_rec_capture_enabled: Arc<AtomicBool>,

    pub(super) audio_rec_active_sample_rate: u32,
    pub(super) audio_rec_file: Option<File>,
    pub(super) audio_rec_buffer: Vec<u8>,
    pub(super) audio_rec_bytes: u32,
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

    pub async fn start_recording(&mut self) -> io::Result<()> {
        // Keep the producer disabled during initialization.
        self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);

        // Discard chunks remaining from an earlier recording.
        while self.recording_rx.pop().is_ok() {}

        let params = self.audio_rec_params.lock().await.clone();

        if params.sample_rate == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid recording sample rate",
            ));
        }

        let header = wav_header(params.sample_rate, 0)?;

        let mut file = File::create(&params.file_path).await?;
        file.write_all(&header).await?;

        self.audio_rec_active_sample_rate = params.sample_rate;
        self.audio_rec_bytes = 0;
        self.audio_rec_buffer.clear();
        self.audio_rec_file = Some(file);

        self.audio_rec_capture_enabled.store(true, Ordering::Relaxed);

        Ok(())
    }

    pub async fn process_recording(&mut self) -> io::Result<()> {
        let Some(file) = self.audio_rec_file.as_mut() else {
            return Ok(());
        };

        while let Ok(chunk) = self.recording_rx.pop() {
            let count = chunk.block_size.min(MAX_BLOCK_SIZE);

            self.audio_rec_buffer.clear();

            for i in 0..count {
                let left = (chunk.left[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                let right = (chunk.right[i].clamp(-1.0, 1.0) * 32767.0) as i16;

                self.audio_rec_buffer.extend_from_slice(&left.to_le_bytes());
                self.audio_rec_buffer.extend_from_slice(&right.to_le_bytes());
            }

            let len = self.audio_rec_buffer.len() as u32;

            let new_size = self
                .audio_rec_bytes
                .checked_add(len)
                .filter(|&size| size <= u32::MAX - 36)
                .ok_or_else(|| io::Error::other("WAV size limit exceeded"))?;

            file.write_all(&self.audio_rec_buffer).await?;

            self.audio_rec_bytes = new_size;
        }

        Ok(())
    }

    pub async fn pause_recording(&mut self) -> io::Result<()> {
        self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);
        self.process_recording().await
    }

    pub fn resume_recording(&self) {
        self.audio_rec_capture_enabled.store(true, Ordering::Relaxed);
    }

    pub async fn stop_recording(&mut self) -> io::Result<()> {
        self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);

        self.process_recording().await?;

        let Some(mut file) = self.audio_rec_file.take() else {
            return Ok(());
        };

        file.seek(SeekFrom::Start(0)).await?;

        file.write_all(&wav_header(self.audio_rec_active_sample_rate, self.audio_rec_bytes)?)
            .await?;

        file.flush().await?;

        Ok(())
    }

    pub async fn update_recording(&mut self) -> io::Result<()> {
        let state = *self.audio_rec_state.read().await;

        match state {
            AudioRecordingState::Running => {
                if self.audio_rec_file.is_none() {
                    self.start_recording().await?;
                }

                self.resume_recording();
                self.process_recording().await?;
            }

            AudioRecordingState::Paused => {
                self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);

                if self.audio_rec_file.is_some() {
                    self.process_recording().await?;
                }
            }

            AudioRecordingState::Stopped => {
                if self.audio_rec_file.is_some() {
                    self.stop_recording().await?;
                }
            }
        }

        Ok(())
    }

    pub async fn into_future(mut self, stop: CancellationToken) -> io::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let cmd_notify = Arc::clone(&self.cmd_notify);
        let midi_notify = Arc::clone(&self.midi_notify);

        self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);

        let result = async {
            loop {
                tokio::select! {
                    _ = stop.cancelled() => break,

                    _ = interval.tick() => {
                        self.process_replies().await;
                        self.update_recording().await?;
                    }

                    _ = cmd_notify.notified(), if self.stream_error() => {
                        self.engine.lock().await.idle_process_commands();
                    }

                    _ = midi_notify.notified(), if self.stream_error() => {
                        self.engine.lock().await.idle_process_midi_events();
                    }
                }
            }

            Ok::<(), io::Error>(())
        }
        .await;

        let cleanup = self.stop_recording().await;
        result.and(cleanup)
    }
}

fn wav_header(sample_rate: u32, data_size: u32) -> tokio::io::Result<[u8; 44]> {
    let riff_size = data_size
        .checked_add(36)
        .ok_or_else(|| tokio::io::Error::other("WAV file size limit exceeded"))?;

    let channels = 2u16;
    let bits_per_sample = 16u16;
    let block_align = channels * bits_per_sample / 8;
    let byte_rate = sample_rate * block_align as u32;

    let mut header = [0u8; 44];

    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&riff_size.to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");

    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16u32.to_le_bytes());
    header[20..22].copy_from_slice(&1u16.to_le_bytes());
    header[22..24].copy_from_slice(&channels.to_le_bytes());
    header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    header[32..34].copy_from_slice(&block_align.to_le_bytes());
    header[34..36].copy_from_slice(&bits_per_sample.to_le_bytes());

    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&data_size.to_le_bytes());

    Ok(header)
}
