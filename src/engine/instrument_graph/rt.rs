use crate::{
    DEFAULT_SAMPLE_RATE,
    audio::AudioBlock,
    engine::{
        drum_machine::{
            DrumScheduleContext, MAX_SCHEDULED_EVENTS_PER_BLOCK, RtDrumMachine, RtDrumMachineSlot,
            ScheduledDrumEvent, TICKS_PER_BEAT,
        },
        effects::RtEffects,
        graph::{InstrumentId, RETIREMENT_QUEUE_CAPACITY},
        instrument_graph::InstrumentGraphError,
    },
    memory::{
        Siv,
        retirement::{RetirementThread, Retirer},
    },
    midi::MidiEvent,
    synths::{self, SoundFontProgram},
};
use arrayvec::ArrayVec;
use std::time::Duration;

pub struct RtInstrument {
    pub kind: RtInstrumentKind,
    pub effects: RtEffects,
    pub gain: f32,
    pub drum_machines: Siv<RtDrumMachineSlot, crate::engine::graph::ControllerId>,
}

#[allow(clippy::large_enum_variant)]
pub enum RtInstrumentKind {
    Dummy(synths::Dummy),
    Fluid(synths::Fluid),
    Sfizz(synths::Sfizz),
    Clap(synths::Clap),
}

impl RtInstrument {
    pub fn silence(&mut self) {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.silence(),
            RtInstrumentKind::Fluid(instrument) => {
                let _ = instrument.silence();
            }
            RtInstrumentKind::Sfizz(instrument) => instrument.silence(),
            RtInstrumentKind::Clap(instrument) => instrument.silence(),
        }
    }

    pub async fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), InstrumentGraphError> {
        self.effects.reconfigure(channel_count, sample_rate)?;
        if let RtInstrumentKind::Clap(instrument) = &mut self.kind {
            instrument.reconfigure(channel_count, sample_rate, max_block_size).await?;
        } else {
            self.set_sample_rate(sample_rate)?;
            self.set_max_block_size(max_block_size)?;
        }
        Ok(())
    }

    pub fn handle_midi_event(&mut self, event: MidiEvent) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.handle_midi_event(event),
            RtInstrumentKind::Fluid(instrument) => instrument.handle_midi_event(event)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.handle_midi_event(event),
            RtInstrumentKind::Clap(instrument) => instrument.handle_midi_event(event),
        }
        Ok(())
    }

    pub fn render_audio_block(
        &mut self,
        output: &mut AudioBlock,
    ) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.render_audio_block(output),
            RtInstrumentKind::Fluid(instrument) => instrument.render_audio_block(output)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.render_audio_block(output)?,
            RtInstrumentKind::Clap(instrument) => instrument.render_audio_block(output)?,
        }
        self.effects.apply(output);
        output.multiply_samples(self.gain);
        Ok(())
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.set_sample_rate(sample_rate),
            RtInstrumentKind::Fluid(instrument) => instrument.set_sample_rate(sample_rate)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.set_sample_rate(sample_rate)?,
            RtInstrumentKind::Clap(_) => {}
        }
        Ok(())
    }

    fn set_max_block_size(&mut self, size: usize) -> Result<(), InstrumentGraphError> {
        match &mut self.kind {
            RtInstrumentKind::Dummy(instrument) => instrument.set_max_block_size(size),
            RtInstrumentKind::Fluid(instrument) => instrument.set_max_block_size(size)?,
            RtInstrumentKind::Sfizz(instrument) => instrument.set_max_block_size(size)?,
            RtInstrumentKind::Clap(_) => {}
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
pub enum RetiredObject {
    Instrument(RtInstrument),
    InstrumentKind(RtInstrumentKind),
    DrumMachine(RtDrumMachine),
}

pub struct RtInstrumentGraph {
    pub retirer: Retirer<RetiredObject>,
    pub retirement_thread: RetirementThread,
    pub instruments: Siv<RtInstrument, InstrumentId>,
    pub global_effects: RtEffects,
    pub target_global_gain: f32,
    pub global_gain: f32,
    pub sample_rate: f32,
}

impl RtInstrumentGraph {
    pub fn new(
        instruments: Siv<RtInstrument, InstrumentId>,
        global_effects: RtEffects,
        global_gain: f32,
    ) -> Self {
        let (retirer, retirement_thread) =
            RetirementThread::spawn(RETIREMENT_QUEUE_CAPACITY, Duration::from_millis(10))
                .expect("failed to spawn retirement thread");
        Self {
            retirer,
            retirement_thread,
            instruments,
            global_effects,
            target_global_gain: global_gain,
            global_gain,
            sample_rate: DEFAULT_SAMPLE_RATE as f32,
        }
    }

    pub fn destroy(self) {
        let _ = self.retirement_thread.join_after_dropping(self.retirer);
    }

    pub fn silence(&mut self) {
        for instrument in self.instruments.iter_values_mut() {
            instrument.silence();
            for machine in instrument.drum_machines.iter_values_mut() {
                machine.active_notes.fill(0);
            }
        }
    }

    pub async fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> Result<(), InstrumentGraphError> {
        self.sample_rate = sample_rate as f32;
        for instrument in self.instruments.iter_values_mut() {
            instrument.reconfigure(channel_count, sample_rate, max_block_size).await?;
        }
        Ok(())
    }

    pub fn handle_midi_event(
        &mut self,
        id: InstrumentId,
        event: MidiEvent,
    ) -> Result<(), InstrumentGraphError> {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.handle_midi_event(event)?;
        }
        Ok(())
    }

    pub fn push(&mut self, instrument: RtInstrument) -> bool {
        if self.instruments.len() == self.instruments.capacity() {
            self.retirer.retire_or_leak(RetiredObject::Instrument(instrument));
            return false;
        }
        self.instruments.push(instrument);
        true
    }

    pub fn replace(&mut self, id: InstrumentId, kind: RtInstrumentKind) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            let previous = std::mem::replace(&mut instrument.kind, kind);
            self.retirer.retire_or_leak(RetiredObject::InstrumentKind(previous));
            true
        } else {
            self.retirer.retire_or_leak(RetiredObject::InstrumentKind(kind));
            false
        }
    }

    pub fn remove(&mut self, id: InstrumentId) -> bool {
        if let Some(instrument) = self.instruments.try_remove(id) {
            self.retirer.retire_or_leak(RetiredObject::Instrument(instrument));
            true
        } else {
            false
        }
    }

    pub fn select_soundfont_program(
        &mut self,
        id: InstrumentId,
        program: SoundFontProgram,
    ) -> bool {
        let Some(instrument) = self.instruments.get_mut(id) else {
            return false;
        };
        match &mut instrument.kind {
            RtInstrumentKind::Fluid(instrument) => instrument.select_program(program).is_ok(),
            _ => false,
        }
    }

    pub fn set_clap_parameter(&mut self, id: InstrumentId, parameter_id: u32, value: f64) -> bool {
        let Some(instrument) = self.instruments.get_mut(id) else {
            return false;
        };
        let RtInstrumentKind::Clap(instrument) = &mut instrument.kind else {
            return false;
        };
        instrument.set_parameter(parameter_id, value)
    }

    pub fn set_effects(&mut self, id: InstrumentId, effects: RtEffects) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.effects = effects;
            true
        } else {
            false
        }
    }

    pub fn set_gain(&mut self, id: InstrumentId, gain: f32) -> bool {
        if let Some(instrument) = self.instruments.get_mut(id) {
            instrument.gain = gain;
            true
        } else {
            false
        }
    }

    pub fn set_drum_machine(
        &mut self,
        address: crate::engine::graph::ControllerAddress,
        machine: Option<RtDrumMachine>,
    ) -> bool {
        let Some(instrument) = self.instruments.get_mut(address.instrument_id) else {
            if let Some(machine) = machine {
                self.retirer.retire_or_leak(RetiredObject::DrumMachine(machine));
            }
            return false;
        };
        let mut notes_to_release = None;
        if let Some(slot) = instrument.drum_machines.get_mut(address.controller_id) {
            for previous in [slot.retired.take(), slot.pending.take()].into_iter().flatten() {
                self.retirer.retire_or_leak(RetiredObject::DrumMachine(previous));
            }
            if machine.is_some() && slot.active.is_some() {
                slot.pending = machine;
            } else {
                if machine.is_none() {
                    notes_to_release = Some(slot.active_notes);
                    slot.active_notes.fill(0);
                }
                if let Some(previous) = std::mem::replace(&mut slot.active, machine) {
                    self.retirer.retire_or_leak(RetiredObject::DrumMachine(previous));
                }
            }
            if let Some(notes) = notes_to_release {
                release_owned_notes(instrument, notes);
            }
            true
        } else {
            false
        }
    }

    pub fn set_drum_machine_enabled(
        &mut self,
        address: crate::engine::graph::ControllerAddress,
        enabled: bool,
    ) -> bool {
        let Some(instrument) = self.instruments.get_mut(address.instrument_id) else {
            return false;
        };
        let Some(slot) = instrument.drum_machines.get_mut(address.controller_id) else {
            return false;
        };
        let mut found = false;
        if let Some(machine) = &mut slot.active {
            machine.set_enabled(enabled);
            found = true;
        }
        if let Some(machine) = &mut slot.pending {
            machine.set_enabled(enabled);
            found = true;
        }
        if !found {
            return false;
        }
        if !enabled {
            let notes = slot.active_notes;
            slot.active_notes.fill(0);
            release_owned_notes(instrument, notes);
        }
        true
    }

    pub fn append_controller_slot(&mut self, instrument_id: InstrumentId) -> bool {
        let Some(instrument) = self.instruments.get_mut(instrument_id) else {
            return false;
        };
        instrument.drum_machines.push(RtDrumMachineSlot::new(None));
        true
    }

    pub fn remove_controller_slot(
        &mut self,
        address: crate::engine::graph::ControllerAddress,
    ) -> bool {
        let Some(instrument) = self.instruments.get_mut(address.instrument_id) else {
            return false;
        };
        let Some(mut slot) = instrument.drum_machines.try_remove(address.controller_id) else {
            return false;
        };
        release_owned_notes(instrument, slot.active_notes);
        for machine in
            [slot.active.take(), slot.pending.take(), slot.retired.take()].into_iter().flatten()
        {
            self.retirer.retire_or_leak(RetiredObject::DrumMachine(machine));
        }
        true
    }

    pub fn schedule_drum_events(
        instrument: &mut RtInstrument,
        ctx: DrumScheduleContext,
        bar_ticks: u64,
        output: &mut ArrayVec<ScheduledDrumEvent, MAX_SCHEDULED_EVENTS_PER_BLOCK>,
    ) {
        for (controller_id, slot) in &mut instrument.drum_machines {
            let Some(active) = &slot.active else {
                continue;
            };
            if slot.pending.is_some() {
                let frames_per_tick =
                    ctx.sample_rate * 60.0 / (ctx.tempo_bpm * TICKS_PER_BEAT as f64);
                let start_tick = ctx.segment_start_tick
                    + ctx.block_start.saturating_sub(ctx.segment_start_frame) as f64
                        / frames_per_tick;
                let quantum = bar_ticks as f64;
                let boundary_tick = (start_tick / quantum).ceil() * quantum;
                let boundary_frame = (ctx.segment_start_frame as f64
                    + (boundary_tick - ctx.segment_start_tick) * frames_per_tick)
                    .round()
                    .max(0.0) as u64;
                let block_end = ctx.block_start.saturating_add(ctx.frame_count as u64);
                if boundary_frame < block_end {
                    let mut old_events = ArrayVec::new();
                    active.schedule(ctx, *controller_id, &mut old_events);
                    let mut boundary_notes = slot.active_notes;
                    for event in old_events {
                        if event.absolute_frame < boundary_frame {
                            update_owned_notes(&mut boundary_notes, event.event);
                            if output.try_push(event).is_err() {
                                return;
                            }
                        }
                    }
                    for (channel, notes) in boundary_notes.into_iter().enumerate() {
                        for note in 0..128 {
                            if notes & (1u128 << note) == 0 {
                                continue;
                            }
                            let release = ScheduledDrumEvent {
                                absolute_frame: boundary_frame,
                                sample_offset: boundary_frame.saturating_sub(ctx.block_start)
                                    as usize,
                                controller_id: *controller_id,
                                event: MidiEvent {
                                    slot: 0,
                                    channel: channel as u8,
                                    kind: crate::midi::MidiEventKind::NoteOff {
                                        key: note,
                                        velocity: 0,
                                    },
                                },
                            };
                            if output.try_push(release).is_err() {
                                return;
                            }
                        }
                    }
                    let previous =
                        slot.active.replace(slot.pending.take().expect("pending machine vanished"));
                    slot.retired = previous;
                    let mut new_events = ArrayVec::new();
                    slot.active.as_ref().expect("active machine vanished").schedule(
                        ctx,
                        *controller_id,
                        &mut new_events,
                    );
                    for event in new_events {
                        if event.absolute_frame >= boundary_frame && output.try_push(event).is_err()
                        {
                            return;
                        }
                    }
                    continue;
                }
            }
            active.schedule(ctx, *controller_id, output);
        }
        output.sort_unstable_by_key(|scheduled| {
            let note_on = matches!(scheduled.event.kind, crate::midi::MidiEventKind::NoteOn { .. });
            let note = match scheduled.event.kind {
                crate::midi::MidiEventKind::NoteOn { key, .. }
                | crate::midi::MidiEventKind::NoteOff { key, .. } => key,
                _ => 0,
            };
            (
                scheduled.sample_offset,
                note_on,
                scheduled.controller_id,
                scheduled.event.channel,
                note,
            )
        });
    }
}

fn release_owned_notes(instrument: &mut RtInstrument, notes: [u128; 16]) {
    for (channel, notes) in notes.into_iter().enumerate() {
        for note in 0..128 {
            if notes & (1u128 << note) != 0 {
                let _ = instrument.handle_midi_event(MidiEvent {
                    slot: 0,
                    channel: channel as u8,
                    kind: crate::midi::MidiEventKind::NoteOff { key: note, velocity: 0 },
                });
            }
        }
    }
}

fn update_owned_notes(notes: &mut [u128; 16], event: MidiEvent) {
    let channel = event.channel as usize;
    match event.kind {
        crate::midi::MidiEventKind::NoteOn { key, velocity } if velocity > 0 => {
            notes[channel] |= 1u128 << key;
        }
        crate::midi::MidiEventKind::NoteOff { key, .. }
        | crate::midi::MidiEventKind::NoteOn { key, velocity: 0 } => {
            notes[channel] &= !(1u128 << key);
        }
        _ => {}
    }
}
