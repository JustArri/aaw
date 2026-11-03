use crate::{
    engine::graph::ControllerId,
    midi::{MidiEvent, MidiEventKind},
};
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const TICKS_PER_BEAT: u32 = 960;
pub const DEFAULT_STEP_TICKS: u32 = TICKS_PER_BEAT / 4;
pub const MAX_SCHEDULED_EVENTS_PER_BLOCK: usize = 256;

#[derive(Debug, Clone, Copy)]
pub struct DrumScheduleContext {
    pub block_start: u64,
    pub frame_count: usize,
    pub sample_rate: f64,
    pub tempo_bpm: f64,
    pub segment_start_frame: u64,
    pub segment_start_tick: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct DrumLaneId(pub u32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum DrumGateMode {
    Gate,
    OneShot,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DrumLane {
    pub id: DrumLaneId,
    pub name: String,
    pub output_note: u8,
    pub output_channel: u8,
    pub velocity: u8,
    pub gate_mode: DrumGateMode,
    #[schema(required)]
    pub choke_group: Option<u8>,
    pub muted: bool,
    pub solo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DrumStep {
    pub enabled: bool,
    pub velocity: u8,
    pub micro_offset_ticks: i32,
    pub gate_length_ticks: u32,
}

impl Default for DrumStep {
    fn default() -> Self {
        Self {
            enabled: false,
            velocity: 100,
            micro_offset_ticks: 0,
            gate_length_ticks: DEFAULT_STEP_TICKS / 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DrumLaneSequence {
    pub lane_id: DrumLaneId,
    pub length_steps: u32,
    pub step_resolution_ticks: u32,
    pub steps: Vec<DrumStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DrumMachineController {
    pub enabled: bool,
    pub name: String,
    pub lanes: Vec<DrumLane>,
    pub sequences: Vec<DrumLaneSequence>,
}

impl Default for DrumMachineController {
    fn default() -> Self {
        const LANES: [(&str, u8); 8] = [
            ("Kick", 36),
            ("Snare", 38),
            ("Closed hat", 42),
            ("Open hat", 46),
            ("Low tom", 45),
            ("Mid tom", 47),
            ("High tom", 50),
            ("Clap", 39),
        ];
        let lanes: Vec<_> = LANES
            .into_iter()
            .enumerate()
            .map(|(id, (name, note))| DrumLane {
                id: DrumLaneId(id as u32),
                name: name.into(),
                output_note: note,
                output_channel: 9,
                velocity: 127,
                gate_mode: DrumGateMode::OneShot,
                choke_group: None,
                muted: false,
                solo: false,
            })
            .collect();
        let sequences = lanes
            .iter()
            .map(|lane| DrumLaneSequence {
                lane_id: lane.id,
                length_steps: 16,
                step_resolution_ticks: DEFAULT_STEP_TICKS,
                steps: vec![DrumStep::default(); 16],
            })
            .collect();
        Self { enabled: true, name: "Drum machine".into(), lanes, sequences }
    }
}

impl DrumMachineController {
    pub fn is_valid(&self) -> bool {
        if self.lanes.len() > 64
            || self.sequences.len() > 64
            || self.lanes.len() != self.sequences.len()
        {
            return false;
        }
        for (index, lane) in self.lanes.iter().enumerate() {
            if lane.output_note > 127
                || lane.output_channel > 15
                || lane.velocity == 0
                || lane.velocity > 127
                || self.lanes[..index].iter().any(|previous| previous.id == lane.id)
            {
                return false;
            }
        }
        for (sequence_index, sequence) in self.sequences.iter().enumerate() {
            if sequence.length_steps == 0
                || sequence.length_steps > 512
                || sequence.step_resolution_ticks == 0
                || sequence.steps.len() < sequence.length_steps as usize
                || sequence.steps.len() > 512
                || !self.lanes.iter().any(|lane| lane.id == sequence.lane_id)
                || self.sequences[..sequence_index]
                    .iter()
                    .any(|previous| previous.lane_id == sequence.lane_id)
                || sequence
                    .steps
                    .iter()
                    .any(|step| step.velocity > 127 || step.gate_length_ticks == 0)
            {
                return false;
            }
        }
        true
    }

    pub fn resize_to_bar(&mut self, transport: MusicalTransport) -> bool {
        let bar_ticks = transport.ticks_per_bar();
        let mut changed = false;
        for sequence in &mut self.sequences {
            let resolution = u64::from(sequence.step_resolution_ticks);
            if resolution == 0 || !bar_ticks.is_multiple_of(resolution) {
                continue;
            }
            let step_count = bar_ticks / resolution;
            if step_count == 0 || step_count > 512 {
                continue;
            }
            let step_count = step_count as usize;
            if sequence.length_steps as usize != step_count {
                sequence.length_steps = step_count as u32;
                changed = true;
            }
            if sequence.steps.len() < step_count {
                sequence.steps.resize(step_count, DrumStep::default());
                changed = true;
            }
        }
        changed
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct MusicalTransport {
    pub tempo_bpm: f64,
    pub beats_per_bar: u8,
    pub beat_unit: u8,
    pub playing: bool,
}

impl Default for MusicalTransport {
    fn default() -> Self {
        Self { tempo_bpm: 120.0, beats_per_bar: 4, beat_unit: 4, playing: false }
    }
}

impl MusicalTransport {
    pub fn is_valid(self) -> bool {
        self.tempo_bpm.is_finite()
            && (20.0..=400.0).contains(&self.tempo_bpm)
            && (1..=32).contains(&self.beats_per_bar)
            && matches!(self.beat_unit, 1 | 2 | 4 | 8 | 16)
    }

    pub fn ticks_per_meter_beat(self) -> u64 {
        u64::from(TICKS_PER_BEAT) * 4 / u64::from(self.beat_unit)
    }

    pub fn ticks_per_bar(self) -> u64 {
        self.ticks_per_meter_beat() * u64::from(self.beats_per_bar)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CompiledEvent {
    tick: u64,
    channel: u8,
    note: u8,
    velocity: u8,
    note_on: bool,
}

pub struct RtDrumMachine {
    enabled: bool,
    loop_ticks: u64,
    events: Vec<CompiledEvent>,
}

impl RtDrumMachine {
    pub fn compile(controller: &DrumMachineController) -> Self {
        let any_solo = controller.lanes.iter().any(|lane| lane.solo);
        let mut events = Vec::new();
        let mut loop_ticks = 0;
        for sequence in controller.sequences.iter().take(64) {
            let Some(lane) = controller.lanes.iter().find(|lane| lane.id == sequence.lane_id)
            else {
                continue;
            };
            if lane.muted
                || (any_solo && !lane.solo)
                || lane.output_channel > 15
                || lane.output_note > 127
            {
                continue;
            }
            let resolution = sequence.step_resolution_ticks.max(1) as u64;
            let length = sequence.length_steps.max(1) as u64;
            loop_ticks = loop_ticks.max(length.saturating_mul(resolution));
            for (index, step) in sequence.steps.iter().take(length as usize).take(512).enumerate() {
                if !step.enabled || step.velocity == 0 || step.velocity > 127 {
                    continue;
                }
                let base = (index as u64).saturating_mul(resolution);
                let velocity =
                    ((u16::from(step.velocity) * u16::from(lane.velocity) + 63) / 127) as u8;
                let tick = if step.micro_offset_ticks < 0 {
                    base.saturating_sub(step.micro_offset_ticks.unsigned_abs() as u64)
                } else {
                    base.saturating_add(step.micro_offset_ticks as u64)
                };
                events.push(CompiledEvent {
                    tick,
                    channel: lane.output_channel,
                    note: lane.output_note,
                    velocity,
                    note_on: true,
                });
                events.push(CompiledEvent {
                    tick: tick.saturating_add(step.gate_length_ticks.max(1) as u64),
                    channel: lane.output_channel,
                    note: lane.output_note,
                    velocity: 0,
                    note_on: false,
                });
            }
        }
        if loop_ticks > 0 {
            let maximum_tick = loop_ticks.saturating_mul(2).saturating_sub(1);
            for event in &mut events {
                event.tick = event.tick.min(maximum_tick);
            }
        }
        events.sort_unstable_by_key(|event| (event.tick, event.note_on, event.channel, event.note));
        Self { enabled: controller.enabled, loop_ticks, events }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn schedule(
        &self,
        ctx: DrumScheduleContext,
        controller_id: ControllerId,
        output: &mut ArrayVec<ScheduledDrumEvent, MAX_SCHEDULED_EVENTS_PER_BLOCK>,
    ) {
        if !self.enabled || self.loop_ticks == 0 || self.events.is_empty() {
            return;
        }
        let frames_per_tick = ctx.sample_rate * 60.0 / (ctx.tempo_bpm * TICKS_PER_BEAT as f64);
        let block_end = ctx.block_start.saturating_add(ctx.frame_count as u64);
        let start_tick = (ctx.segment_start_tick
            + ctx.block_start.saturating_sub(ctx.segment_start_frame) as f64 / frames_per_tick)
            .floor()
            .max(0.0) as u64;
        let first_loop = start_tick / self.loop_ticks;
        let end_tick = (ctx.segment_start_tick
            + block_end.saturating_sub(ctx.segment_start_frame) as f64 / frames_per_tick)
            .ceil()
            .max(0.0) as u64;
        let last_loop = (end_tick / self.loop_ticks)
            .saturating_add(1)
            .min(first_loop.saturating_add(MAX_SCHEDULED_EVENTS_PER_BLOCK as u64));
        for loop_index in first_loop.saturating_sub(1)..=last_loop {
            let loop_tick = loop_index.saturating_mul(self.loop_ticks);
            for event in &self.events {
                let absolute_tick = loop_tick.saturating_add(event.tick);
                let frame = (ctx.segment_start_frame as f64
                    + (absolute_tick as f64 - ctx.segment_start_tick) * frames_per_tick)
                    .round()
                    .max(0.0) as u64;
                if frame < ctx.block_start || frame >= block_end {
                    continue;
                }
                let midi = MidiEvent {
                    slot: 0,
                    channel: event.channel,
                    kind: if event.note_on {
                        MidiEventKind::NoteOn { key: event.note, velocity: event.velocity }
                    } else {
                        MidiEventKind::NoteOff { key: event.note, velocity: 0 }
                    },
                };
                if output
                    .try_push(ScheduledDrumEvent {
                        absolute_frame: frame,
                        sample_offset: (frame - ctx.block_start) as usize,
                        controller_id,
                        event: midi,
                    })
                    .is_err()
                {
                    return;
                }
            }
        }
    }
}

pub struct RtDrumMachineSlot {
    pub active: Option<RtDrumMachine>,
    pub pending: Option<RtDrumMachine>,
    pub retired: Option<RtDrumMachine>,
    pub active_notes: [u128; 16],
}

impl RtDrumMachineSlot {
    pub fn new(active: Option<RtDrumMachine>) -> Self {
        Self { active, pending: None, retired: None, active_notes: [0; 16] }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScheduledDrumEvent {
    pub absolute_frame: u64,
    pub sample_offset: usize,
    pub controller_id: ControllerId,
    pub event: MidiEvent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduling_is_sample_clock_aligned_without_drift() {
        let mut controller = DrumMachineController::default();
        controller.sequences[0].steps[0].enabled = true;
        controller.sequences[0].steps[4].enabled = true;
        let machine = RtDrumMachine::compile(&controller);
        let mut events = ArrayVec::new();
        machine.schedule(
            DrumScheduleContext {
                block_start: 0,
                frame_count: 120_001,
                sample_rate: 48_000.0,
                tempo_bpm: 120.0,
                segment_start_frame: 0,
                segment_start_tick: 0.0,
            },
            ControllerId(0),
            &mut events,
        );
        let note_ons: Vec<_> = events
            .iter()
            .filter(|event| matches!(event.event.kind, MidiEventKind::NoteOn { .. }))
            .map(|event| event.sample_offset)
            .collect();
        assert_eq!(note_ons, [0, 24_000, 96_000, 120_000]);
    }

    #[test]
    fn muted_lane_does_not_compile_events() {
        let mut controller = DrumMachineController::default();
        controller.lanes[0].muted = true;
        controller.sequences[0].steps[0].enabled = true;
        let machine = RtDrumMachine::compile(&controller);
        let mut events = ArrayVec::new();
        machine.schedule(
            DrumScheduleContext {
                block_start: 0,
                frame_count: 256,
                sample_rate: 48_000.0,
                tempo_bpm: 120.0,
                segment_start_frame: 0,
                segment_start_tick: 0.0,
            },
            ControllerId(0),
            &mut events,
        );
        assert!(events.is_empty());
    }

    #[test]
    fn enabled_state_can_change_without_recompiling() {
        let mut controller = DrumMachineController::default();
        controller.sequences[0].steps[0].enabled = true;
        let mut machine = RtDrumMachine::compile(&controller);
        let context = DrumScheduleContext {
            block_start: 0,
            frame_count: 256,
            sample_rate: 48_000.0,
            tempo_bpm: 120.0,
            segment_start_frame: 0,
            segment_start_tick: 0.0,
        };

        machine.set_enabled(false);
        let mut disabled_events = ArrayVec::new();
        machine.schedule(context, ControllerId(0), &mut disabled_events);
        assert!(disabled_events.is_empty());

        machine.set_enabled(true);
        let mut enabled_events = ArrayVec::new();
        machine.schedule(context, ControllerId(0), &mut enabled_events);
        assert!(!enabled_events.is_empty());
    }

    #[test]
    fn lane_velocity_scales_step_velocity() {
        let mut controller = DrumMachineController::default();
        controller.lanes[0].velocity = 64;
        controller.sequences[0].steps[0].enabled = true;
        controller.sequences[0].steps[0].velocity = 100;
        let machine = RtDrumMachine::compile(&controller);
        let note_on = machine.events.iter().find(|event| event.note_on).expect("note-on event");
        assert_eq!(note_on.velocity, 50);
    }

    #[test]
    fn controller_can_have_no_lanes() {
        let mut controller = DrumMachineController::default();
        controller.lanes.clear();
        controller.sequences.clear();
        assert!(controller.is_valid());
    }

    #[test]
    fn scheduling_is_independent_of_buffer_size() {
        let mut controller = DrumMachineController::default();
        controller.sequences[0].steps[1].enabled = true;
        controller.sequences[0].steps[9].enabled = true;
        let machine = RtDrumMachine::compile(&controller);
        let context = DrumScheduleContext {
            block_start: 0,
            frame_count: 200_000,
            sample_rate: 44_100.0,
            tempo_bpm: 137.0,
            segment_start_frame: 0,
            segment_start_tick: 0.0,
        };
        let mut whole = ArrayVec::new();
        machine.schedule(context, ControllerId(3), &mut whole);

        let mut chunked = Vec::new();
        let mut block_start = 0;
        while block_start < context.frame_count as u64 {
            let mut events = ArrayVec::new();
            machine.schedule(
                DrumScheduleContext {
                    block_start,
                    frame_count: 127.min(context.frame_count - block_start as usize),
                    ..context
                },
                ControllerId(3),
                &mut events,
            );
            chunked.extend(events.into_iter().map(|event| event.absolute_frame));
            block_start += 127;
        }
        assert_eq!(whole.iter().map(|event| event.absolute_frame).collect::<Vec<_>>(), chunked);
    }

    #[test]
    fn controller_configuration_round_trips_through_json() {
        let mut controller = DrumMachineController::default();
        controller.sequences[2].steps[7].enabled = true;
        controller.sequences[2].steps[7].velocity = 73;
        let json = serde_json::to_value(&controller).expect("serialize drum machine");
        let restored: DrumMachineController =
            serde_json::from_value(json.clone()).expect("deserialize drum machine");
        assert_eq!(serde_json::to_value(restored).expect("serialize restored drum machine"), json);
    }

    #[test]
    fn time_signature_controls_beat_and_bar_lengths() {
        let transport = MusicalTransport::default();
        assert_eq!(transport.ticks_per_meter_beat(), 960);
        assert_eq!(transport.ticks_per_bar(), 3_840);

        let three_four = MusicalTransport { beats_per_bar: 3, ..transport };
        assert_eq!(three_four.ticks_per_bar(), 2_880);

        let six_eight = MusicalTransport { beats_per_bar: 6, beat_unit: 8, ..transport };
        assert_eq!(six_eight.ticks_per_meter_beat(), 480);
        assert_eq!(six_eight.ticks_per_bar(), 2_880);
    }

    #[test]
    fn one_bar_grid_resizes_without_discarding_hidden_steps() {
        let mut controller = DrumMachineController::default();
        controller.sequences[0].steps[15].enabled = true;

        assert!(
            controller.resize_to_bar(MusicalTransport {
                beats_per_bar: 3,
                ..MusicalTransport::default()
            })
        );
        assert_eq!(controller.sequences[0].length_steps, 12);
        assert_eq!(controller.sequences[0].steps.len(), 16);

        assert!(
            controller.resize_to_bar(MusicalTransport {
                beats_per_bar: 5,
                ..MusicalTransport::default()
            })
        );
        assert_eq!(controller.sequences[0].length_steps, 20);
        assert_eq!(controller.sequences[0].steps.len(), 20);
        assert!(controller.sequences[0].steps[15].enabled);
    }
}
