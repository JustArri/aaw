use crate::{
    MAX_BLOCK_SIZE,
    audio::AudioRecordingChunk,
    engine::{
        ClapParameterChange, Command, Engine, InstrumentMidiEvent,
        controller_graph::{RtController, RtControllerGraph, RtInstrumentControllers},
        drum_machine::{DrumMachineController, MusicalTransport, RtDrumMachine},
        effects::{Effects, RtEffects},
        graph::{ControllerAddress, ControllerId, InstrumentId},
        instrument_graph::{
            CtrInstrument, CtrInstrumentKind, CtrInstrumentProgramChange, RtInstrument,
            RtInstrumentKind,
        },
        midi_app_controls::{
            CtrMidiEngineControls, CtrMidiProjectControls, MidiAppAction, RtMidiEngineControls,
            RtMidiProjectControls,
        },
        project::{CtrProject, CtrProjectStorage, RtProject, StoredProject},
        user_presets::{CtrUserPresets, UserPresetAction},
    },
    midi::{
        ControlChange, MidiEvent, MidiEventKind, MidiFilter, MidiKeyTransform,
        MidiVelocityTransform,
    },
    rt_channels::{NotifiedRtSharedProducer, RtConsumer},
    server::api::notifications::{ProjectChange, ProjectChangeKind},
    synths::{self, SoundFontProgram},
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
    #[error(transparent)]
    Instrument(#[from] synths::Error),

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
    pub(super) midi_tx: NotifiedRtSharedProducer<InstrumentMidiEvent>,
    pub(super) next_cmd_id: Arc<AtomicU32>,
    pub(super) project: Arc<RwLock<CtrProject>>,
    pub(super) controller_graph: Arc<RwLock<RtControllerGraph>>,
    pub(super) ctr_midi_app_controls: Arc<RwLock<CtrMidiEngineControls>>,
    pub(super) rt_midi_app_controls: Arc<RwLock<RtMidiEngineControls>>,
    pub(super) project_midi_app_controls: Arc<RwLock<RtMidiProjectControls>>,
    pub(super) project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
    pub(super) midi_app_controls_change_tx: broadcast::Sender<Arc<CtrMidiEngineControls>>,
    pub(super) audio_rec_state: Arc<RwLock<AudioRecordingState>>,
    pub(super) audio_rec_params: Arc<Mutex<AudioRecordingParams>>,
}

impl EngineHandle {
    pub async fn process_midi_event(&self, event: MidiEvent) {
        let mut app_actions = Vec::new();
        self.rt_midi_app_controls.read().await.process(event, |action| app_actions.push(action));
        self.project_midi_app_controls
            .read()
            .await
            .process(event, |action| app_actions.push(action));

        let mut processed = Vec::new();
        self.controller_graph
            .read()
            .await
            .process(event, |instrument_id, event| processed.push((instrument_id, event)));

        for &(instrument_id, event) in &processed {
            let _ = self.midi_tx.push(InstrumentMidiEvent { instrument_id, event }).await;
        }

        let mut affected = Vec::new();
        for &(instrument_id, event) in &processed {
            if affected.contains(&instrument_id) {
                continue;
            }
            affected.push(instrument_id);
            match event.kind {
                MidiEventKind::ControlChange { controller, value } => {
                    self.remember_controller_value(instrument_id, controller, value).await;
                    if controller == ControlChange::BankSelectLsb as u8 {
                        self.apply_bank_change(instrument_id, value).await;
                    }
                }
                MidiEventKind::ProgramChange { program } => {
                    self.apply_program_change(instrument_id, program).await;
                }
                _ => {}
            }
        }

        for action in app_actions {
            self.execute_midi_app_action(action).await;
        }
    }

    async fn execute_midi_app_action(&self, action: MidiAppAction) {
        match action {
            MidiAppAction::ToggleTransport => {
                let mut transport = self.project.read().await.transport;
                transport.playing = !transport.playing;
                let _ = self.set_transport(transport).await;
            }
            MidiAppAction::SetTransportPlaying(playing) => {
                let mut transport = self.project.read().await.transport;
                if transport.playing != playing {
                    transport.playing = playing;
                    let _ = self.set_transport(transport).await;
                }
            }
            MidiAppAction::ToggleInstrumentController(address) => {
                let enabled = self
                    .project
                    .read()
                    .await
                    .instrument_graph
                    .controller(address.instrument_id, address.controller_id)
                    .map(|controller| controller.enabled());
                if let Some(enabled) = enabled {
                    self.controller_set_enabled(address, !enabled).await;
                }
            }
            MidiAppAction::SetInstrumentControllerEnabled(address, enabled) => {
                self.controller_set_enabled(address, enabled).await;
            }
            MidiAppAction::ActivateUserPreset(index) => {
                self.activate_user_preset(index).await;
            }
        }
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

    pub fn subscribe_midi_app_controls_changes(
        &self,
    ) -> broadcast::Receiver<Arc<CtrMidiEngineControls>> {
        self.midi_app_controls_change_tx.subscribe()
    }

    pub async fn start_recording(
        &self,
        sample_rate: u32,
        file_path: PathBuf,
    ) -> Result<(), EngineError> {
        if *self.audio_rec_state.read().await != AudioRecordingState::Stopped {
            return Err(EngineError::Rejected);
        }
        let mut parameters = self.audio_rec_params.lock().await;
        parameters.sample_rate = sample_rate;
        parameters.file_path = file_path;
        drop(parameters);
        *self.audio_rec_state.write().await = AudioRecordingState::Running;
        Ok(())
    }

    pub async fn set_recording_paused(&self, paused: bool) -> Result<(), EngineError> {
        if *self.audio_rec_state.read().await == AudioRecordingState::Stopped {
            return Err(EngineError::Rejected);
        }
        *self.audio_rec_state.write().await =
            if paused { AudioRecordingState::Paused } else { AudioRecordingState::Running };
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
        self.accept(Command::Silence).await
    }

    pub async fn change_project_storage(&self, storage: Option<CtrProjectStorage>) {
        let mut project = self.project.write().await;
        project.storage = storage.clone();
        self.broadcast_project_change(project.revision, ProjectChangeKind::Storage { storage });
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

    pub async fn replace_project(
        &self,
        ctr: CtrProject,
        rt: RtProject,
        controller_graph: RtControllerGraph,
        midi_app_controls: RtMidiProjectControls,
    ) -> Result<(), EngineError> {
        self.accept(Command::ReplaceProject { project: Box::new(rt) }).await?;
        *self.controller_graph.write().await = controller_graph;
        *self.project_midi_app_controls.write().await = midi_app_controls;
        self.pending_replies.lock().await.clear();
        let revision = ctr.revision;
        *self.project.write().await = ctr;
        self.broadcast_project_change(revision, ProjectChangeKind::Full);
        Ok(())
    }

    pub async fn set_global_effects(
        &self,
        effects: Effects,
        rt_effects: RtEffects,
    ) -> Result<(), EngineError> {
        self.accept(Command::SetGlobalEffects { effects: rt_effects }).await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.global_effects = effects.clone();
        self.broadcast_project_change(revision, ProjectChangeKind::GlobalEffects { effects });
        Ok(())
    }

    pub async fn set_global_gain(&self, gain: f32) -> Result<(), EngineError> {
        self.accept(Command::SetGlobalGain { gain }).await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.global_gain = gain;
        self.broadcast_project_change(revision, ProjectChangeKind::GlobalGain { gain });
        Ok(())
    }

    pub async fn set_global_transposition(&self, transposition: i8) {
        self.controller_graph.write().await.global_transposition = transposition;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.global_transposition = transposition;
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::GlobalTransposition { transposition },
        );
    }

    pub async fn set_transport(&self, transport: MusicalTransport) -> Result<(), EngineError> {
        if !transport.is_valid() {
            return Err(EngineError::Rejected);
        }
        let mut resized = Vec::new();
        {
            let project = self.project.read().await;
            for (instrument_id, instrument) in &project.instrument_graph.instruments {
                for (controller_id, controller) in &instrument.controllers {
                    let crate::engine::controller_graph::CtrController::DrumMachine(machine) =
                        controller
                    else {
                        continue;
                    };
                    let mut machine = machine.clone();
                    if machine.resize_to_bar(transport) {
                        let address = ControllerAddress {
                            instrument_id: *instrument_id,
                            controller_id: *controller_id,
                        };
                        resized.push((address, machine));
                    }
                }
            }
        }
        self.accept(Command::SetTransport { transport }).await?;
        for (address, machine) in &resized {
            self.accept(Command::ControllerSetDrumMachine {
                address: *address,
                machine: Some(RtDrumMachine::compile(machine)),
            })
            .await?;
        }
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.transport = transport;
        for (address, machine) in resized {
            if let Some(controller) = project
                .instrument_graph
                .controller_mut(address.instrument_id, address.controller_id)
            {
                *controller = crate::engine::controller_graph::CtrController::DrumMachine(machine);
            }
            self.broadcast_project_change(
                revision,
                ProjectChangeKind::Controller { address, removed: false },
            );
        }
        self.broadcast_project_change(revision, ProjectChangeKind::Transport { transport });
        Ok(())
    }

    pub async fn instrument_push(
        &self,
        ctr: CtrInstrument,
        rt: RtInstrument,
        rt_controllers: RtInstrumentControllers,
    ) -> Result<InstrumentId, EngineError> {
        self.accept(Command::InstrumentPush { instrument: rt }).await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        let id = project.instrument_graph.push(ctr);
        let controller_id = self.controller_graph.write().await.instruments.push(rt_controllers);
        if id != controller_id {
            return Err(EngineError::Rejected);
        }
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::Instrument { id, removed: false },
        );
        Ok(id)
    }

    pub async fn instrument_replace(
        &self,
        id: InstrumentId,
        kind: CtrInstrumentKind,
        rt_kind: RtInstrumentKind,
    ) -> Result<(), EngineError> {
        self.accept(Command::InstrumentReplace { id, kind: rt_kind }).await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.replace_kind(id, kind.clone());
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentKind { id, kind });
        Ok(())
    }

    pub async fn instrument_remove(&self, id: InstrumentId) -> Result<(), EngineError> {
        self.accept(Command::InstrumentRemove { id }).await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.remove(id);
        let _ = self.controller_graph.write().await.instruments.try_remove(id);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::Instrument { id, removed: true },
        );
        Ok(())
    }

    pub async fn instrument_select_soundfont_program(
        &self,
        id: InstrumentId,
        program: SoundFontProgram,
    ) -> Result<(), EngineError> {
        self.accept(Command::InstrumentSelectSoundfontProgram { id, program }).await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.select_soundfont_program(id, program);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentSoundfontProgram { id, program },
        );
        Ok(())
    }

    pub async fn instrument_set_name(&self, id: InstrumentId, name: String) {
        let mut project = self.project.write().await;
        let Some(instrument) = project.instrument_graph.instruments.get_mut(id) else {
            return;
        };
        instrument.name = name.clone();
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentName { id, name });
    }

    pub async fn instrument_set_clap_parameter(
        &self,
        id: InstrumentId,
        parameter_id: u32,
        value: f64,
    ) -> Result<(), EngineError> {
        self.project.read().await.instrument_graph.validate_clap_parameter(
            id,
            parameter_id,
            value,
        )?;
        self.accept(Command::InstrumentSetClapParameter { id, parameter_id, value }).await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.instrument_graph.set_clap_parameter(id, parameter_id, value)?;
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentClapParameter { id, parameter_id, value },
        );
        Ok(())
    }

    pub async fn instrument_set_effects(
        &self,
        id: InstrumentId,
        effects: Effects,
        rt_effects: RtEffects,
    ) -> Result<(), EngineError> {
        self.accept(Command::InstrumentSetEffects { id, effects: rt_effects }).await?;
        let mut project = self.project.write().await;
        let Some(instrument) = project.instrument_graph.instruments.get_mut(id) else {
            return Err(EngineError::Rejected);
        };
        instrument.effects = effects.clone();
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentEffects { id, effects },
        );
        Ok(())
    }

    pub async fn instrument_set_gain(
        &self,
        id: InstrumentId,
        gain: f32,
    ) -> Result<(), EngineError> {
        self.accept(Command::InstrumentSetGain { id, gain }).await?;
        let mut project = self.project.write().await;
        let Some(instrument) = project.instrument_graph.instruments.get_mut(id) else {
            return Err(EngineError::Rejected);
        };
        instrument.gain = gain;
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(revision, ProjectChangeKind::InstrumentGain { id, gain });
        Ok(())
    }

    pub async fn controller_append(
        &self,
        instrument_id: InstrumentId,
    ) -> Result<ControllerId, EngineError> {
        if self.project.read().await.instrument_graph.instruments.get(instrument_id).is_none() {
            return Err(EngineError::Rejected);
        }
        self.accept(Command::ControllerAppend { instrument_id }).await?;
        let mut project = self.project.write().await;
        let id = project
            .instrument_graph
            .append_controller(instrument_id)
            .ok_or(EngineError::Rejected)?;
        let rt_id = self
            .controller_graph
            .write()
            .await
            .instruments
            .get_mut(instrument_id)
            .ok_or(EngineError::Rejected)?
            .controllers
            .push(RtController::MidiInput(Default::default()));
        if id != rt_id {
            return Err(EngineError::Rejected);
        }
        let revision = CtrProject::next_revision(&mut project.revision);
        let address = ControllerAddress { instrument_id, controller_id: id };
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::Controller { address, removed: false },
        );
        Ok(id)
    }

    pub async fn drum_machine_append(
        &self,
        instrument_id: InstrumentId,
    ) -> Result<ControllerId, EngineError> {
        let transport = {
            let project = self.project.read().await;
            if project.instrument_graph.instruments.get(instrument_id).is_none() {
                return Err(EngineError::Rejected);
            }
            project.transport
        };
        self.accept(Command::ControllerAppend { instrument_id }).await?;
        let mut controller = DrumMachineController::default();
        controller.resize_to_bar(transport);
        let mut project = self.project.write().await;
        let instrument = project
            .instrument_graph
            .instruments
            .get_mut(instrument_id)
            .ok_or(EngineError::Rejected)?;
        let id = instrument
            .controllers
            .push(crate::engine::controller_graph::CtrController::DrumMachine(controller.clone()));
        let rt_id = self
            .controller_graph
            .write()
            .await
            .instruments
            .get_mut(instrument_id)
            .ok_or(EngineError::Rejected)?
            .controllers
            .push(RtController::DrumMachine);
        if id != rt_id {
            return Err(EngineError::Rejected);
        }
        let address = ControllerAddress { instrument_id, controller_id: id };
        drop(project);
        self.accept(Command::ControllerSetDrumMachine {
            address,
            machine: Some(RtDrumMachine::compile(&controller)),
        })
        .await?;
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::Controller { address, removed: false },
        );
        Ok(id)
    }

    pub async fn controller_remove(&self, address: ControllerAddress) -> Result<(), EngineError> {
        if self
            .project
            .read()
            .await
            .instrument_graph
            .controller(address.instrument_id, address.controller_id)
            .is_none()
        {
            return Err(EngineError::Rejected);
        }
        self.accept(Command::ControllerRemove { address }).await?;
        let mut project = self.project.write().await;
        if !project.instrument_graph.remove_controller(address.instrument_id, address.controller_id)
        {
            return Err(EngineError::Rejected);
        }
        let removed = self
            .controller_graph
            .write()
            .await
            .instruments
            .get_mut(address.instrument_id)
            .and_then(|instrument| instrument.controllers.try_remove(address.controller_id));
        if removed.is_none() {
            return Err(EngineError::Rejected);
        }
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::Controller { address, removed: true },
        );
        Ok(())
    }

    pub async fn controller_replace(
        &self,
        address: ControllerAddress,
        controller: crate::engine::controller_graph::CtrController,
    ) -> Result<(), EngineError> {
        if let Some(machine) = controller.drum_machine()
            && !machine.is_valid()
        {
            return Err(EngineError::Rejected);
        }
        let (_, rt_controller) =
            crate::engine::controller_graph::prepare(controller.clone().into());
        let machine = controller.drum_machine().map(RtDrumMachine::compile);
        self.accept(Command::ControllerSetDrumMachine { address, machine }).await?;
        let mut project = self.project.write().await;
        let Some(ctr_controller) =
            project.instrument_graph.controller_mut(address.instrument_id, address.controller_id)
        else {
            return Err(EngineError::Rejected);
        };
        let mut controllers = self.controller_graph.write().await;
        let Some(rt) = controllers.get_mut(address) else {
            return Err(EngineError::Rejected);
        };
        *ctr_controller = controller;
        *rt = rt_controller;
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::Controller { address, removed: false },
        );
        Ok(())
    }

    pub async fn controller_set_enabled(&self, address: ControllerAddress, flag: bool) {
        let is_drum_machine = {
            let project = self.project.read().await;
            project
                .instrument_graph
                .controller(address.instrument_id, address.controller_id)
                .map(|controller| controller.drum_machine().is_some())
        };
        if is_drum_machine == Some(true) {
            if self
                .accept(Command::ControllerSetDrumMachineEnabled { address, enabled: flag })
                .await
                .is_err()
            {
                return;
            }
            let mut project = self.project.write().await;
            let Some(crate::engine::controller_graph::CtrController::DrumMachine(controller)) =
                project
                    .instrument_graph
                    .controller_mut(address.instrument_id, address.controller_id)
            else {
                return;
            };
            controller.enabled = flag;
            let revision = CtrProject::next_revision(&mut project.revision);
            self.broadcast_project_change(
                revision,
                ProjectChangeKind::ControllerEnabled { address, flag },
            );
            return;
        }
        if is_drum_machine == Some(false) {
            self.update_controller(address, ControllerUpdate::Enabled(flag)).await;
        }
    }

    pub async fn controller_set_ignore_key_transform(
        &self,
        address: ControllerAddress,
        flag: bool,
    ) {
        self.update_controller(address, ControllerUpdate::IgnoreKeyTransform(flag)).await;
    }

    pub async fn controller_set_key_transform(
        &self,
        address: ControllerAddress,
        transform: MidiKeyTransform,
    ) {
        self.update_controller(address, ControllerUpdate::KeyTransform(transform)).await;
    }

    pub async fn controller_set_ignore_velocity_transform(
        &self,
        address: ControllerAddress,
        flag: bool,
    ) {
        self.update_controller(address, ControllerUpdate::IgnoreVelocityTransform(flag)).await;
    }

    pub async fn controller_set_velocity_transform(
        &self,
        address: ControllerAddress,
        transform: MidiVelocityTransform,
    ) {
        self.update_controller(address, ControllerUpdate::VelocityTransform(transform)).await;
    }

    pub async fn controller_set_channel_filter(
        &self,
        address: ControllerAddress,
        filter: MidiFilter,
    ) {
        self.update_controller(address, ControllerUpdate::ChannelFilter(filter)).await;
    }

    pub async fn controller_set_source_filter(
        &self,
        address: ControllerAddress,
        filter: MidiFilter,
    ) {
        self.update_controller(address, ControllerUpdate::SourceFilter(filter)).await;
    }

    pub async fn controller_set_cc_filter(&self, address: ControllerAddress, filter: MidiFilter) {
        self.update_controller(address, ControllerUpdate::CcFilter(filter)).await;
    }

    pub async fn controller_set_ignore_pitch_bend(&self, address: ControllerAddress, flag: bool) {
        self.update_controller(address, ControllerUpdate::IgnorePitchBend(flag)).await;
    }

    pub async fn controller_set_ignore_program_change(
        &self,
        address: ControllerAddress,
        flag: bool,
    ) {
        self.update_controller(address, ControllerUpdate::IgnoreProgramChange(flag)).await;
    }

    pub async fn controller_set_ignore_channel_aftertouch(
        &self,
        address: ControllerAddress,
        flag: bool,
    ) {
        self.update_controller(address, ControllerUpdate::IgnoreChannelAftertouch(flag)).await;
    }

    pub async fn controller_set_ignore_polyphonic_aftertouch(
        &self,
        address: ControllerAddress,
        flag: bool,
    ) {
        self.update_controller(address, ControllerUpdate::IgnorePolyphonicAftertouch(flag)).await;
    }

    pub async fn controller_set_use_global_transposition(
        &self,
        address: ControllerAddress,
        flag: bool,
    ) {
        self.update_controller(address, ControllerUpdate::UseGlobalTransposition(flag)).await;
    }

    pub async fn controller_set_name(&self, address: ControllerAddress, name: String) {
        self.update_controller(address, ControllerUpdate::Name(name)).await;
    }

    pub async fn set_user_presets(&self, presets: CtrUserPresets) {
        let mut project = self.project.write().await;
        let revision = CtrProject::next_revision(&mut project.revision);
        project.user_presets = presets;
        self.broadcast_project_change(revision, ProjectChangeKind::UserPresets);
    }

    pub async fn activate_user_preset(&self, index: usize) {
        let project = self.project.read().await;
        let Some(preset) = project.user_presets.entries.get(index) else {
            return;
        };
        let actions = preset.actions.clone();
        let all_controllers: Vec<_> = (&project.instrument_graph.instruments)
            .into_iter()
            .flat_map(|(instrument_id, instrument)| {
                instrument.controllers.iter_valid_ids().map(|controller_id| ControllerAddress {
                    instrument_id: *instrument_id,
                    controller_id: *controller_id,
                })
            })
            .collect();
        drop(project);

        for action in actions {
            match action {
                UserPresetAction::DisableAllInstrumentControllers => {
                    for controller in &all_controllers {
                        self.controller_set_enabled(*controller, false).await;
                    }
                }
                UserPresetAction::SetInstrumentControllerEnabled { controller, flag } => {
                    self.controller_set_enabled(controller, flag).await;
                }
                UserPresetAction::SetSoundFontProgram { instrument, program } => {
                    let _ = self.instrument_select_soundfont_program(instrument, program).await;
                }
                UserPresetAction::SetClapParameter { instrument, parameter_id, value } => {
                    let _ =
                        self.instrument_set_clap_parameter(instrument, parameter_id, value).await;
                }
            }
        }
    }

    pub async fn midi_engine_controls(&self) -> CtrMidiEngineControls {
        self.ctr_midi_app_controls.read().await.clone()
    }

    pub async fn set_midi_engine_controls(&self, controls: CtrMidiEngineControls) {
        let rt = RtMidiEngineControls::from(&controls);
        *self.rt_midi_app_controls.write().await = rt;
        *self.ctr_midi_app_controls.write().await = controls.clone();
        let _ = self.midi_app_controls_change_tx.send(Arc::new(controls));
    }

    pub async fn set_midi_project_controls(&self, controls: CtrMidiProjectControls) {
        let rt = RtMidiProjectControls::from(&controls);
        *self.project_midi_app_controls.write().await = rt;
        let mut project = self.project.write().await;
        project.midi_app_controls = controls.clone();
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(revision, ProjectChangeKind::MidiAppControls { controls });
    }

    async fn update_controller(&self, address: ControllerAddress, update: ControllerUpdate) {
        let mut project = self.project.write().await;
        let Some(controller) =
            project.instrument_graph.controller_mut(address.instrument_id, address.controller_id)
        else {
            return;
        };
        let Some(controller) = controller.midi_input_mut() else {
            return;
        };
        let mut controllers = self.controller_graph.write().await;
        let Some(RtController::MidiInput(rt)) = controllers.get_mut(address) else {
            return;
        };

        let kind = update.apply(controller, rt, address);
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(revision, kind);
    }

    async fn remember_controller_value(&self, id: InstrumentId, controller: u8, value: u8) {
        self.project.write().await.instrument_graph.save_controller_value(id, controller, value);
    }

    async fn apply_program_change(&self, id: InstrumentId, program: u8) {
        let mut project = self.project.write().await;
        let Some(CtrInstrumentProgramChange::SoundFont(program)) =
            project.instrument_graph.change_program(id, program)
        else {
            return;
        };
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentSoundfontProgram { id, program },
        );
    }

    async fn apply_bank_change(&self, id: InstrumentId, bank: u8) {
        let mut project = self.project.write().await;
        let Some(CtrInstrumentProgramChange::SoundFont(program)) =
            project.instrument_graph.change_bank_lsb(id, bank)
        else {
            return;
        };
        let revision = CtrProject::next_revision(&mut project.revision);
        self.broadcast_project_change(
            revision,
            ProjectChangeKind::InstrumentSoundfontProgram { id, program },
        );
    }

    async fn accept(&self, command: Command) -> Result<(), EngineError> {
        if self.command(command).await? { Ok(()) } else { Err(EngineError::Rejected) }
    }

    async fn command(&self, command: Command) -> Result<bool, EngineError> {
        let id = self.next_cmd_id.fetch_add(1, Ordering::Relaxed);
        let (reply_tx, reply_rx) = oneshot::channel();
        self.pending_replies.lock().await.insert(id, reply_tx);
        if self.cmd_tx.push((id, command)).await.is_err() {
            self.pending_replies.lock().await.remove(&id);
            return Err(EngineError::QueueFull);
        }
        reply_rx.await.map_err(|_| EngineError::ReplyChannelClosed)
    }

    fn broadcast_project_change(&self, revision: u32, kind: ProjectChangeKind) {
        let _ = self.project_change_tx.send(Arc::new(ProjectChange { revision, kind }));
    }
}

enum ControllerUpdate {
    Enabled(bool),
    SourceFilter(MidiFilter),
    IgnoreKeyTransform(bool),
    KeyTransform(MidiKeyTransform),
    IgnoreVelocityTransform(bool),
    VelocityTransform(MidiVelocityTransform),
    ChannelFilter(MidiFilter),
    CcFilter(MidiFilter),
    IgnorePitchBend(bool),
    IgnoreProgramChange(bool),
    IgnoreChannelAftertouch(bool),
    IgnorePolyphonicAftertouch(bool),
    UseGlobalTransposition(bool),
    Name(String),
}

impl ControllerUpdate {
    fn apply(
        self,
        ctr: &mut crate::engine::controller_graph::CtrMidiInputController,
        rt: &mut crate::engine::controller_graph::RtMidiInputController,
        address: ControllerAddress,
    ) -> ProjectChangeKind {
        match self {
            Self::Enabled(flag) => {
                ctr.enabled = flag;
                rt.enabled = flag;
                ProjectChangeKind::ControllerEnabled { address, flag }
            }
            Self::SourceFilter(filter) => {
                rt.source_mask = filter.as_u128();
                ctr.source_filter = filter.clone();
                ProjectChangeKind::ControllerSourceFilter { address, filter }
            }
            Self::IgnoreKeyTransform(flag) => {
                ctr.ignore_key_transform = flag;
                rt.ignore_key_transform = flag;
                ProjectChangeKind::ControllerIgnoreKeyTransform { address, flag }
            }
            Self::KeyTransform(transform) => {
                rt.key_transform = transform.compile();
                ctr.key_transform = transform.clone();
                ProjectChangeKind::ControllerKeyTransform { address, transform }
            }
            Self::IgnoreVelocityTransform(flag) => {
                ctr.ignore_velocity_transform = flag;
                rt.ignore_velocity_transform = flag;
                ProjectChangeKind::ControllerIgnoreVelocityTransform { address, flag }
            }
            Self::VelocityTransform(transform) => {
                rt.velocity_transform = transform.compile();
                ctr.velocity_transform = transform.clone();
                ProjectChangeKind::ControllerVelocityTransform { address, transform }
            }
            Self::ChannelFilter(filter) => {
                rt.channel_mask = filter.as_u16();
                ctr.channel_filter = filter.clone();
                ProjectChangeKind::ControllerChannelFilter { address, filter }
            }
            Self::CcFilter(filter) => {
                rt.cc_mask = filter.as_u128();
                ctr.cc_filter = filter.clone();
                ProjectChangeKind::ControllerCcFilter { address, filter }
            }
            Self::IgnorePitchBend(flag) => {
                ctr.ignore_pitch_bend = flag;
                rt.ignore_pitch_bend = flag;
                ProjectChangeKind::ControllerIgnorePitchBend { address, flag }
            }
            Self::IgnoreProgramChange(flag) => {
                ctr.ignore_program_change = flag;
                rt.ignore_program_change = flag;
                ProjectChangeKind::ControllerIgnoreProgramChange { address, flag }
            }
            Self::IgnoreChannelAftertouch(flag) => {
                ctr.ignore_channel_aftertouch = flag;
                rt.ignore_channel_aftertouch = flag;
                ProjectChangeKind::ControllerIgnoreChannelAftertouch { address, flag }
            }
            Self::IgnorePolyphonicAftertouch(flag) => {
                ctr.ignore_polyphonic_aftertouch = flag;
                rt.ignore_polyphonic_aftertouch = flag;
                ProjectChangeKind::ControllerIgnorePolyphonicAftertouch { address, flag }
            }
            Self::UseGlobalTransposition(flag) => {
                ctr.use_global_transposition = flag;
                rt.use_global_transposition = flag;
                ProjectChangeKind::ControllerUseGlobalTransposition { address, flag }
            }
            Self::Name(name) => {
                ctr.name = name.clone();
                ProjectChangeKind::ControllerName { address, name }
            }
        }
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
    pub(super) clap_parameter_rx: RtConsumer<ClapParameterChange>,
    pub(super) stream_error: Arc<AtomicBool>,
    pub(super) cmd_notify: Arc<tokio::sync::Notify>,
    pub(super) midi_notify: Arc<tokio::sync::Notify>,
    pub(super) engine: Arc<Mutex<Engine>>,
    pub(super) audio_rec_state: Arc<RwLock<AudioRecordingState>>,
    pub(super) audio_rec_params: Arc<Mutex<AudioRecordingParams>>,
    pub(super) project: Arc<RwLock<CtrProject>>,
    pub(super) project_change_tx: broadcast::Sender<Arc<ProjectChange>>,
    pub(super) audio_rec_capture_enabled: Arc<AtomicBool>,
    pub(super) audio_rec_active_sample_rate: u32,
    pub(super) audio_rec_file: Option<File>,
    pub(super) audio_rec_buffer: Vec<u8>,
    pub(super) audio_rec_bytes: u32,
}

impl Runtime {
    pub async fn process_replies(&mut self) {
        while let Ok((id, reply)) = self.reply_rx.pop() {
            if let Some(sender) = self.pending_replies.lock().await.remove(&id) {
                let _ = sender.send(reply);
            }
        }
    }

    pub async fn process_clap_parameter_changes(&mut self) {
        while let Ok(change) = self.clap_parameter_rx.pop() {
            if !change.value.is_finite() {
                continue;
            }
            let mut project = self.project.write().await;
            if !project.instrument_graph.update_clap_parameter(
                change.instrument_id,
                change.parameter_id,
                change.value,
            ) {
                continue;
            }
            let revision = CtrProject::next_revision(&mut project.revision);
            drop(project);
            let _ = self.project_change_tx.send(Arc::new(ProjectChange {
                revision,
                kind: ProjectChangeKind::InstrumentClapParameter {
                    id: change.instrument_id,
                    parameter_id: change.parameter_id,
                    value: change.value,
                },
            }));
        }
    }

    pub fn stream_error(&self) -> bool {
        self.stream_error.load(Ordering::Relaxed)
    }

    pub async fn start_recording(&mut self) -> io::Result<()> {
        self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);
        while self.recording_rx.pop().is_ok() {}
        let parameters = self.audio_rec_params.lock().await.clone();
        if parameters.sample_rate == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid sample rate"));
        }
        let mut file = File::create(&parameters.file_path).await?;
        file.write_all(&wav_header(parameters.sample_rate, 0)?).await?;
        self.audio_rec_active_sample_rate = parameters.sample_rate;
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
            for index in 0..count {
                let left = (chunk.left[index].clamp(-1.0, 1.0) * 32767.0) as i16;
                let right = (chunk.right[index].clamp(-1.0, 1.0) * 32767.0) as i16;
                self.audio_rec_buffer.extend_from_slice(&left.to_le_bytes());
                self.audio_rec_buffer.extend_from_slice(&right.to_le_bytes());
            }
            let length = self.audio_rec_buffer.len() as u32;
            self.audio_rec_bytes = self
                .audio_rec_bytes
                .checked_add(length)
                .filter(|size| *size <= u32::MAX - 36)
                .ok_or_else(|| io::Error::other("WAV size limit exceeded"))?;
            file.write_all(&self.audio_rec_buffer).await?;
        }
        Ok(())
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
        file.flush().await
    }

    pub async fn update_recording(&mut self) -> io::Result<()> {
        let recording_state = *self.audio_rec_state.read().await;
        match recording_state {
            AudioRecordingState::Running => {
                if self.audio_rec_file.is_none() {
                    self.start_recording().await?;
                }
                self.audio_rec_capture_enabled.store(true, Ordering::Relaxed);
                self.process_recording().await?;
            }
            AudioRecordingState::Paused => {
                self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);
                self.process_recording().await?;
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
        let command_notify = Arc::clone(&self.cmd_notify);
        let midi_notify = Arc::clone(&self.midi_notify);
        self.audio_rec_capture_enabled.store(false, Ordering::Relaxed);

        let result = async {
            loop {
                tokio::select! {
                    _ = stop.cancelled() => break,
                    _ = interval.tick() => {
                        self.process_replies().await;
                        if self.stream_error() {
                            self.engine.lock().await.idle_flush_clap_parameters();
                        }
                        self.process_clap_parameter_changes().await;
                        self.update_recording().await?;
                    }
                    _ = command_notify.notified(), if self.stream_error() => {
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

fn wav_header(sample_rate: u32, data_size: u32) -> io::Result<[u8; 44]> {
    let riff_size =
        data_size.checked_add(36).ok_or_else(|| io::Error::other("WAV size limit exceeded"))?;
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
