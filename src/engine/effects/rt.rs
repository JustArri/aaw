use crate::{
    DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE,
    audio::block::AudioBlock,
    engine::effects::{
        EFFECTS_TUNING, Effect, EffectKind, Effects, EqualizerBand as EffectEqualizerBand, Error,
    },
};
use arrayvec::ArrayVec;
use std::f32::consts::TAU;

pub struct RtEffects {
    effects: ArrayVec<RtEffect, { EFFECTS_TUNING.max_effects }>,
    channel_count: usize,
    sample_rate: u32,
}

impl Default for RtEffects {
    fn default() -> Self {
        Self {
            effects: Default::default(),
            channel_count: DEFAULT_CHANNEL_COUNT,
            sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }
}

impl RtEffects {
    pub fn new(
        parameters: &Effects,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        if parameters.effects.len() > EFFECTS_TUNING.max_effects {
            return Err(Error::TooManyEffects {
                actual: parameters.effects.len(),
                maximum: EFFECTS_TUNING.max_effects,
            });
        }

        let mut effects = ArrayVec::new();
        for effect in &parameters.effects {
            effects.push(RtEffect::new(effect, channel_count, sample_rate)?);
        }

        Ok(Self { effects, channel_count, sample_rate })
    }

    pub(super) fn new_replacing_failed(
        parameters: &mut Effects,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        if parameters.effects.len() > EFFECTS_TUNING.max_effects {
            return Err(Error::TooManyEffects {
                actual: parameters.effects.len(),
                maximum: EFFECTS_TUNING.max_effects,
            });
        }

        let mut effects = ArrayVec::new();
        for parameters in &mut parameters.effects {
            match RtEffect::new(parameters, channel_count, sample_rate) {
                Ok(effect) => effects.push(effect),
                Err(error) => {
                    parameters.kind = EffectKind::LoadFailed { description: error.to_string() };
                    effects.push(RtEffect {
                        enabled: parameters.enabled,
                        kind: RtEffectKind::LoadFailed,
                    });
                }
            }
        }

        Ok(Self { effects, channel_count, sample_rate })
    }

    pub fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        for effect in &mut self.effects {
            effect.reconfigure(channel_count, sample_rate)?;
        }
        self.channel_count = channel_count;
        self.sample_rate = sample_rate;
        Ok(())
    }

    pub fn apply(&mut self, block: &mut AudioBlock) {
        for effect in &mut self.effects {
            effect.apply(block);
        }
    }
}

pub struct RtEffect {
    pub enabled: bool,
    pub kind: RtEffectKind,
}

pub enum RtEffectKind {
    Gain(RtGain),
    LowPassFilter(RtLowPassFilter),
    HighPassFilter(RtHighPassFilter),
    Equalizer(RtEqualizer),
    Compressor(RtCompressor),
    Limiter(RtLimiter),
    Delay(RtDelay),
    Reverb(RtReverb),
    Chorus(RtChorus),
    Distortion(RtDistortion),
    LoadFailed,
}

impl RtEffect {
    fn new(parameters: &Effect, channel_count: usize, sample_rate: u32) -> Result<Self, Error> {
        let kind = match &parameters.kind {
            EffectKind::Gain { value } => RtEffectKind::Gain(RtGain::new(*value)?),
            EffectKind::LowPassFilter { frequency, resonance } => RtEffectKind::LowPassFilter(
                RtLowPassFilter::new(*frequency, *resonance, channel_count, sample_rate)?,
            ),
            EffectKind::HighPassFilter { frequency, resonance } => RtEffectKind::HighPassFilter(
                RtHighPassFilter::new(*frequency, *resonance, channel_count, sample_rate)?,
            ),
            EffectKind::Equalizer { bands } => {
                RtEffectKind::Equalizer(RtEqualizer::new(bands, channel_count, sample_rate)?)
            }
            EffectKind::Compressor {
                threshold_db,
                ratio,
                attack_ms,
                release_ms,
                makeup_gain_db,
            } => RtEffectKind::Compressor(RtCompressor::new(
                *threshold_db,
                *ratio,
                *attack_ms,
                *release_ms,
                *makeup_gain_db,
                sample_rate,
            )?),
            EffectKind::Limiter { threshold_db, release_ms } => {
                RtEffectKind::Limiter(RtLimiter::new(*threshold_db, *release_ms, sample_rate)?)
            }
            EffectKind::Delay { time_ms, feedback, mix } => RtEffectKind::Delay(RtDelay::new(
                *time_ms,
                *feedback,
                *mix,
                channel_count,
                sample_rate,
            )?),
            EffectKind::Reverb { room_size, damping, mix } => RtEffectKind::Reverb(RtReverb::new(
                *room_size,
                *damping,
                *mix,
                channel_count,
                sample_rate,
            )?),
            EffectKind::Chorus { rate_hz, depth_ms, delay_ms, mix } => RtEffectKind::Chorus(
                RtChorus::new(*rate_hz, *depth_ms, *delay_ms, *mix, channel_count, sample_rate)?,
            ),
            EffectKind::Distortion { drive_db, mix } => {
                RtEffectKind::Distortion(RtDistortion::new(*drive_db, *mix)?)
            }
            EffectKind::LoadFailed { .. } => RtEffectKind::LoadFailed,
        };
        Ok(Self { enabled: parameters.enabled, kind })
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        match &mut self.kind {
            RtEffectKind::Gain(_) => Ok(()),
            RtEffectKind::LowPassFilter(effect) => effect.reconfigure(channel_count, sample_rate),
            RtEffectKind::HighPassFilter(effect) => effect.reconfigure(channel_count, sample_rate),
            RtEffectKind::Equalizer(effect) => effect.reconfigure(channel_count, sample_rate),
            RtEffectKind::Compressor(effect) => effect.reconfigure(sample_rate),
            RtEffectKind::Limiter(effect) => effect.reconfigure(sample_rate),
            RtEffectKind::Delay(effect) => effect.reconfigure(channel_count, sample_rate),
            RtEffectKind::Reverb(effect) => effect.reconfigure(channel_count, sample_rate),
            RtEffectKind::Chorus(effect) => effect.reconfigure(channel_count, sample_rate),
            RtEffectKind::Distortion(_) => Ok(()),
            RtEffectKind::LoadFailed => Ok(()),
        }
    }

    pub fn apply(&mut self, block: &mut AudioBlock) {
        if !self.enabled {
            return;
        }
        match &mut self.kind {
            RtEffectKind::Gain(effect) => effect.apply(block),
            RtEffectKind::LowPassFilter(effect) => effect.apply(block),
            RtEffectKind::HighPassFilter(effect) => effect.apply(block),
            RtEffectKind::Equalizer(effect) => effect.apply(block),
            RtEffectKind::Compressor(effect) => effect.apply(block),
            RtEffectKind::Limiter(effect) => effect.apply(block),
            RtEffectKind::Delay(effect) => effect.apply(block),
            RtEffectKind::Reverb(effect) => effect.apply(block),
            RtEffectKind::Chorus(effect) => effect.apply(block),
            RtEffectKind::Distortion(effect) => effect.apply(block),
            RtEffectKind::LoadFailed => {}
        }
    }
}

pub struct RtGain {
    value: f32,
}

impl RtGain {
    fn new(value: f32) -> Result<Self, Error> {
        if !value.is_finite() {
            return Err(Error::InvalidGain);
        }
        Ok(Self { value })
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        let sample_count = block.sample_count_per_channel();
        let channel_count = block.channel_count();
        for channel in block.samples_mut().iter_mut().take(channel_count) {
            for sample in &mut channel[..sample_count] {
                *sample *= self.value;
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
struct FilterState {
    z1: f32,
    z2: f32,
}

#[derive(Clone, Copy)]
struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

pub struct RtLowPassFilter {
    frequency: f32,
    resonance: f32,
    coefficients: BiquadCoefficients,
    channel_states: Vec<FilterState>,
}

impl RtLowPassFilter {
    fn new(
        frequency: f32,
        resonance: f32,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        validate_low_pass_parameters(frequency, resonance)?;
        let coefficients = low_pass_coefficients(frequency, resonance, sample_rate);
        let channel_states = vec![FilterState::default(); channel_count];
        Ok(Self { frequency, resonance, coefficients, channel_states })
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        self.coefficients = low_pass_coefficients(self.frequency, self.resonance, sample_rate);
        self.channel_states.resize(channel_count, FilterState::default());
        self.channel_states.fill(FilterState::default());
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        apply_biquad(block, self.coefficients, &mut self.channel_states);
    }
}

pub struct RtHighPassFilter {
    frequency: f32,
    resonance: f32,
    coefficients: BiquadCoefficients,
    channel_states: Vec<FilterState>,
}

impl RtHighPassFilter {
    fn new(
        frequency: f32,
        resonance: f32,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        validate_high_pass_parameters(frequency, resonance)?;
        Ok(Self {
            frequency,
            resonance,
            coefficients: high_pass_coefficients(frequency, resonance, sample_rate),
            channel_states: vec![FilterState::default(); channel_count],
        })
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        self.coefficients = high_pass_coefficients(self.frequency, self.resonance, sample_rate);
        self.channel_states.resize(channel_count, FilterState::default());
        self.channel_states.fill(FilterState::default());
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        apply_biquad(block, self.coefficients, &mut self.channel_states);
    }
}

pub struct RtCompressor {
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_gain_db: f32,
    attack_coefficient: f32,
    release_coefficient: f32,
    envelope: f32,
}

impl RtCompressor {
    fn new(
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_gain_db: f32,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        validate_compressor_parameters(threshold_db, ratio, attack_ms, release_ms, makeup_gain_db)?;
        Ok(Self {
            threshold_db,
            ratio,
            attack_ms,
            release_ms,
            makeup_gain_db,
            attack_coefficient: time_coefficient(attack_ms, sample_rate),
            release_coefficient: time_coefficient(release_ms, sample_rate),
            envelope: 0.0,
        })
    }

    fn reconfigure(&mut self, sample_rate: u32) -> Result<(), Error> {
        self.attack_coefficient = time_coefficient(self.attack_ms, sample_rate);
        self.release_coefficient = time_coefficient(self.release_ms, sample_rate);
        self.envelope = 0.0;
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        let sample_count = block.sample_count_per_channel();
        let channel_count = block.channel_count();
        let channels = &mut block.samples_mut()[..channel_count];
        let threshold = db_to_linear(self.threshold_db);
        let makeup = db_to_linear(self.makeup_gain_db);
        let exponent = 1.0 - self.ratio.recip();
        for index in 0..sample_count {
            let peak = channels.iter().map(|channel| channel[index].abs()).fold(0.0, f32::max);
            let coefficient = if peak > self.envelope {
                self.attack_coefficient
            } else {
                self.release_coefficient
            };
            self.envelope = coefficient.mul_add(self.envelope, (1.0 - coefficient) * peak);
            let reduction = if self.envelope > threshold {
                (threshold / self.envelope).powf(exponent)
            } else {
                1.0
            };
            let gain = reduction * makeup;
            for channel in channels.iter_mut() {
                channel[index] *= gain;
            }
        }
    }
}

pub struct RtLimiter {
    threshold_db: f32,
    release_ms: f32,
    release_coefficient: f32,
    gain: f32,
}

impl RtLimiter {
    fn new(threshold_db: f32, release_ms: f32, sample_rate: u32) -> Result<Self, Error> {
        validate_limiter_parameters(threshold_db, release_ms)?;
        Ok(Self {
            threshold_db,
            release_ms,
            release_coefficient: time_coefficient(release_ms, sample_rate),
            gain: 1.0,
        })
    }

    fn reconfigure(&mut self, sample_rate: u32) -> Result<(), Error> {
        self.release_coefficient = time_coefficient(self.release_ms, sample_rate);
        self.gain = 1.0;
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        let sample_count = block.sample_count_per_channel();
        let channel_count = block.channel_count();
        let channels = &mut block.samples_mut()[..channel_count];
        let threshold = db_to_linear(self.threshold_db);
        for index in 0..sample_count {
            let peak = channels.iter().map(|channel| channel[index].abs()).fold(0.0, f32::max);
            let target = if peak > threshold { threshold / peak } else { 1.0 };
            if target < self.gain {
                self.gain = target;
            } else {
                self.gain = self
                    .release_coefficient
                    .mul_add(self.gain, (1.0 - self.release_coefficient) * target);
            }
            for channel in channels.iter_mut() {
                channel[index] *= self.gain;
            }
        }
    }
}

pub struct RtDelay {
    time_ms: f32,
    feedback: f32,
    mix: f32,
    buffers: Vec<Vec<f32>>,
    position: usize,
}

impl RtDelay {
    fn new(
        time_ms: f32,
        feedback: f32,
        mix: f32,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        validate_delay_parameters(time_ms, feedback, mix)?;
        let mut effect = Self { time_ms, feedback, mix, buffers: Vec::new(), position: 0 };
        effect.reconfigure(channel_count, sample_rate)?;
        Ok(effect)
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        let length = milliseconds_to_samples(self.time_ms, sample_rate).max(1);
        self.buffers = vec![vec![0.0; length]; channel_count];
        self.position = 0;
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        let sample_count = block.sample_count_per_channel();
        let channel_count = block.channel_count().min(self.buffers.len());
        if channel_count == 0 {
            return;
        }
        let channels = &mut block.samples_mut()[..channel_count];
        let length = self.buffers[0].len();
        for index in 0..sample_count {
            for (channel, buffer) in channels.iter_mut().zip(&mut self.buffers) {
                let input = channel[index];
                let delayed = buffer[self.position];
                buffer[self.position] = input + delayed * self.feedback;
                channel[index] = input * (1.0 - self.mix) + delayed * self.mix;
            }
            self.position = (self.position + 1) % length;
        }
    }
}

pub struct RtDistortion {
    drive: f32,
    normalization: f32,
    mix: f32,
}

impl RtDistortion {
    fn new(drive_db: f32, mix: f32) -> Result<Self, Error> {
        validate_distortion_parameters(drive_db, mix)?;
        let drive = db_to_linear(drive_db);
        Ok(Self { drive, normalization: drive.tanh().recip(), mix })
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        let sample_count = block.sample_count_per_channel();
        let channel_count = block.channel_count();
        for channel in block.samples_mut().iter_mut().take(channel_count) {
            for sample in &mut channel[..sample_count] {
                let dry = *sample;
                let wet = (dry * self.drive).tanh() * self.normalization;
                *sample = dry * (1.0 - self.mix) + wet * self.mix;
            }
        }
    }
}

pub struct RtChorus {
    rate_hz: f32,
    depth_ms: f32,
    delay_ms: f32,
    mix: f32,
    sample_rate: u32,
    buffers: Vec<Vec<f32>>,
    position: usize,
    phase: f32,
}

impl RtChorus {
    fn new(
        rate_hz: f32,
        depth_ms: f32,
        delay_ms: f32,
        mix: f32,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        validate_chorus_parameters(rate_hz, depth_ms, delay_ms, mix)?;
        let mut effect = Self {
            rate_hz,
            depth_ms,
            delay_ms,
            mix,
            sample_rate,
            buffers: Vec::new(),
            position: 0,
            phase: 0.0,
        };
        effect.reconfigure(channel_count, sample_rate)?;
        Ok(effect)
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        self.sample_rate = sample_rate;
        let maximum_delay = self.delay_ms + self.depth_ms;
        let length = milliseconds_to_samples(maximum_delay, sample_rate) + 2;
        self.buffers = vec![vec![0.0; length]; channel_count];
        self.position = 0;
        self.phase = 0.0;
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        let sample_count = block.sample_count_per_channel();
        let channel_count = block.channel_count().min(self.buffers.len());
        if channel_count == 0 {
            return;
        }
        let channels = &mut block.samples_mut()[..channel_count];
        let length = self.buffers[0].len();
        let phase_step = self.rate_hz / self.sample_rate as f32;
        for index in 0..sample_count {
            let modulation = (TAU * self.phase).sin();
            let delay_samples =
                (self.delay_ms + self.depth_ms * modulation) * self.sample_rate as f32 / 1000.0;
            let read_position = (self.position as f32 - delay_samples).rem_euclid(length as f32);
            let first = read_position.floor() as usize;
            let second = (first + 1) % length;
            let fraction = read_position - first as f32;
            for (channel, buffer) in channels.iter_mut().zip(&mut self.buffers) {
                let input = channel[index];
                let delayed = buffer[first] + (buffer[second] - buffer[first]) * fraction;
                buffer[self.position] = input;
                channel[index] = input * (1.0 - self.mix) + delayed * self.mix;
            }
            self.position = (self.position + 1) % length;
            self.phase = (self.phase + phase_step).fract();
        }
    }
}

struct CombFilter {
    buffer: Vec<f32>,
    position: usize,
    filtered: f32,
}

impl CombFilter {
    fn new(length: usize) -> Self {
        Self { buffer: vec![0.0; length.max(1)], position: 0, filtered: 0.0 }
    }

    fn process(&mut self, input: f32, feedback: f32, damping: f32) -> f32 {
        let output = self.buffer[self.position];
        self.filtered = output * (1.0 - damping) + self.filtered * damping;
        self.buffer[self.position] = input + self.filtered * feedback;
        self.position = (self.position + 1) % self.buffer.len();
        output
    }
}

struct AllPassFilter {
    buffer: Vec<f32>,
    position: usize,
}

impl AllPassFilter {
    fn new(length: usize) -> Self {
        Self { buffer: vec![0.0; length.max(1)], position: 0 }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.position];
        let output = buffered - input;
        self.buffer[self.position] = input + buffered * EFFECTS_TUNING.reverb_all_pass_feedback;
        self.position = (self.position + 1) % self.buffer.len();
        output
    }
}

struct ReverbChannel {
    combs: Vec<CombFilter>,
    all_passes: Vec<AllPassFilter>,
}

impl ReverbChannel {
    fn new(sample_rate: u32, channel_index: usize) -> Self {
        let scale = sample_rate as f32 / EFFECTS_TUNING.reverb_reference_sample_rate as f32;
        let spread = channel_index * 23;
        Self {
            combs: EFFECTS_TUNING
                .reverb_comb_delays
                .iter()
                .map(|delay| CombFilter::new(((*delay + spread) as f32 * scale).round() as usize))
                .collect(),
            all_passes: EFFECTS_TUNING
                .reverb_all_pass_delays
                .iter()
                .map(
                    |delay| AllPassFilter::new(((*delay + spread) as f32 * scale).round() as usize),
                )
                .collect(),
        }
    }

    fn process(&mut self, input: f32, feedback: f32, damping: f32) -> f32 {
        let mut output =
            self.combs.iter_mut().map(|comb| comb.process(input, feedback, damping)).sum::<f32>()
                / self.combs.len() as f32;
        for all_pass in &mut self.all_passes {
            output = all_pass.process(output);
        }
        output
    }
}

pub struct RtReverb {
    room_size: f32,
    damping: f32,
    mix: f32,
    channels: Vec<ReverbChannel>,
}

impl RtReverb {
    fn new(
        room_size: f32,
        damping: f32,
        mix: f32,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        validate_reverb_parameters(room_size, damping, mix)?;
        let mut effect = Self { room_size, damping, mix, channels: Vec::new() };
        effect.reconfigure(channel_count, sample_rate)?;
        Ok(effect)
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        self.channels =
            (0..channel_count).map(|index| ReverbChannel::new(sample_rate, index)).collect();
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        let sample_count = block.sample_count_per_channel();
        let channel_count = block.channel_count().min(self.channels.len());
        let feedback = 0.7 + self.room_size * 0.28;
        for (samples, reverb) in
            block.samples_mut().iter_mut().zip(&mut self.channels).take(channel_count)
        {
            for sample in &mut samples[..sample_count] {
                let dry = *sample;
                let wet = reverb.process(dry, feedback, self.damping);
                *sample = dry * (1.0 - self.mix) + wet * self.mix;
            }
        }
    }
}

pub struct RtEqualizer {
    bands: Vec<RtEqualizerBand>,
}

impl RtEqualizer {
    fn new(
        parameters: &[EffectEqualizerBand],
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        if parameters.len() > EFFECTS_TUNING.max_equalizer_bands {
            return Err(Error::TooManyEqualizerBands {
                actual: parameters.len(),
                maximum: EFFECTS_TUNING.max_equalizer_bands,
            });
        }

        let mut bands = Vec::with_capacity(parameters.len());
        for band in parameters {
            bands.push(RtEqualizerBand::new(band, channel_count, sample_rate)?);
        }
        Ok(Self { bands })
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        for band in &mut self.bands {
            band.reconfigure(channel_count, sample_rate)?;
        }
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        for band in &mut self.bands {
            band.apply(block);
        }
    }
}

#[derive(Clone, Copy)]
enum EqualizerBandKind {
    LowShelf,
    Peaking,
    HighShelf,
}

struct RtEqualizerBand {
    kind: EqualizerBandKind,
    frequency: f32,
    gain_db: f32,
    resonance: f32,
    coefficients: BiquadCoefficients,
    channel_states: Vec<FilterState>,
}

impl RtEqualizerBand {
    fn new(
        parameters: &EffectEqualizerBand,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        let (kind, frequency, gain_db, resonance) = match *parameters {
            EffectEqualizerBand::LowShelf { frequency, gain_db, resonance } => {
                (EqualizerBandKind::LowShelf, frequency, gain_db, resonance)
            }
            EffectEqualizerBand::Peaking { frequency, gain_db, resonance } => {
                (EqualizerBandKind::Peaking, frequency, gain_db, resonance)
            }
            EffectEqualizerBand::HighShelf { frequency, gain_db, resonance } => {
                (EqualizerBandKind::HighShelf, frequency, gain_db, resonance)
            }
        };
        validate_equalizer_parameters(frequency, gain_db, resonance)?;
        let coefficients =
            equalizer_coefficients(kind, frequency, gain_db, resonance, sample_rate)?;
        let channel_states = vec![FilterState::default(); channel_count];
        Ok(Self { kind, frequency, gain_db, resonance, coefficients, channel_states })
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        self.coefficients = equalizer_coefficients(
            self.kind,
            self.frequency,
            self.gain_db,
            self.resonance,
            sample_rate,
        )?;
        self.channel_states.resize(channel_count, FilterState::default());
        self.channel_states.fill(FilterState::default());
        Ok(())
    }

    fn apply(&mut self, block: &mut AudioBlock) {
        apply_biquad(block, self.coefficients, &mut self.channel_states);
    }
}

fn apply_biquad(
    block: &mut AudioBlock,
    coefficients: BiquadCoefficients,
    channel_states: &mut [FilterState],
) {
    let sample_count = block.sample_count_per_channel();
    let channel_count = block.channel_count().min(channel_states.len());

    for (samples, state) in block.samples_mut().iter_mut().zip(channel_states).take(channel_count) {
        for sample in &mut samples[..sample_count] {
            let input = *sample;
            let output = coefficients.b0.mul_add(input, state.z1);
            state.z1 = coefficients.b1.mul_add(input, (-coefficients.a1).mul_add(output, state.z2));
            state.z2 = coefficients.b2.mul_add(input, -coefficients.a2 * output);
            *sample = output;
        }

        if state.z1.abs() < EFFECTS_TUNING.filter_state_cutoff {
            state.z1 = 0.0;
        }
        if state.z2.abs() < EFFECTS_TUNING.filter_state_cutoff {
            state.z2 = 0.0;
        }
    }
}

fn validate_low_pass_parameters(frequency: f32, resonance: f32) -> Result<(), Error> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return Err(Error::InvalidLowPassFrequency);
    }
    if !resonance.is_finite() || resonance <= 0.0 {
        return Err(Error::InvalidLowPassResonance);
    }
    Ok(())
}

fn validate_high_pass_parameters(frequency: f32, resonance: f32) -> Result<(), Error> {
    if !frequency.is_finite() || frequency <= 0.0 || !resonance.is_finite() || resonance <= 0.0 {
        return Err(Error::InvalidHighPass);
    }
    Ok(())
}

fn validate_compressor_parameters(
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_gain_db: f32,
) -> Result<(), Error> {
    if !threshold_db.is_finite()
        || !(-120.0..=24.0).contains(&threshold_db)
        || !ratio.is_finite()
        || !(1.0..=100.0).contains(&ratio)
        || !attack_ms.is_finite()
        || !(0.0..=60000.0).contains(&attack_ms)
        || attack_ms == 0.0
        || !release_ms.is_finite()
        || !(0.0..=60000.0).contains(&release_ms)
        || release_ms == 0.0
        || !makeup_gain_db.is_finite()
        || !(-60.0..=60.0).contains(&makeup_gain_db)
    {
        return Err(Error::InvalidCompressor);
    }
    Ok(())
}

fn validate_limiter_parameters(threshold_db: f32, release_ms: f32) -> Result<(), Error> {
    if !threshold_db.is_finite()
        || !(-120.0..=0.0).contains(&threshold_db)
        || !release_ms.is_finite()
        || !(0.0..=60000.0).contains(&release_ms)
        || release_ms == 0.0
    {
        return Err(Error::InvalidLimiter);
    }
    Ok(())
}

fn validate_delay_parameters(time_ms: f32, feedback: f32, mix: f32) -> Result<(), Error> {
    if !time_ms.is_finite()
        || time_ms <= 0.0
        || time_ms > EFFECTS_TUNING.max_delay_ms
        || !feedback.is_finite()
        || !(0.0..1.0).contains(&feedback)
        || !valid_mix(mix)
    {
        return Err(Error::InvalidDelay);
    }
    Ok(())
}

fn validate_reverb_parameters(room_size: f32, damping: f32, mix: f32) -> Result<(), Error> {
    if !unit_parameter(room_size) || !unit_parameter(damping) || !valid_mix(mix) {
        return Err(Error::InvalidReverb);
    }
    Ok(())
}

fn validate_chorus_parameters(
    rate_hz: f32,
    depth_ms: f32,
    delay_ms: f32,
    mix: f32,
) -> Result<(), Error> {
    if !rate_hz.is_finite()
        || rate_hz <= 0.0
        || rate_hz > 20.0
        || !depth_ms.is_finite()
        || depth_ms < 0.0
        || !delay_ms.is_finite()
        || delay_ms <= 0.0
        || depth_ms > delay_ms
        || delay_ms + depth_ms > EFFECTS_TUNING.max_chorus_delay_ms
        || !valid_mix(mix)
    {
        return Err(Error::InvalidChorus);
    }
    Ok(())
}

fn validate_distortion_parameters(drive_db: f32, mix: f32) -> Result<(), Error> {
    if !drive_db.is_finite() || !(0.0..=60.0).contains(&drive_db) || !valid_mix(mix) {
        return Err(Error::InvalidDistortion);
    }
    Ok(())
}

fn unit_parameter(value: f32) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn valid_mix(value: f32) -> bool {
    unit_parameter(value)
}

fn db_to_linear(value: f32) -> f32 {
    10.0f32.powf(value / 20.0)
}

fn time_coefficient(milliseconds: f32, sample_rate: u32) -> f32 {
    (-1.0 / (milliseconds * 0.001 * sample_rate as f32)).exp()
}

fn milliseconds_to_samples(milliseconds: f32, sample_rate: u32) -> usize {
    (milliseconds * 0.001 * sample_rate as f32).round() as usize
}

fn validate_equalizer_parameters(
    frequency: f32,
    gain_db: f32,
    resonance: f32,
) -> Result<(), Error> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return Err(Error::InvalidEqualizerFrequency);
    }
    if !gain_db.is_finite() {
        return Err(Error::InvalidEqualizerGain);
    }
    if !resonance.is_finite() || resonance <= 0.0 {
        return Err(Error::InvalidEqualizerResonance);
    }
    Ok(())
}

fn low_pass_coefficients(frequency: f32, resonance: f32, sample_rate: u32) -> BiquadCoefficients {
    let sample_rate = sample_rate as f32;
    let frequency = frequency.min(sample_rate * 0.499);
    let omega = TAU * frequency / sample_rate;
    let sin = omega.sin();
    let cos = omega.cos();
    let alpha = sin / (2.0 * resonance);
    let a0_recip = (1.0 + alpha).recip();
    let b0 = (1.0 - cos) * 0.5 * a0_recip;

    BiquadCoefficients {
        b0,
        b1: (1.0 - cos) * a0_recip,
        b2: b0,
        a1: -2.0 * cos * a0_recip,
        a2: (1.0 - alpha) * a0_recip,
    }
}

fn high_pass_coefficients(frequency: f32, resonance: f32, sample_rate: u32) -> BiquadCoefficients {
    let sample_rate = sample_rate as f32;
    let frequency = frequency.min(sample_rate * 0.499);
    let omega = TAU * frequency / sample_rate;
    let sin = omega.sin();
    let cos = omega.cos();
    let alpha = sin / (2.0 * resonance);
    let a0_recip = (1.0 + alpha).recip();
    let b0 = (1.0 + cos) * 0.5 * a0_recip;

    BiquadCoefficients {
        b0,
        b1: -(1.0 + cos) * a0_recip,
        b2: b0,
        a1: -2.0 * cos * a0_recip,
        a2: (1.0 - alpha) * a0_recip,
    }
}

fn equalizer_coefficients(
    kind: EqualizerBandKind,
    frequency: f32,
    gain_db: f32,
    resonance: f32,
    sample_rate: u32,
) -> Result<BiquadCoefficients, Error> {
    let sample_rate = sample_rate as f32;
    let frequency = frequency.min(sample_rate * 0.499);
    let omega = TAU * frequency / sample_rate;
    let sin = omega.sin();
    let cos = omega.cos();
    let alpha = sin / (2.0 * resonance);
    let amplitude = 10.0f32.powf(gain_db / 40.0);
    if !amplitude.is_finite() || amplitude == 0.0 {
        return Err(Error::InvalidEqualizerGain);
    }

    let coefficients = match kind {
        EqualizerBandKind::Peaking => normalize_biquad(
            1.0 + alpha * amplitude,
            -2.0 * cos,
            1.0 - alpha * amplitude,
            1.0 + alpha / amplitude,
            -2.0 * cos,
            1.0 - alpha / amplitude,
        ),
        EqualizerBandKind::LowShelf => {
            let term = 2.0 * amplitude.sqrt() * alpha;
            normalize_biquad(
                amplitude * ((amplitude + 1.0) - (amplitude - 1.0) * cos + term),
                2.0 * amplitude * ((amplitude - 1.0) - (amplitude + 1.0) * cos),
                amplitude * ((amplitude + 1.0) - (amplitude - 1.0) * cos - term),
                (amplitude + 1.0) + (amplitude - 1.0) * cos + term,
                -2.0 * ((amplitude - 1.0) + (amplitude + 1.0) * cos),
                (amplitude + 1.0) + (amplitude - 1.0) * cos - term,
            )
        }
        EqualizerBandKind::HighShelf => {
            let term = 2.0 * amplitude.sqrt() * alpha;
            normalize_biquad(
                amplitude * ((amplitude + 1.0) + (amplitude - 1.0) * cos + term),
                -2.0 * amplitude * ((amplitude - 1.0) + (amplitude + 1.0) * cos),
                amplitude * ((amplitude + 1.0) + (amplitude - 1.0) * cos - term),
                (amplitude + 1.0) - (amplitude - 1.0) * cos + term,
                2.0 * ((amplitude - 1.0) - (amplitude + 1.0) * cos),
                (amplitude + 1.0) - (amplitude - 1.0) * cos - term,
            )
        }
    };

    Ok(coefficients)
}

fn normalize_biquad(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> BiquadCoefficients {
    let a0_recip = a0.recip();
    BiquadCoefficients {
        b0: b0 * a0_recip,
        b1: b1 * a0_recip,
        b2: b2 * a0_recip,
        a1: a1 * a0_recip,
        a2: a2 * a0_recip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled(kind: EffectKind) -> Effect {
        Effect { enabled: true, kind }
    }

    #[test]
    fn standard_effects_produce_finite_audio_and_reconfigure() {
        let parameters = Effects {
            effects: vec![
                enabled(EffectKind::HighPassFilter { frequency: 80.0, resonance: 0.707 }),
                enabled(EffectKind::Compressor {
                    threshold_db: -18.0,
                    ratio: 4.0,
                    attack_ms: 10.0,
                    release_ms: 100.0,
                    makeup_gain_db: 2.0,
                }),
                enabled(EffectKind::Limiter { threshold_db: -1.0, release_ms: 50.0 }),
                enabled(EffectKind::Delay { time_ms: 40.0, feedback: 0.3, mix: 0.2 }),
                enabled(EffectKind::Reverb { room_size: 0.5, damping: 0.4, mix: 0.2 }),
                enabled(EffectKind::Chorus {
                    rate_hz: 0.8,
                    depth_ms: 3.0,
                    delay_ms: 12.0,
                    mix: 0.3,
                }),
                enabled(EffectKind::Distortion { drive_db: 8.0, mix: 0.25 }),
            ],
        };
        let mut effects = RtEffects::new(&parameters, 2, 48000).unwrap();
        let mut block = AudioBlock::new(2, 256);
        assert!(block.resize(256));
        for (channel_index, channel) in block.samples_mut().iter_mut().enumerate() {
            for (sample_index, sample) in channel.iter_mut().enumerate() {
                *sample = ((sample_index as f32 * 0.07) + channel_index as f32).sin() * 0.5;
            }
        }

        effects.apply(&mut block);
        effects.reconfigure(2, 44100).unwrap();
        effects.apply(&mut block);

        assert!(block.samples_mut().iter().flatten().all(|sample| sample.is_finite()));
    }

    #[test]
    fn delay_outputs_an_impulse_after_its_configured_time() {
        let parameters = Effects {
            effects: vec![enabled(EffectKind::Delay { time_ms: 1.0, feedback: 0.0, mix: 1.0 })],
        };
        let mut effects = RtEffects::new(&parameters, 1, 1000).unwrap();
        let mut block = AudioBlock::new(1, 4);
        assert!(block.resize(4));
        block.samples_mut()[0][0] = 1.0;

        effects.apply(&mut block);

        assert_eq!(block.samples_mut()[0][0], 0.0);
        assert_eq!(block.samples_mut()[0][1], 1.0);
    }

    #[test]
    fn disabled_effect_is_a_passthrough() {
        let parameters = Effects {
            effects: vec![Effect { enabled: false, kind: EffectKind::Gain { value: 0.0 } }],
        };
        let mut effects = RtEffects::new(&parameters, 1, 48000).unwrap();
        let mut block = AudioBlock::new(1, 1);
        assert!(block.resize(1));
        block.samples_mut()[0][0] = 0.75;

        effects.apply(&mut block);

        assert_eq!(block.samples_mut()[0][0], 0.75);
    }

    #[test]
    fn missing_enabled_field_defaults_to_enabled() {
        let effects: Effects = serde_json::from_str(
            r#"{"version":1,"effects":[{"type":"Gain","data":{"value":1.0}}]}"#,
        )
        .unwrap();

        assert!(effects.effects[0].enabled);
    }
}
