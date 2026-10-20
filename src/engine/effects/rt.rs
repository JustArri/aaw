use crate::{
    audio::block::AudioBlock,
    engine::effects::{CtrEffect, CtrEffects, CtrEqualizerBand, EFFECTS_TUNING, Error},
};
use arrayvec::ArrayVec;
use std::f32::consts::TAU;

pub struct RtEffects {
    effects: ArrayVec<Effect, { EFFECTS_TUNING.max_effects }>,
    channel_count: usize,
    sample_rate: u32,
}

impl RtEffects {
    pub fn new(
        parameters: &CtrEffects,
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
            effects.push(Effect::new(effect, channel_count, sample_rate)?);
        }

        Ok(Self { effects, channel_count, sample_rate })
    }

    pub(super) fn new_replacing_failed(
        parameters: &mut CtrEffects,
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
            match Effect::new(parameters, channel_count, sample_rate) {
                Ok(effect) => effects.push(effect),
                Err(error) => {
                    *parameters = CtrEffect::LoadFailed { description: error.to_string() };
                    effects.push(Effect::LoadFailed);
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

pub enum Effect {
    Gain(RtGain),
    LowPassFilter(RtLowPassFilter),
    Equalizer(RtEqualizer),
    LoadFailed,
}

impl Effect {
    fn new(parameters: &CtrEffect, channel_count: usize, sample_rate: u32) -> Result<Self, Error> {
        match parameters {
            CtrEffect::Gain { value } => Ok(Self::Gain(RtGain::new(*value)?)),
            CtrEffect::LowPassFilter { frequency, resonance } => Ok(Self::LowPassFilter(
                RtLowPassFilter::new(*frequency, *resonance, channel_count, sample_rate)?,
            )),
            CtrEffect::Equalizer { bands } => {
                Ok(Self::Equalizer(RtEqualizer::new(bands, channel_count, sample_rate)?))
            }
            CtrEffect::LoadFailed { .. } => Ok(Self::LoadFailed),
        }
    }

    fn reconfigure(&mut self, channel_count: usize, sample_rate: u32) -> Result<(), Error> {
        match self {
            Self::Gain(_) => Ok(()),
            Self::LowPassFilter(effect) => effect.reconfigure(channel_count, sample_rate),
            Self::Equalizer(effect) => effect.reconfigure(channel_count, sample_rate),
            Self::LoadFailed => Ok(()),
        }
    }

    pub fn apply(&mut self, block: &mut AudioBlock) {
        match self {
            Self::Gain(effect) => effect.apply(block),
            Self::LowPassFilter(effect) => effect.apply(block),
            Self::Equalizer(effect) => effect.apply(block),
            Self::LoadFailed => {}
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

pub struct RtEqualizer {
    bands: Vec<EqualizerBand>,
}

impl RtEqualizer {
    fn new(
        parameters: &[CtrEqualizerBand],
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
            bands.push(EqualizerBand::new(band, channel_count, sample_rate)?);
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

struct EqualizerBand {
    kind: EqualizerBandKind,
    frequency: f32,
    gain_db: f32,
    resonance: f32,
    coefficients: BiquadCoefficients,
    channel_states: Vec<FilterState>,
}

impl EqualizerBand {
    fn new(
        parameters: &CtrEqualizerBand,
        channel_count: usize,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        let (kind, frequency, gain_db, resonance) = match *parameters {
            CtrEqualizerBand::LowShelf { frequency, gain_db, resonance } => {
                (EqualizerBandKind::LowShelf, frequency, gain_db, resonance)
            }
            CtrEqualizerBand::Peaking { frequency, gain_db, resonance } => {
                (EqualizerBandKind::Peaking, frequency, gain_db, resonance)
            }
            CtrEqualizerBand::HighShelf { frequency, gain_db, resonance } => {
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
