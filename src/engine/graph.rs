use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const DEFAULT_SAMPLE_RATE: u32 = 48000;
pub const DEFAULT_BLOCK_SIZE: usize = 128;
pub const RETIREMENT_QUEUE_CAPACITY: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum SynthKind {
    Dummy,
    Fluid,
    Sfizz,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
pub struct LocalSynthId(pub usize);

impl From<usize> for LocalSynthId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<LocalSynthId> for usize {
    fn from(id: LocalSynthId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SynthId(pub SynthKind, pub LocalSynthId);
