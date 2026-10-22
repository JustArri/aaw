use crate::{
    audio::block::AudioBlock,
    engine::{
        effects::RtEffects,
        graph::{InstrumentId, SynthId},
        project::RtProject,
        synth_graph::{RtSynth, RtSynthKind, SynthGraphError},
    },
    midi::{CompiledMidiKeyTransform, CompiledMidiVelocityTransform, MidiEvent},
    rt_channels::{RtConsumer, RtProducer},
    synths::SoundFontProgram,
};

// sacrifice memory for latency (avoid allocations/deallocations in the audio thread)
#[allow(clippy::large_enum_variant)]
pub enum Command {
    ReplaceProject { project: Box<RtProject> },

    Silence,
    SetEffects { effects: RtEffects },

    SynthPush { synth: RtSynth },
    SynthReplace { id: SynthId, kind: RtSynthKind },
    SynthRemove { id: SynthId },
    SynthSelectSoundfontProgram { id: SynthId, program: SoundFontProgram },
    SynthSetEffects { id: SynthId, effects: RtEffects },

    SetGlobalTransposition { transposition: i8 },

    InstrAppendNew,
    InstrRemove { id: InstrumentId },
    InstrSetSynth { id: InstrumentId, synth_id: Option<SynthId> },
    InstrSetEnabled { id: InstrumentId, flag: bool },
    InstrSetIgnoreKeyTransform { id: InstrumentId, flag: bool },
    InstrSetKeyTransform { id: InstrumentId, transform: CompiledMidiKeyTransform },
    InstrSetIgnoreVelocityTransform { id: InstrumentId, flag: bool },
    InstrSetVelocityTransform { id: InstrumentId, transform: CompiledMidiVelocityTransform },
    InstrSetChannelFilter { id: InstrumentId, mask: u16 },
    InstrSetControlChangeFilter { id: InstrumentId, mask: u128 },
    InstrSetIgnorePitchBend { id: InstrumentId, flag: bool },
    InstrSetIgnoreProgramChange { id: InstrumentId, flag: bool },
    InstrSetIgnoreChannelAftertouch { id: InstrumentId, flag: bool },
    InstrSetIgnorePolyphonicAftertouch { id: InstrumentId, flag: bool },
    InstrSetUseGlobalTransposition { id: InstrumentId, flag: bool },
}

pub struct AudioBlocks {
    pub main: AudioBlock,
    pub temp: AudioBlock,
}

pub struct Engine {
    pub project: RtProject,
    pub audio_blocks: Option<AudioBlocks>,
    pub midi_rx: RtConsumer<MidiEvent>,
    pub cmd_rx: RtConsumer<(u32, Command)>,
    pub reply_tx: RtProducer<(u32, bool)>,
}

impl Engine {
    pub fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), SynthGraphError> {
        self.silence();
        self.create_buffers(channel_count, max_block_size);
        self.project.synth_graph.reconfigure(channel_count, sample_rate, max_block_size)?;
        Ok(())
    }

    pub fn idle_process_commands(&mut self) {
        self.dispatch_commands();
    }

    pub fn idle_process_midi_events(&mut self) {
        self.dispatch_midi_events();
    }

    pub fn run(&mut self, buffer: &mut [f32]) {
        self.dispatch_commands();
        self.dispatch_midi_events();
        self.render_next_audio_block(buffer);
    }

    pub fn create_buffers(&mut self, channel_count: usize, max_block_size: usize) {
        let main = AudioBlock::new(channel_count, max_block_size);
        let temp = main.mirror();
        self.audio_blocks = Some(AudioBlocks { main, temp });
    }

    fn dispatch_commands(&mut self) {
        while let Ok((id, cmd)) = self.cmd_rx.pop() {
            let res = self.dispatch_command(cmd);
            if self.reply_tx.push((id, res)).is_err() {
                tracing::error!("failed to ack cmd #{id}")
            }
        }
    }

    fn silence(&mut self) {
        self.project.synth_graph.silence();
    }

    fn dispatch_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::ReplaceProject { project } => {
                self.project = *project;
                true
            }

            Command::Silence => {
                self.silence();
                true
            }
            Command::SetEffects { effects } => {
                self.project.synth_graph.set_effects(effects);
                true
            }

            Command::SynthPush { synth } => self.project.synth_graph.push(synth),
            Command::SynthReplace { id, kind } => self.project.synth_graph.replace(id, kind),
            Command::SynthRemove { id } => {
                let removed = self.project.synth_graph.remove(id);
                if removed {
                    self.project.instrument_graph.remove_synth_references(id);
                }
                removed
            }
            Command::SynthSelectSoundfontProgram { id, program } => {
                self.project.synth_graph.select_soundfont_program(id, program)
            }
            Command::SynthSetEffects { id, effects } => {
                self.project.synth_graph.set_synth_effects(id, effects)
            }

            Command::SetGlobalTransposition { transposition } => {
                self.project.instrument_graph.set_global_transposition(transposition);
                true
            }

            Command::InstrAppendNew => self.project.instrument_graph.append_new(),
            Command::InstrRemove { id } => self.project.instrument_graph.remove(id),
            Command::InstrSetSynth { id, synth_id } => {
                self.project.instrument_graph.set_synth(id, synth_id)
            }
            Command::InstrSetEnabled { id, flag } => {
                self.project.instrument_graph.set_enabled(id, flag)
            }
            Command::InstrSetIgnoreKeyTransform { id, flag } => {
                self.project.instrument_graph.set_ignore_key_transform(id, flag)
            }
            Command::InstrSetKeyTransform { id, transform } => {
                self.project.instrument_graph.set_key_transform(id, transform)
            }
            Command::InstrSetIgnoreVelocityTransform { id, flag } => {
                self.project.instrument_graph.set_ignore_velocity_transform(id, flag)
            }
            Command::InstrSetVelocityTransform { id, transform } => {
                self.project.instrument_graph.set_velocity_transform(id, transform)
            }
            Command::InstrSetChannelFilter { id, mask } => {
                self.project.instrument_graph.set_channel_filter(id, mask)
            }
            Command::InstrSetControlChangeFilter { id, mask } => {
                self.project.instrument_graph.set_cc_filter(id, mask)
            }
            Command::InstrSetIgnorePitchBend { id, flag } => {
                self.project.instrument_graph.set_ignore_pitch_bend(id, flag)
            }
            Command::InstrSetIgnoreProgramChange { id, flag } => {
                self.project.instrument_graph.set_ignore_program_change(id, flag)
            }
            Command::InstrSetIgnoreChannelAftertouch { id, flag } => {
                self.project.instrument_graph.set_ignore_channel_aftertouch(id, flag)
            }
            Command::InstrSetIgnorePolyphonicAftertouch { id, flag } => {
                self.project.instrument_graph.set_ignore_polyphonic_aftertouch(id, flag)
            }
            Command::InstrSetUseGlobalTransposition { id, flag } => {
                self.project.instrument_graph.set_use_global_transposition(id, flag)
            }
        }
    }

    fn dispatch_midi_events(&mut self) {
        while let Ok(event) = self.midi_rx.pop() {
            self.dispatch_midi_event(event);
        }
    }

    fn dispatch_midi_event(&mut self, event: MidiEvent) {
        self.project.instrument_graph.midi_handler_iter(event).for_each(|(id, event)| {
            let _ = self.project.synth_graph.handle_midi_event(id, event);
        });
    }

    fn render_next_audio_block(&mut self, buffer: &mut [f32]) {
        let Some(audio_blocks) = self.audio_blocks.as_mut() else {
            buffer.fill(0.0);
            return;
        };

        let sample_count = buffer.len() / audio_blocks.main.channel_count();
        audio_blocks.main.resize(sample_count);
        audio_blocks.temp.resize(sample_count);

        audio_blocks.main.clear();

        for synth in self.project.synth_graph.synths.iter_values_mut() {
            let _ = synth.render_audio_block(&mut audio_blocks.temp);
            audio_blocks.main.add_samples_from_unchecked(&audio_blocks.temp);
        }

        audio_blocks.main.write_interleaved_f32(buffer);
    }
}
