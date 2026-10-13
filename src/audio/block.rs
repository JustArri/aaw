pub struct AudioBlock {
    channel_count: usize,
    max_sample_count: usize,
    sample_count: usize,
    samples: Vec<Vec<f32>>,
}

impl AudioBlock {
    pub fn new(channel_count: usize, max_sample_count_per_channel: usize) -> Self {
        Self {
            channel_count,
            max_sample_count: max_sample_count_per_channel,
            sample_count: 0,
            samples: vec![vec![0.0; max_sample_count_per_channel]; channel_count],
        }
    }

    pub fn mirror(&self) -> Self {
        Self::new(self.channel_count, self.max_sample_count)
    }

    pub fn resize(&mut self, sample_count_per_channel: usize) -> bool {
        if sample_count_per_channel > self.max_sample_count {
            self.sample_count = 0;
            return false;
        }
        self.sample_count = sample_count_per_channel;
        true
    }

    pub fn clear(&mut self) {
        for channel in 0..self.channel_count {
            self.samples[channel][0..self.sample_count].fill(0.0);
        }
    }

    pub fn sample_count_per_channel(&self) -> usize {
        self.sample_count
    }

    pub fn channel_count(&self) -> usize {
        self.channel_count
    }

    pub fn samples_mut(&mut self) -> &mut Vec<Vec<f32>> {
        &mut self.samples
    }

    pub fn add_samples_from_unchecked(&mut self, other: &AudioBlock) {
        for channel in 0..self.channel_count {
            let local_samples = &mut self.samples[channel];
            let other_samples = &other.samples[channel];

            for i in 0..self.sample_count {
                local_samples[i] += other_samples[i];
            }
        }
    }

    pub fn write_interleaved_f32(&self, output: &mut [f32]) {
        for sample in 0..self.sample_count {
            for channel in 0..self.channel_count {
                output[sample * self.channel_count + channel] =
                    self.samples[channel][sample].clamp(-1.0, 1.0);
            }
        }
    }

    pub fn write_stacked_f32(&self, output: &mut [f32]) {
        for channel in 0..self.channel_count {
            for sample in 0..self.sample_count {
                output[sample * self.channel_count + channel] =
                    self.samples[channel][sample].clamp(-1.0, 1.0);
            }
        }
    }
}
