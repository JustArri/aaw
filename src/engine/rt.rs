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
    Panic,

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

pub struct Engine {
    pub synth_graph: synth_graph::rt::Graph,
    pub dummy_graph: dummy_graph::rt::Graph,
    pub fluid_graph: fluid_graph::rt::Graph,
    pub sfizz_graph: sfizz_graph::rt::Graph,
    pub audio_blocks: Option<AudioBlocks>,
    pub midi_rx: RtConsumer<MidiEvent>,
    pub cmd_rx: RtConsumer<(u32, Command)>,
    pub reply_tx: RtProducer<(u32, bool)>,
}

impl Engine {
    pub fn run(&mut self, buffer: &mut [f32]) {
        self.dispatch_commands();
        self.dispatch_midi_events();
        self.render_next_audio_block(buffer);
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.dummy_graph.set_sample_rate(sample_rate);
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

    fn dispatch_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Panic => {
                self.dummy_graph.panic();
                self.fluid_graph.panic();
                self.sfizz_graph.panic();
                true
            }

            Command::DummyPush { synth } => self.dummy_graph.push(synth),
            Command::DummyRemove { synth_id } => {
                let removed = self.dummy_graph.remove(synth_id);
                if removed {
                    self.synth_graph.remove_synth_references(SynthId(SynthKind::Dummy, synth_id));
                }
                removed
            }

            Command::FluidPush { synth } => self.fluid_graph.push(synth),
            Command::FluidReplace { synth_id, synth } => self.fluid_graph.replace(synth_id, synth),
            Command::FluidRemove { synth_id } => {
                let removed = self.fluid_graph.remove(synth_id);
                if removed {
                    self.synth_graph.remove_synth_references(SynthId(SynthKind::Fluid, synth_id));
                }
                removed
            }
            Command::FluidSelectProgram { synth_id, bank, preset_id } => {
                self.fluid_graph.select_program(synth_id, bank, preset_id)
            }

            Command::SfizzPush { synth } => self.sfizz_graph.push(synth),
            Command::SfizzReplace { synth_id, synth } => self.sfizz_graph.replace(synth_id, synth),
            Command::SfizzRemove { synth_id } => {
                let removed = self.sfizz_graph.remove(synth_id);
                if removed {
                    self.synth_graph.remove_synth_references(SynthId(SynthKind::Dummy, synth_id));
                }
                removed
            }

            Command::SetGlobalTransposition { transposition } => {
                self.synth_graph.set_global_transposition(transposition);
                true
            }

            Command::SynthSlotAppendNew => self.synth_graph.append_new(),
            Command::SynthSlotRemove { slot_id } => self.synth_graph.remove(slot_id),
            Command::SetSynth { slot_id, synth_id } => {
                self.synth_graph.set_synth(slot_id, synth_id)
            }
            Command::SetEnabled { slot_id, flag } => self.synth_graph.set_enabled(slot_id, flag),
            Command::SetKeyTransform { slot_id, transform } => {
                self.synth_graph.set_key_transform(slot_id, transform)
            }
            Command::SetChannelFilter { slot_id, filter } => {
                self.synth_graph.set_channel_filter(slot_id, filter)
            }
            Command::SetControlChangeFilter { slot_id, filter } => {
                self.synth_graph.set_cc_filter(slot_id, filter)
            }
            Command::SetUseGlobalTransposition { slot_id, flag } => {
                self.synth_graph.set_use_global_transposition(slot_id, flag)
            }
        }
    }

    fn dispatch_midi_events(&mut self) {
        while let Ok(event) = self.midi_rx.pop() {
            self.dispatch_midi_event(event);
        }
    }

    fn dispatch_midi_event(&mut self, event: MidiEvent) {
        self.synth_graph.midi_handler_iter(event).for_each(
            |(SynthId(synth_kind, local_id), event)| match synth_kind {
                SynthKind::Dummy => self.dummy_graph.handle_midi_event(local_id, event),
                SynthKind::Fluid => {
                    let _ = self.fluid_graph.handle_midi_event(local_id, event);
                }
                SynthKind::Sfizz => self.sfizz_graph.handle_midi_event(local_id, event),
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

        self.synth_graph.audio_renderer_iter().for_each(|SynthId(synth_kind, local_id)| {
            match synth_kind {
                SynthKind::Dummy => {
                    self.dummy_graph.render_audio_block(local_id, &mut audio_blocks.temp)
                }
                SynthKind::Fluid => {
                    let _ = self.fluid_graph.render_audio_block(local_id, &mut audio_blocks.temp);
                }
                SynthKind::Sfizz => {
                    let _ = self.sfizz_graph.render_audio_block(local_id, &mut audio_blocks.temp);
                }
            }
            audio_blocks.main.add_samples_from_unchecked(&audio_blocks.temp);
        });

        audio_blocks.main.write_interleaved_f32(buffer);
    }
}
