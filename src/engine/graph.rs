use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const DEFAULT_CHANNEL_COUNT: usize = 2;
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;
pub const DEFAULT_BLOCK_SIZE: usize = 128;
pub const RETIREMENT_QUEUE_CAPACITY: usize = 16;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
#[schema(value_type = usize)]
pub struct LocalId(pub usize);

impl From<usize> for LocalId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<LocalId> for usize {
    fn from(id: LocalId) -> Self {
        id.0
    }
}
