use crate::{
    audio::block::AudioBlock,
    engine::{
        dummy_graph, fluid_graph,
        graph::{LocalSynthId, SynthId, SynthKind},
        sfizz_graph,
        synth_graph::{self, SlotId},
    },
    midi::{MidiEvent, MidiFilter, MidiKeyTransform},
    rt_channels::{RtConsumer, RtProducer},
    synths,
};

pub enum Command {
    ReplaceProject { project: Box<Project> },

    Silence,

    DummyPush { synth: synths::Dummy },
    DummyRemove { synth_id: LocalSynthId },

    FluidPush { synth: synths::Fluid },
    FluidReplace { synth_id: LocalSynthId, synth: synths::Fluid },
    FluidRemove { synth_id: LocalSynthId },
    FluidSelectProgram { synth_id: LocalSynthId, bank: u16, preset_id: u8 },

    SfizzPush { synth: synths::Sfizz },
    SfizzReplace { synth_id: LocalSynthId, synth: synths::Sfizz },
    SfizzRemove { synth_id: LocalSynthId },

    SetGlobalTransposition { transposition: i8 },

    SynthSlotAppendNew,
    SynthSlotRemove { slot_id: SlotId },
    SetSynth { slot_id: SlotId, synth_id: Option<SynthId> },
    SetEnabled { slot_id: SlotId, flag: bool },
    SetKeyTransform { slot_id: SlotId, transform: MidiKeyTransform },
    SetChannelFilter { slot_id: SlotId, filter: MidiFilter },
    SetControlChangeFilter { slot_id: SlotId, filter: MidiFilter },
    SetUseGlobalTransposition { slot_id: SlotId, flag: bool },
}

pub struct AudioBlocks {
    pub main: AudioBlock,
    pub temp: AudioBlock,
}

pub struct Project {
    pub synth_graph: synth_graph::rt::Graph,
    pub dummy_graph: dummy_graph::rt::Graph,
    pub fluid_graph: fluid_graph::rt::Graph,
    pub sfizz_graph: sfizz_graph::rt::Graph,
}

pub struct Engine {
    pub project: Project,
    pub audio_blocks: Option<AudioBlocks>,
    pub midi_rx: RtConsumer<MidiEvent>,
    pub cmd_rx: RtConsumer<(u32, Command)>,
    pub reply_tx: RtProducer<(u32, bool)>,
}

impl Engine {
    pub fn init(
        &mut self,
        channel_count: usize,
        max_sample_count_per_channel: usize,
        sample_rate: u32,
    ) -> synths::Result<()> {
        self.silence();
        self.create_buffers(channel_count, max_sample_count_per_channel);
        self.set_sample_rate(sample_rate)?;
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

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> synths::Result<()> {
        self.project.dummy_graph.set_sample_rate(sample_rate);
        self.project.fluid_graph.set_sample_rate(sample_rate)?;
        self.project.sfizz_graph.set_sample_rate(sample_rate)?;
        Ok(())
    }

    pub fn create_buffers(&mut self, channel_count: usize, max_sample_count_per_channel: usize) {
        let main = AudioBlock::new(channel_count, max_sample_count_per_channel);
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
        self.project.dummy_graph.silence();
        self.project.fluid_graph.silence();
        self.project.sfizz_graph.silence();
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

            Command::DummyPush { synth } => self.project.dummy_graph.push(synth),
            Command::DummyRemove { synth_id } => {
                let removed = self.project.dummy_graph.remove(synth_id);
                if removed {
                    self.project
                        .synth_graph
                        .remove_synth_references(SynthId { kind: SynthKind::Dummy, id: synth_id });
                }
                removed
            }

            Command::FluidPush { synth } => self.project.fluid_graph.push(synth),
            Command::FluidReplace { synth_id, synth } => {
                self.project.fluid_graph.replace(synth_id, synth)
            }
            Command::FluidRemove { synth_id } => {
                let removed = self.project.fluid_graph.remove(synth_id);
                if removed {
                    self.project
                        .synth_graph
                        .remove_synth_references(SynthId { kind: SynthKind::Fluid, id: synth_id });
                }
                removed
            }
            Command::FluidSelectProgram { synth_id, bank, preset_id } => {
                self.project.fluid_graph.select_program(synth_id, bank, preset_id)
            }

            Command::SfizzPush { synth } => self.project.sfizz_graph.push(synth),
            Command::SfizzReplace { synth_id, synth } => {
                self.project.sfizz_graph.replace(synth_id, synth)
            }
            Command::SfizzRemove { synth_id } => {
                let removed = self.project.sfizz_graph.remove(synth_id);
                if removed {
                    self.project
                        .synth_graph
                        .remove_synth_references(SynthId { kind: SynthKind::Dummy, id: synth_id });
                }
                removed
            }

            Command::SetGlobalTransposition { transposition } => {
                self.project.synth_graph.set_global_transposition(transposition);
                true
            }

            Command::SynthSlotAppendNew => self.project.synth_graph.append_new(),
            Command::SynthSlotRemove { slot_id } => self.project.synth_graph.remove(slot_id),
            Command::SetSynth { slot_id, synth_id } => {
                self.project.synth_graph.set_synth(slot_id, synth_id)
            }
            Command::SetEnabled { slot_id, flag } => {
                self.project.synth_graph.set_enabled(slot_id, flag)
            }
            Command::SetKeyTransform { slot_id, transform } => {
                self.project.synth_graph.set_key_transform(slot_id, transform)
            }
            Command::SetChannelFilter { slot_id, filter } => {
                self.project.synth_graph.set_channel_filter(slot_id, filter)
            }
            Command::SetControlChangeFilter { slot_id, filter } => {
                self.project.synth_graph.set_cc_filter(slot_id, filter)
            }
            Command::SetUseGlobalTransposition { slot_id, flag } => {
                self.project.synth_graph.set_use_global_transposition(slot_id, flag)
            }
        }
    }

    fn dispatch_midi_events(&mut self) {
        while let Ok(event) = self.midi_rx.pop() {
            self.dispatch_midi_event(event);
        }
    }

    fn dispatch_midi_event(&mut self, event: MidiEvent) {
        self.project.synth_graph.midi_handler_iter(event).for_each(
            |(SynthId { kind, id }, event)| match kind {
                SynthKind::Dummy => self.project.dummy_graph.handle_midi_event(id, event),
                SynthKind::Fluid => {
                    let _ = self.project.fluid_graph.handle_midi_event(id, event);
                }
                SynthKind::Sfizz => self.project.sfizz_graph.handle_midi_event(id, event),
            },
        );
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

        self.project.synth_graph.audio_renderer_iter().for_each(|SynthId { kind, id }| {
            match kind {
                SynthKind::Dummy => {
                    self.project.dummy_graph.render_audio_block(id, &mut audio_blocks.temp)
                }
                SynthKind::Fluid => {
                    let _ = self.project.fluid_graph.render_audio_block(id, &mut audio_blocks.temp);
                }
                SynthKind::Sfizz => {
                    let _ = self.project.sfizz_graph.render_audio_block(id, &mut audio_blocks.temp);
                }
            }
            audio_blocks.main.add_samples_from_unchecked(&audio_blocks.temp);
        });

        audio_blocks.main.write_interleaved_f32(buffer);
    }
}
