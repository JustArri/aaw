use crate::{
    audio::{AudioBlock, AudioRecordingChunk},
    engine::{
        drum_machine::{MAX_SCHEDULED_EVENTS_PER_BLOCK, MusicalTransport, ScheduledDrumEvent},
        effects::RtEffects,
        graph::InstrumentId,
        instrument_graph::{InstrumentGraphError, RtInstrument, RtInstrumentKind},
        metrics::{CallbackDetails, CallbackMidiEvent, EngineMetrics},
        project::RtProject,
    },
    midi::MidiEvent,
    rt_channels::{RtConsumer, RtProducer},
    synths::SoundFontProgram,
};
use arrayvec::ArrayVec;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct InstrumentMidiEvent {
    pub instrument_id: InstrumentId,
    pub event: MidiEvent,
}

#[derive(Debug, Clone, Copy)]
pub struct ClapParameterChange {
    pub instrument_id: InstrumentId,
    pub parameter_id: u32,
    pub value: f64,
}

#[allow(clippy::large_enum_variant)]
pub enum Command {
    ReplaceProject {
        project: Box<RtProject>,
    },
    Silence,
    SetGlobalEffects {
        effects: RtEffects,
    },
    SetGlobalGain {
        gain: f32,
    },
    InstrumentPush {
        instrument: RtInstrument,
    },
    InstrumentReplace {
        id: InstrumentId,
        kind: RtInstrumentKind,
    },
    InstrumentRemove {
        id: InstrumentId,
    },
    InstrumentSelectSoundfontProgram {
        id: InstrumentId,
        program: SoundFontProgram,
    },
    InstrumentSetClapParameter {
        id: InstrumentId,
        parameter_id: u32,
        value: f64,
    },
    InstrumentSetEffects {
        id: InstrumentId,
        effects: RtEffects,
    },
    InstrumentSetGain {
        id: InstrumentId,
        gain: f32,
    },
    SetTransport {
        transport: MusicalTransport,
    },
    ControllerSetDrumMachine {
        address: crate::engine::graph::ControllerAddress,
        machine: Option<crate::engine::drum_machine::RtDrumMachine>,
    },
    ControllerAppend {
        instrument_id: InstrumentId,
    },
    ControllerRemove {
        address: crate::engine::graph::ControllerAddress,
    },
}

pub struct AudioBlocks {
    pub main: AudioBlock,
    pub temp: AudioBlock,
}

pub struct Engine {
    pub project: RtProject,
    pub audio_blocks: Option<AudioBlocks>,
    pub midi_rx: RtConsumer<InstrumentMidiEvent>,
    pub cmd_rx: RtConsumer<(u32, Command)>,
    pub reply_tx: RtProducer<(u32, bool)>,
    pub recording_tx: RtProducer<AudioRecordingChunk>,
    pub clap_parameter_tx: RtProducer<ClapParameterChange>,
    pub sample_rate: f32,
    pub audio_rec_capture_enabled: Arc<AtomicBool>,
}

impl Engine {
    pub async fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), InstrumentGraphError> {
        self.sample_rate = sample_rate as f32;
        self.silence();
        self.create_buffers(channel_count, max_block_size);
        self.project.instrument_graph.reconfigure(channel_count, sample_rate, max_block_size).await
    }

    pub fn idle_process_commands(&mut self) {
        self.dispatch_commands();
    }

    pub fn idle_process_midi_events(&mut self) {
        let _ = self.dispatch_midi_events();
    }

    pub fn idle_flush_clap_parameters(&mut self) {
        for (instrument_id, instrument) in &mut self.project.instrument_graph.instruments {
            let RtInstrumentKind::Clap(clap) = &mut instrument.kind else {
                continue;
            };
            clap.flush_parameters();
            push_clap_changes(
                &mut self.clap_parameter_tx,
                *instrument_id,
                clap.parameter_changes(),
            );
        }
    }

    pub fn run(&mut self, buffer: &mut [f32], metrics: &EngineMetrics) -> CallbackDetails {
        let mut details = CallbackDetails::default();
        let started = Instant::now();
        self.dispatch_commands();
        details.commands_ns = elapsed_nanos(started);
        metrics.record_commands(details.commands_ns);

        let started = Instant::now();
        let (midi_event_count, last_midi_event) = self.dispatch_midi_events();
        details.midi_ns = elapsed_nanos(started);
        details.midi_event_count = midi_event_count;
        details.last_midi_event = last_midi_event;
        metrics.record_midi(details.midi_ns);

        self.render_next_audio_block(buffer, metrics, &mut details);
        details
    }

    pub fn create_buffers(&mut self, channel_count: usize, max_block_size: usize) {
        let main = AudioBlock::new(channel_count, max_block_size);
        let temp = main.mirror();
        self.audio_blocks = Some(AudioBlocks { main, temp });
    }

    fn dispatch_commands(&mut self) {
        while let Ok((id, command)) = self.cmd_rx.pop() {
            let accepted = self.dispatch_command(command);
            if self.reply_tx.push((id, accepted)).is_err() {
                tracing::error!("failed to acknowledge command #{id}");
            }
        }
    }

    fn silence(&mut self) {
        self.project.instrument_graph.silence();
    }

    fn dispatch_command(&mut self, command: Command) -> bool {
        match command {
            Command::ReplaceProject { project } => {
                self.project = *project;
                true
            }
            Command::Silence => {
                self.silence();
                true
            }
            Command::SetGlobalEffects { effects } => {
                self.project.instrument_graph.global_effects = effects;
                true
            }
            Command::SetGlobalGain { gain } => {
                self.project.instrument_graph.target_global_gain = gain;
                true
            }
            Command::InstrumentPush { instrument } => {
                self.project.instrument_graph.push(instrument)
            }
            Command::InstrumentReplace { id, kind } => {
                self.project.instrument_graph.replace(id, kind)
            }
            Command::InstrumentRemove { id } => self.project.instrument_graph.remove(id),
            Command::InstrumentSelectSoundfontProgram { id, program } => {
                self.project.instrument_graph.select_soundfont_program(id, program)
            }
            Command::InstrumentSetClapParameter { id, parameter_id, value } => {
                self.project.instrument_graph.set_clap_parameter(id, parameter_id, value)
            }
            Command::InstrumentSetEffects { id, effects } => {
                self.project.instrument_graph.set_effects(id, effects)
            }
            Command::InstrumentSetGain { id, gain } => {
                self.project.instrument_graph.set_gain(id, gain)
            }
            Command::SetTransport { transport } => {
                if self.project.transport.config.playing && !transport.playing {
                    self.silence();
                    self.project.transport.sample_position = 0;
                    self.project.transport.segment_start_frame = 0;
                    self.project.transport.segment_start_tick = 0.0;
                } else if self.project.transport.config.playing
                    && transport.playing
                    && self.project.transport.config.tempo_bpm != transport.tempo_bpm
                {
                    let current = self.project.transport.sample_position;
                    let previous_frames_per_tick = self.sample_rate as f64 * 60.0
                        / (self.project.transport.config.tempo_bpm
                            * crate::engine::drum_machine::TICKS_PER_BEAT as f64);
                    self.project.transport.segment_start_tick +=
                        current.saturating_sub(self.project.transport.segment_start_frame) as f64
                            / previous_frames_per_tick;
                    self.project.transport.segment_start_frame = current;
                }
                self.project.transport.config = transport;
                true
            }
            Command::ControllerSetDrumMachine { address, machine } => {
                self.project.instrument_graph.set_drum_machine(address, machine)
            }
            Command::ControllerAppend { instrument_id } => {
                self.project.instrument_graph.append_controller_slot(instrument_id)
            }
            Command::ControllerRemove { address } => {
                self.project.instrument_graph.remove_controller_slot(address)
            }
        }
    }

    fn dispatch_midi_events(&mut self) -> (u64, Option<CallbackMidiEvent>) {
        let mut count = 0;
        let mut last = None;
        while let Ok(event) = self.midi_rx.pop() {
            count += 1;
            last = Some(CallbackMidiEvent {
                instrument_id: event.instrument_id.0,
                event: event.event,
            });
            let _ =
                self.project.instrument_graph.handle_midi_event(event.instrument_id, event.event);
        }
        (count, last)
    }

    fn render_next_audio_block(
        &mut self,
        buffer: &mut [f32],
        metrics: &EngineMetrics,
        details: &mut CallbackDetails,
    ) {
        let Some(audio_blocks) = self.audio_blocks.as_mut() else {
            buffer.fill(0.0);
            return;
        };
        let sample_count = buffer.len() / audio_blocks.main.channel_count();
        audio_blocks.main.resize(sample_count);
        audio_blocks.temp.resize(sample_count);
        audio_blocks.main.clear();

        let instruments_started = Instant::now();
        let transport = self.project.transport.config;
        let block_start = self.project.transport.sample_position;
        let segment_start_frame = self.project.transport.segment_start_frame;
        let segment_start_tick = self.project.transport.segment_start_tick;
        for (instrument_id, instrument) in &mut self.project.instrument_graph.instruments {
            let started = Instant::now();
            let mut scheduled: ArrayVec<ScheduledDrumEvent, MAX_SCHEDULED_EVENTS_PER_BLOCK> =
                ArrayVec::new();
            if transport.playing {
                crate::engine::instrument_graph::RtInstrumentGraph::schedule_drum_events(
                    instrument,
                    crate::engine::drum_machine::DrumScheduleContext {
                        block_start,
                        frame_count: sample_count,
                        sample_rate: self.sample_rate as f64,
                        tempo_bpm: transport.tempo_bpm,
                        segment_start_frame,
                        segment_start_tick,
                    },
                    transport.ticks_per_bar(),
                    &mut scheduled,
                );
            }
            render_instrument_with_events(
                instrument,
                &mut audio_blocks.temp,
                &mut audio_blocks.main,
                sample_count,
                &scheduled,
            );
            if let RtInstrumentKind::Clap(clap) = &mut instrument.kind {
                push_clap_changes(
                    &mut self.clap_parameter_tx,
                    *instrument_id,
                    clap.parameter_changes(),
                );
            }
            let elapsed_ns = elapsed_nanos(started);
            metrics.record_instrument(instrument_id.0, elapsed_ns);
            if elapsed_ns > details.slowest_instrument_ns {
                details.slowest_instrument_id = Some(instrument_id.0);
                details.slowest_instrument_ns = elapsed_ns;
            }
        }
        details.instruments_ns = elapsed_nanos(instruments_started);
        metrics.record_instruments(details.instruments_ns);
        if transport.playing {
            self.project.transport.sample_position =
                self.project.transport.sample_position.saturating_add(sample_count as u64);
        }
        let frames_per_tick = self.sample_rate as f64 * 60.0
            / (transport.tempo_bpm * crate::engine::drum_machine::TICKS_PER_BEAT as f64);
        let current_tick = self.project.transport.segment_start_tick
            + self
                .project
                .transport
                .sample_position
                .saturating_sub(self.project.transport.segment_start_frame) as f64
                / frames_per_tick;
        metrics.record_transport_tick(current_tick);

        let global_started = Instant::now();
        let smoothing_time = 0.5;
        let alpha = 1.0 - (-1.0 / (self.sample_rate * smoothing_time)).exp();
        let graph = &mut self.project.instrument_graph;
        graph.global_effects.apply(&mut audio_blocks.main);
        graph.global_gain = audio_blocks.main.multiply_samples_transition(
            graph.global_gain,
            graph.target_global_gain,
            alpha,
        );
        details.global_processing_ns = elapsed_nanos(global_started);
        metrics.record_global_processing(details.global_processing_ns);

        let conversion_started = Instant::now();
        audio_blocks.main.write_interleaved_f32(buffer);
        details.output_conversion_ns = elapsed_nanos(conversion_started);

        if self.audio_rec_capture_enabled.load(Ordering::Relaxed)
            && let Ok(mut chunk) = self.recording_tx.write_chunk(1)
        {
            let (slots, _) = chunk.as_mut_slices();
            slots[0].fill_from(&audio_blocks.main);
            chunk.commit_all();
        }
    }
}

fn render_instrument_with_events(
    instrument: &mut RtInstrument,
    temp: &mut AudioBlock,
    main: &mut AudioBlock,
    sample_count: usize,
    events: &[ScheduledDrumEvent],
) {
    let mut cursor = 0;
    let mut event_index = 0;
    while event_index < events.len() {
        let offset = events[event_index].sample_offset.min(sample_count);
        render_instrument_segment(instrument, temp, main, cursor, offset.saturating_sub(cursor));
        cursor = offset;
        while event_index < events.len() && events[event_index].sample_offset == offset {
            track_drum_note(instrument, events[event_index]);
            let _ = instrument.handle_midi_event(events[event_index].event);
            event_index += 1;
        }
    }
    render_instrument_segment(instrument, temp, main, cursor, sample_count.saturating_sub(cursor));
}

fn track_drum_note(instrument: &mut RtInstrument, scheduled: ScheduledDrumEvent) {
    let channel = scheduled.event.channel as usize;
    let Some(slot) = instrument.drum_machines.get_mut(scheduled.controller_id) else {
        return;
    };
    match scheduled.event.kind {
        crate::midi::MidiEventKind::NoteOn { key, velocity } if velocity > 0 => {
            slot.active_notes[channel] |= 1u128 << key;
        }
        crate::midi::MidiEventKind::NoteOff { key, .. }
        | crate::midi::MidiEventKind::NoteOn { key, velocity: 0 } => {
            slot.active_notes[channel] &= !(1u128 << key);
        }
        _ => {}
    }
}

fn render_instrument_segment(
    instrument: &mut RtInstrument,
    temp: &mut AudioBlock,
    main: &mut AudioBlock,
    offset: usize,
    length: usize,
) {
    if length == 0 {
        return;
    }
    temp.resize(length);
    temp.clear();
    let _ = instrument.render_audio_block(temp);
    main.add_samples_from_at(temp, offset);
}

fn elapsed_nanos(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn push_clap_changes(
    sender: &mut RtProducer<ClapParameterChange>,
    instrument_id: InstrumentId,
    changes: impl Iterator<Item = (u32, f64)>,
) {
    for (parameter_id, value) in changes {
        if sender.push(ClapParameterChange { instrument_id, parameter_id, value }).is_err() {
            break;
        }
    }
}
