use crate::midi::{MidiEvent, MidiEventKind};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use utoipa::ToSchema;

const LOAD_SCALE: u64 = 1_000_000;
const MAX_MONITORED_INSTRUMENTS: usize = 1_024;
const DEADLINE_MISS_HISTORY_SIZE: usize = 32;
const PERFORMANCE_HISTORY_SIZE: usize = 120;
const NO_INSTRUMENT: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, Default)]
pub struct CallbackDetails {
    pub lock_wait_ns: u64,
    pub commands_ns: u64,
    pub midi_ns: u64,
    pub instruments_ns: u64,
    pub global_processing_ns: u64,
    pub output_conversion_ns: u64,
    pub midi_event_count: u64,
    pub last_midi_event: Option<CallbackMidiEvent>,
    pub slowest_instrument_id: Option<usize>,
    pub slowest_instrument_ns: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct CallbackMidiEvent {
    pub instrument_id: usize,
    pub event: MidiEvent,
}

#[derive(Default)]
struct DeadlineMissSlot {
    sequence: AtomicU64,
    callback_index: AtomicU64,
    frames: AtomicU64,
    budget_ns: AtomicU64,
    total_ns: AtomicU64,
    lock_wait_ns: AtomicU64,
    commands_ns: AtomicU64,
    midi_ns: AtomicU64,
    instruments_ns: AtomicU64,
    global_processing_ns: AtomicU64,
    output_conversion_ns: AtomicU64,
    midi_event_count: AtomicU64,
    midi_instrument_id: AtomicU64,
    midi_event: AtomicU64,
    slowest_instrument_id: AtomicU64,
    slowest_instrument_ns: AtomicU64,
}

#[derive(Default)]
struct PerformanceHistorySlot {
    sequence: AtomicU64,
    callback_index: AtomicU64,
    average_load_scaled: AtomicU64,
    peak_load_scaled: AtomicU64,
    average_render_time_ns: AtomicU64,
    max_render_time_ns: AtomicU64,
    average_budget_ns: AtomicU64,
    deadline_miss_count: AtomicU64,
}

#[derive(Default)]
struct TimingMetric {
    total_ns: AtomicU64,
    last_ns: AtomicU64,
    max_ns: AtomicU64,
    count: AtomicU64,
}

impl TimingMetric {
    fn reset(&self) {
        self.total_ns.store(0, Ordering::Relaxed);
        self.last_ns.store(0, Ordering::Relaxed);
        self.max_ns.store(0, Ordering::Relaxed);
        self.count.store(0, Ordering::Relaxed);
    }

    fn record(&self, elapsed_ns: u64) {
        self.total_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
        self.last_ns.store(elapsed_ns, Ordering::Relaxed);
        self.max_ns.fetch_max(elapsed_ns, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> TimingMetricSnapshot {
        let count = self.count.load(Ordering::Relaxed);
        let total_ns = self.total_ns.load(Ordering::Relaxed);
        TimingMetricSnapshot {
            average_time_us: nanos_to_micros(total_ns.checked_div(count).unwrap_or(0)),
            last_time_us: nanos_to_micros(self.last_ns.load(Ordering::Relaxed)),
            max_time_us: nanos_to_micros(self.max_ns.load(Ordering::Relaxed)),
        }
    }
}

impl DeadlineMissSlot {
    fn write(
        &self,
        sequence: u64,
        callback_index: u64,
        frames: u64,
        budget_ns: u64,
        total_ns: u64,
        details: CallbackDetails,
    ) {
        self.sequence.store(0, Ordering::Release);
        self.callback_index.store(callback_index, Ordering::Relaxed);
        self.frames.store(frames, Ordering::Relaxed);
        self.budget_ns.store(budget_ns, Ordering::Relaxed);
        self.total_ns.store(total_ns, Ordering::Relaxed);
        self.lock_wait_ns.store(details.lock_wait_ns, Ordering::Relaxed);
        self.commands_ns.store(details.commands_ns, Ordering::Relaxed);
        self.midi_ns.store(details.midi_ns, Ordering::Relaxed);
        self.instruments_ns.store(details.instruments_ns, Ordering::Relaxed);
        self.global_processing_ns.store(details.global_processing_ns, Ordering::Relaxed);
        self.output_conversion_ns.store(details.output_conversion_ns, Ordering::Relaxed);
        self.midi_event_count.store(details.midi_event_count, Ordering::Relaxed);
        self.midi_instrument_id.store(
            details.last_midi_event.map_or(NO_INSTRUMENT, |event| event.instrument_id as u64),
            Ordering::Relaxed,
        );
        self.midi_event.store(
            details.last_midi_event.map_or(0, |event| encode_midi_event(event.event)),
            Ordering::Relaxed,
        );
        self.slowest_instrument_id.store(
            details.slowest_instrument_id.map_or(NO_INSTRUMENT, |id| id as u64),
            Ordering::Relaxed,
        );
        self.slowest_instrument_ns.store(details.slowest_instrument_ns, Ordering::Relaxed);
        self.sequence.store(sequence, Ordering::Release);
    }

    fn snapshot(&self) -> Option<(u64, DeadlineMissSnapshot)> {
        let sequence = self.sequence.load(Ordering::Acquire);
        if sequence == 0 {
            return None;
        }

        let callback_index = self.callback_index.load(Ordering::Relaxed);
        let frames = self.frames.load(Ordering::Relaxed);
        let budget_ns = self.budget_ns.load(Ordering::Relaxed);
        let total_ns = self.total_ns.load(Ordering::Relaxed);
        let midi_instrument_id = self.midi_instrument_id.load(Ordering::Relaxed);
        let encoded_midi_event = self.midi_event.load(Ordering::Relaxed);
        let slowest_instrument_id = self.slowest_instrument_id.load(Ordering::Relaxed);
        let snapshot = DeadlineMissSnapshot {
            callback_index,
            frames,
            budget_time_us: nanos_to_micros(budget_ns),
            total_time_us: nanos_to_micros(total_ns),
            load_percent: percent(total_ns, budget_ns),
            lock_wait_us: nanos_to_micros(self.lock_wait_ns.load(Ordering::Relaxed)),
            commands_us: nanos_to_micros(self.commands_ns.load(Ordering::Relaxed)),
            midi_us: nanos_to_micros(self.midi_ns.load(Ordering::Relaxed)),
            instruments_us: nanos_to_micros(self.instruments_ns.load(Ordering::Relaxed)),
            global_processing_us: nanos_to_micros(
                self.global_processing_ns.load(Ordering::Relaxed),
            ),
            output_conversion_us: nanos_to_micros(
                self.output_conversion_ns.load(Ordering::Relaxed),
            ),
            midi_event_count: self.midi_event_count.load(Ordering::Relaxed),
            last_midi_event: (midi_instrument_id != NO_INSTRUMENT).then(|| {
                DeadlineMissMidiEventSnapshot {
                    instrument_id: midi_instrument_id as usize,
                    event: decode_midi_event(encoded_midi_event),
                }
            }),
            slowest_instrument_id: (slowest_instrument_id != NO_INSTRUMENT)
                .then_some(slowest_instrument_id as usize),
            slowest_instrument_time_us: nanos_to_micros(
                self.slowest_instrument_ns.load(Ordering::Relaxed),
            ),
        };

        (self.sequence.load(Ordering::Acquire) == sequence).then_some((sequence, snapshot))
    }
}

#[derive(Debug, Clone)]
pub struct AudioPerformanceSample {
    callback_index: u64,
    render_time_ns: u64,
    budget_ns: u64,
    peak_load_scaled: u64,
    max_render_time_ns: u64,
    callback_count: u64,
    deadline_miss_count: u64,
}

impl PerformanceHistorySlot {
    fn write(&self, sequence: u64, sample: AudioPerformanceSample) {
        self.sequence.store(0, Ordering::Release);
        self.callback_index.store(sample.callback_index, Ordering::Relaxed);
        self.average_load_scaled.store(
            sample.render_time_ns.saturating_mul(LOAD_SCALE) / sample.budget_ns.max(1),
            Ordering::Relaxed,
        );
        self.peak_load_scaled.store(sample.peak_load_scaled, Ordering::Relaxed);
        self.average_render_time_ns.store(
            sample.render_time_ns.checked_div(sample.callback_count).unwrap_or(0),
            Ordering::Relaxed,
        );
        self.max_render_time_ns.store(sample.max_render_time_ns, Ordering::Relaxed);
        self.average_budget_ns.store(
            sample.budget_ns.checked_div(sample.callback_count).unwrap_or(0),
            Ordering::Relaxed,
        );
        self.deadline_miss_count.store(sample.deadline_miss_count, Ordering::Relaxed);
        self.sequence.store(sequence, Ordering::Release);
    }

    fn snapshot(&self) -> Option<(u64, AudioPerformanceHistorySample)> {
        let sequence = self.sequence.load(Ordering::Acquire);
        if sequence == 0 {
            return None;
        }

        let snapshot = AudioPerformanceHistorySample {
            callback_index: self.callback_index.load(Ordering::Relaxed),
            average_load_percent: scaled_percent(self.average_load_scaled.load(Ordering::Relaxed)),
            peak_load_percent: scaled_percent(self.peak_load_scaled.load(Ordering::Relaxed)),
            average_render_time_us: nanos_to_micros(
                self.average_render_time_ns.load(Ordering::Relaxed),
            ),
            max_render_time_us: nanos_to_micros(self.max_render_time_ns.load(Ordering::Relaxed)),
            average_budget_time_us: nanos_to_micros(self.average_budget_ns.load(Ordering::Relaxed)),
            deadline_miss_count: self.deadline_miss_count.load(Ordering::Relaxed),
        };

        (self.sequence.load(Ordering::Acquire) == sequence).then_some((sequence, snapshot))
    }
}

pub struct AudioMetrics {
    active: AtomicBool,
    callback_count: AtomicU64,
    rendered_frames: AtomicU64,
    render_time_ns: AtomicU64,
    callback_budget_ns: AtomicU64,
    last_render_time_ns: AtomicU64,
    max_render_time_ns: AtomicU64,
    last_callback_frames: AtomicU64,
    max_callback_frames: AtomicU64,
    last_load_scaled: AtomicU64,
    peak_load_scaled: AtomicU64,
    deadline_miss_count: AtomicU64,
    stream_error_count: AtomicU64,
    oversized_callback_count: AtomicU64,
    lock_wait: TimingMetric,
    commands: TimingMetric,
    midi: TimingMetric,
    instruments: TimingMetric,
    global_processing: TimingMetric,
    output_conversion: TimingMetric,
    instrument_metrics: Box<[TimingMetric]>,
    deadline_miss_write_index: AtomicU64,
    deadline_miss_history: Box<[DeadlineMissSlot]>,
    performance_history_write_index: AtomicU64,
    performance_history: Box<[PerformanceHistorySlot]>,
    performance_interval_frames: AtomicU64,
    performance_interval_render_time_ns: AtomicU64,
    performance_interval_budget_ns: AtomicU64,
    performance_interval_peak_load_scaled: AtomicU64,
    performance_interval_max_render_time_ns: AtomicU64,
    performance_interval_callback_count: AtomicU64,
    performance_interval_deadline_miss_count: AtomicU64,
    realtime_scheduling_status: AtomicU64,
    transport_tick_bits: AtomicU64,
}

impl Default for AudioMetrics {
    fn default() -> Self {
        Self {
            active: AtomicBool::default(),
            callback_count: AtomicU64::default(),
            rendered_frames: AtomicU64::default(),
            render_time_ns: AtomicU64::default(),
            callback_budget_ns: AtomicU64::default(),
            last_render_time_ns: AtomicU64::default(),
            max_render_time_ns: AtomicU64::default(),
            last_callback_frames: AtomicU64::default(),
            max_callback_frames: AtomicU64::default(),
            last_load_scaled: AtomicU64::default(),
            peak_load_scaled: AtomicU64::default(),
            deadline_miss_count: AtomicU64::default(),
            stream_error_count: AtomicU64::default(),
            oversized_callback_count: AtomicU64::default(),
            lock_wait: TimingMetric::default(),
            commands: TimingMetric::default(),
            midi: TimingMetric::default(),
            instruments: TimingMetric::default(),
            global_processing: TimingMetric::default(),
            output_conversion: TimingMetric::default(),
            instrument_metrics: (0..MAX_MONITORED_INSTRUMENTS)
                .map(|_| TimingMetric::default())
                .collect(),
            deadline_miss_write_index: AtomicU64::default(),
            deadline_miss_history: (0..DEADLINE_MISS_HISTORY_SIZE)
                .map(|_| DeadlineMissSlot::default())
                .collect(),
            performance_history_write_index: AtomicU64::default(),
            performance_history: (0..PERFORMANCE_HISTORY_SIZE)
                .map(|_| PerformanceHistorySlot::default())
                .collect(),
            performance_interval_frames: AtomicU64::default(),
            performance_interval_render_time_ns: AtomicU64::default(),
            performance_interval_budget_ns: AtomicU64::default(),
            performance_interval_peak_load_scaled: AtomicU64::default(),
            performance_interval_max_render_time_ns: AtomicU64::default(),
            performance_interval_callback_count: AtomicU64::default(),
            performance_interval_deadline_miss_count: AtomicU64::default(),
            realtime_scheduling_status: AtomicU64::default(),
            transport_tick_bits: AtomicU64::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct TimingMetricSnapshot {
    pub average_time_us: f64,
    pub last_time_us: f64,
    pub max_time_us: f64,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct AudioPhaseMetricsSnapshot {
    pub lock_wait: TimingMetricSnapshot,
    pub commands: TimingMetricSnapshot,
    pub midi: TimingMetricSnapshot,
    pub instruments: TimingMetricSnapshot,
    pub global_processing: TimingMetricSnapshot,
    pub output_conversion: TimingMetricSnapshot,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct InstrumentMetricsSnapshot {
    pub instrument_id: usize,
    pub render_count: u64,
    pub average_time_us: f64,
    pub last_time_us: f64,
    pub max_time_us: f64,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct DeadlineMissMidiEventSnapshot {
    pub instrument_id: usize,
    pub event: MidiEvent,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct DeadlineMissSnapshot {
    pub callback_index: u64,
    pub frames: u64,
    pub budget_time_us: f64,
    pub total_time_us: f64,
    pub load_percent: f64,
    pub lock_wait_us: f64,
    pub commands_us: f64,
    pub midi_us: f64,
    pub instruments_us: f64,
    pub global_processing_us: f64,
    pub output_conversion_us: f64,
    pub midi_event_count: u64,

    #[schema(required)]
    pub last_midi_event: Option<DeadlineMissMidiEventSnapshot>,

    #[schema(required)]
    pub slowest_instrument_id: Option<usize>,

    pub slowest_instrument_time_us: f64,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct AudioPerformanceHistorySample {
    pub callback_index: u64,
    pub average_load_percent: f64,
    pub peak_load_percent: f64,
    pub average_render_time_us: f64,
    pub max_render_time_us: f64,
    pub average_budget_time_us: f64,
    pub deadline_miss_count: u64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AudioMetricsSnapshot {
    pub active: bool,
    pub callback_count: u64,
    pub rendered_frames: u64,
    pub average_load_percent: f64,
    pub last_load_percent: f64,
    pub peak_load_percent: f64,
    pub last_render_time_us: f64,
    pub max_render_time_us: f64,
    pub deadline_miss_count: u64,
    pub stream_error_count: u64,
    pub oversized_callback_count: u64,
    pub last_callback_frames: u64,
    pub max_callback_frames: u64,
    pub phases: AudioPhaseMetricsSnapshot,
    pub instruments: Vec<InstrumentMetricsSnapshot>,
    pub deadline_misses: Vec<DeadlineMissSnapshot>,
    pub performance_history: Vec<AudioPerformanceHistorySample>,
    pub realtime_scheduling: RealtimeSchedulingStatus,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub enum RealtimeSchedulingStatus {
    NotAttempted,
    Enabled,
    Failed,
    Unsupported,
}

impl AudioMetrics {
    pub fn reset(&self) {
        self.callback_count.store(0, Ordering::Relaxed);
        self.rendered_frames.store(0, Ordering::Relaxed);
        self.render_time_ns.store(0, Ordering::Relaxed);
        self.callback_budget_ns.store(0, Ordering::Relaxed);
        self.last_render_time_ns.store(0, Ordering::Relaxed);
        self.max_render_time_ns.store(0, Ordering::Relaxed);
        self.last_callback_frames.store(0, Ordering::Relaxed);
        self.max_callback_frames.store(0, Ordering::Relaxed);
        self.last_load_scaled.store(0, Ordering::Relaxed);
        self.peak_load_scaled.store(0, Ordering::Relaxed);
        self.deadline_miss_count.store(0, Ordering::Relaxed);
        self.stream_error_count.store(0, Ordering::Relaxed);
        self.oversized_callback_count.store(0, Ordering::Relaxed);
        self.lock_wait.reset();
        self.commands.reset();
        self.midi.reset();
        self.instruments.reset();
        self.global_processing.reset();
        self.output_conversion.reset();
        for instrument in &self.instrument_metrics {
            instrument.reset();
        }
        self.deadline_miss_write_index.store(0, Ordering::Relaxed);
        for miss in &self.deadline_miss_history {
            miss.sequence.store(0, Ordering::Relaxed);
        }
        self.performance_history_write_index.store(0, Ordering::Relaxed);
        for sample in &self.performance_history {
            sample.sequence.store(0, Ordering::Relaxed);
        }
        self.performance_interval_frames.store(0, Ordering::Relaxed);
        self.performance_interval_render_time_ns.store(0, Ordering::Relaxed);
        self.performance_interval_budget_ns.store(0, Ordering::Relaxed);
        self.performance_interval_peak_load_scaled.store(0, Ordering::Relaxed);
        self.performance_interval_max_render_time_ns.store(0, Ordering::Relaxed);
        self.performance_interval_callback_count.store(0, Ordering::Relaxed);
        self.performance_interval_deadline_miss_count.store(0, Ordering::Relaxed);
        self.realtime_scheduling_status.store(0, Ordering::Relaxed);
        self.transport_tick_bits.store(0, Ordering::Relaxed);
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    pub fn record_callback(
        &self,
        frames: usize,
        sample_rate: u32,
        render_time_ns: u64,
        details: CallbackDetails,
    ) {
        let frames = frames as u64;
        let budget_ns = frames.saturating_mul(1_000_000_000) / u64::from(sample_rate.max(1));
        let load_scaled = render_time_ns.saturating_mul(LOAD_SCALE) / budget_ns.max(1);

        let callback_index = self.callback_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.rendered_frames.fetch_add(frames, Ordering::Relaxed);
        self.render_time_ns.fetch_add(render_time_ns, Ordering::Relaxed);
        self.callback_budget_ns.fetch_add(budget_ns, Ordering::Relaxed);
        self.last_render_time_ns.store(render_time_ns, Ordering::Relaxed);
        self.max_render_time_ns.fetch_max(render_time_ns, Ordering::Relaxed);
        self.last_callback_frames.store(frames, Ordering::Relaxed);
        self.max_callback_frames.fetch_max(frames, Ordering::Relaxed);
        self.last_load_scaled.store(load_scaled, Ordering::Relaxed);
        self.peak_load_scaled.fetch_max(load_scaled, Ordering::Relaxed);

        let missed_deadline = render_time_ns > budget_ns;
        if missed_deadline {
            self.deadline_miss_count.fetch_add(1, Ordering::Relaxed);
            let write_index = self.deadline_miss_write_index.fetch_add(1, Ordering::Relaxed);
            let slot = &self.deadline_miss_history
                [write_index as usize % self.deadline_miss_history.len()];
            slot.write(write_index + 1, callback_index, frames, budget_ns, render_time_ns, details);
        }

        self.performance_interval_render_time_ns.fetch_add(render_time_ns, Ordering::Relaxed);
        self.performance_interval_budget_ns.fetch_add(budget_ns, Ordering::Relaxed);
        self.performance_interval_peak_load_scaled.fetch_max(load_scaled, Ordering::Relaxed);
        self.performance_interval_max_render_time_ns.fetch_max(render_time_ns, Ordering::Relaxed);
        self.performance_interval_callback_count.fetch_add(1, Ordering::Relaxed);
        if missed_deadline {
            self.performance_interval_deadline_miss_count.fetch_add(1, Ordering::Relaxed);
        }

        let interval_frames =
            self.performance_interval_frames.fetch_add(frames, Ordering::Relaxed) + frames;
        if interval_frames >= u64::from(sample_rate.max(1)) {
            let render_time_ns =
                self.performance_interval_render_time_ns.swap(0, Ordering::Relaxed);
            let budget_ns = self.performance_interval_budget_ns.swap(0, Ordering::Relaxed);
            let peak_load_scaled =
                self.performance_interval_peak_load_scaled.swap(0, Ordering::Relaxed);
            let max_render_time_ns =
                self.performance_interval_max_render_time_ns.swap(0, Ordering::Relaxed);
            let callback_count =
                self.performance_interval_callback_count.swap(0, Ordering::Relaxed);
            let deadline_miss_count =
                self.performance_interval_deadline_miss_count.swap(0, Ordering::Relaxed);
            self.performance_interval_frames.store(0, Ordering::Relaxed);

            let write_index = self.performance_history_write_index.fetch_add(1, Ordering::Relaxed);
            let slot =
                &self.performance_history[write_index as usize % self.performance_history.len()];

            slot.write(
                write_index + 1,
                AudioPerformanceSample {
                    callback_index,
                    render_time_ns,
                    budget_ns,
                    peak_load_scaled,
                    max_render_time_ns,
                    callback_count,
                    deadline_miss_count,
                },
            );
        }
    }

    pub fn record_lock_wait(&self, elapsed_ns: u64) {
        self.lock_wait.record(elapsed_ns);
    }
    pub fn record_commands(&self, elapsed_ns: u64) {
        self.commands.record(elapsed_ns);
    }
    pub fn record_midi(&self, elapsed_ns: u64) {
        self.midi.record(elapsed_ns);
    }
    pub fn record_instruments(&self, elapsed_ns: u64) {
        self.instruments.record(elapsed_ns);
    }

    pub fn record_instrument(&self, instrument_id: usize, elapsed_ns: u64) {
        if let Some(metric) = self.instrument_metrics.get(instrument_id) {
            metric.record(elapsed_ns);
        }
    }

    pub fn record_global_processing(&self, elapsed_ns: u64) {
        self.global_processing.record(elapsed_ns);
    }

    pub fn record_output_conversion(&self, elapsed_ns: u64) {
        self.output_conversion.record(elapsed_ns);
    }

    pub fn record_stream_error(&self) {
        self.stream_error_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_oversized_callback(&self) {
        self.oversized_callback_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_realtime_scheduling(&self, result: Option<bool>) {
        let status = match result {
            Some(true) => 1,
            Some(false) => 2,
            None => 3,
        };
        self.realtime_scheduling_status.store(status, Ordering::Relaxed);
    }

    pub fn record_transport_tick(&self, tick: f64) {
        self.transport_tick_bits.store(tick.to_bits(), Ordering::Relaxed);
    }

    pub fn transport_tick(&self) -> f64 {
        f64::from_bits(self.transport_tick_bits.load(Ordering::Relaxed))
    }

    pub fn snapshot(&self) -> AudioMetricsSnapshot {
        let total_time = self.render_time_ns.load(Ordering::Relaxed);
        let total_budget = self.callback_budget_ns.load(Ordering::Relaxed);
        let instruments = self
            .instrument_metrics
            .iter()
            .enumerate()
            .filter_map(|(id, metric)| {
                let render_count = metric.count.load(Ordering::Relaxed);
                if render_count == 0 {
                    return None;
                }
                let timing = metric.snapshot();
                Some(InstrumentMetricsSnapshot {
                    instrument_id: id,
                    render_count,
                    average_time_us: timing.average_time_us,
                    last_time_us: timing.last_time_us,
                    max_time_us: timing.max_time_us,
                })
            })
            .collect();
        let mut deadline_misses: Vec<_> =
            self.deadline_miss_history.iter().filter_map(DeadlineMissSlot::snapshot).collect();
        deadline_misses.sort_unstable_by_key(|(sequence, _)| *sequence);
        let deadline_misses = deadline_misses.into_iter().map(|(_, miss)| miss).collect();
        let mut performance_history: Vec<_> =
            self.performance_history.iter().filter_map(PerformanceHistorySlot::snapshot).collect();
        performance_history.sort_unstable_by_key(|(sequence, _)| *sequence);
        let performance_history =
            performance_history.into_iter().map(|(_, sample)| sample).collect();

        AudioMetricsSnapshot {
            active: self.active.load(Ordering::Relaxed),
            callback_count: self.callback_count.load(Ordering::Relaxed),
            rendered_frames: self.rendered_frames.load(Ordering::Relaxed),
            average_load_percent: percent(total_time, total_budget),
            last_load_percent: scaled_percent(self.last_load_scaled.load(Ordering::Relaxed)),
            peak_load_percent: scaled_percent(self.peak_load_scaled.load(Ordering::Relaxed)),
            last_render_time_us: nanos_to_micros(self.last_render_time_ns.load(Ordering::Relaxed)),
            max_render_time_us: nanos_to_micros(self.max_render_time_ns.load(Ordering::Relaxed)),
            deadline_miss_count: self.deadline_miss_count.load(Ordering::Relaxed),
            stream_error_count: self.stream_error_count.load(Ordering::Relaxed),
            oversized_callback_count: self.oversized_callback_count.load(Ordering::Relaxed),
            last_callback_frames: self.last_callback_frames.load(Ordering::Relaxed),
            max_callback_frames: self.max_callback_frames.load(Ordering::Relaxed),
            phases: AudioPhaseMetricsSnapshot {
                lock_wait: self.lock_wait.snapshot(),
                commands: self.commands.snapshot(),
                midi: self.midi.snapshot(),
                instruments: self.instruments.snapshot(),
                global_processing: self.global_processing.snapshot(),
                output_conversion: self.output_conversion.snapshot(),
            },
            instruments,
            deadline_misses,
            performance_history,
            realtime_scheduling: match self.realtime_scheduling_status.load(Ordering::Relaxed) {
                1 => RealtimeSchedulingStatus::Enabled,
                2 => RealtimeSchedulingStatus::Failed,
                3 => RealtimeSchedulingStatus::Unsupported,
                _ => RealtimeSchedulingStatus::NotAttempted,
            },
        }
    }
}

fn percent(value: u64, total: u64) -> f64 {
    if total == 0 { 0.0 } else { value as f64 * 100.0 / total as f64 }
}
fn scaled_percent(value: u64) -> f64 {
    value as f64 * 100.0 / LOAD_SCALE as f64
}
fn nanos_to_micros(value: u64) -> f64 {
    value as f64 / 1_000.0
}

fn encode_midi_event(event: MidiEvent) -> u64 {
    let (kind, data1, data2): (u8, u16, u16) = match event.kind {
        MidiEventKind::NoteOn { key, velocity } => (0, u16::from(key), u16::from(velocity)),
        MidiEventKind::NoteOff { key, velocity } => (1, u16::from(key), u16::from(velocity)),
        MidiEventKind::ControlChange { controller, value } => {
            (2, u16::from(controller), u16::from(value))
        }
        MidiEventKind::PitchBend { value } => (3, value as u16, 0),
        MidiEventKind::ProgramChange { program } => (4, u16::from(program), 0),
        MidiEventKind::ChannelAftertouch { value } => (5, u16::from(value), 0),
        MidiEventKind::PolyphonicAftertouch { key, value } => (6, u16::from(key), u16::from(value)),
    };

    u64::from(kind)
        | (u64::from(event.channel) << 3)
        | ((event.slot as u64 & 0xffff) << 7)
        | (u64::from(data1) << 23)
        | (u64::from(data2) << 39)
}

fn decode_midi_event(encoded: u64) -> MidiEvent {
    let channel = ((encoded >> 3) & 0x0f) as u8;
    let slot = ((encoded >> 7) & 0xffff) as usize;
    let data1 = ((encoded >> 23) & 0xffff) as u16;
    let data2 = ((encoded >> 39) & 0xffff) as u16;
    let kind = match encoded & 0x07 {
        0 => MidiEventKind::NoteOn { key: data1 as u8, velocity: data2 as u8 },
        1 => MidiEventKind::NoteOff { key: data1 as u8, velocity: data2 as u8 },
        2 => MidiEventKind::ControlChange { controller: data1 as u8, value: data2 as u8 },
        3 => MidiEventKind::PitchBend { value: data1 as i16 },
        4 => MidiEventKind::ProgramChange { program: data1 as u8 },
        5 => MidiEventKind::ChannelAftertouch { value: data1 as u8 },
        6 => MidiEventKind::PolyphonicAftertouch { key: data1 as u8, value: data2 as u8 },
        _ => unreachable!(),
    };
    MidiEvent { slot, channel, kind }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_load_phases_and_instruments() {
        let metrics = AudioMetrics::default();
        metrics.record_callback(480, 48_000, 5_000_000, CallbackDetails::default());
        metrics.record_callback(480, 48_000, 12_000_000, CallbackDetails::default());
        metrics.record_commands(1_000);
        metrics.record_instrument(3, 2_000);
        metrics.record_transport_tick(1_234.5);
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.callback_count, 2);
        assert_eq!(snapshot.average_load_percent, 85.0);
        assert_eq!(snapshot.last_load_percent, 120.0);
        assert_eq!(snapshot.deadline_miss_count, 1);
        assert_eq!(snapshot.deadline_misses.len(), 1);
        assert_eq!(snapshot.deadline_misses[0].callback_index, 2);
        assert_eq!(snapshot.phases.commands.max_time_us, 1.0);
        assert_eq!(snapshot.instruments[0].instrument_id, 3);
        assert_eq!(snapshot.instruments[0].max_time_us, 2.0);
        assert_eq!(metrics.transport_tick(), 1_234.5);
    }

    #[test]
    fn midi_event_trace_round_trips() {
        let event =
            MidiEvent { slot: 7, channel: 12, kind: MidiEventKind::PitchBend { value: -4_096 } };
        let decoded = decode_midi_event(encode_midi_event(event));
        assert_eq!(decoded.slot, event.slot);
        assert_eq!(decoded.channel, event.channel);
        assert!(matches!(decoded.kind, MidiEventKind::PitchBend { value: -4_096 }));
    }

    #[test]
    fn records_one_second_performance_history_samples() {
        let metrics = AudioMetrics::default();
        for _ in 0..100 {
            metrics.record_callback(480, 48_000, 5_000_000, CallbackDetails::default());
        }

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.performance_history.len(), 1);
        let sample = snapshot.performance_history[0];
        assert_eq!(sample.callback_index, 100);
        assert_eq!(sample.average_load_percent, 50.0);
        assert_eq!(sample.peak_load_percent, 50.0);
        assert_eq!(sample.average_render_time_us, 5_000.0);
        assert_eq!(sample.max_render_time_us, 5_000.0);
        assert_eq!(sample.average_budget_time_us, 10_000.0);
        assert_eq!(sample.deadline_miss_count, 0);
    }
}
