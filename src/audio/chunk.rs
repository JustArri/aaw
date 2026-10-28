use crate::{MAX_BLOCK_SIZE, audio::AudioBlock};

pub struct AudioRecordingChunk {
    pub left: [f32; MAX_BLOCK_SIZE],
    pub right: [f32; MAX_BLOCK_SIZE],
    pub block_size: usize,
}

impl Default for AudioRecordingChunk {
    fn default() -> Self {
        Self { left: [0.0; MAX_BLOCK_SIZE], right: [0.0; MAX_BLOCK_SIZE], block_size: 0 }
    }
}

impl AudioRecordingChunk {
    pub fn fill_from(&mut self, block: &AudioBlock) {
        let count = block.sample_count_per_channel();
        assert!(count <= MAX_BLOCK_SIZE);

        self.block_size = count;

        match block.channel_count() {
            0 => {
                self.left[..count].fill(0.0);
                self.right[..count].fill(0.0);
            }
            1 => {
                self.left[..count].copy_from_slice(&block.samples()[0][..count]);
                self.right[..count].fill(0.0);
            }
            _ => {
                self.left[..count].copy_from_slice(&block.samples()[0][..count]);
                self.right[..count].copy_from_slice(&block.samples()[1][..count]);
            }
        }
    }
}
