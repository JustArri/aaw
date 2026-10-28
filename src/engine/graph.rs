use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const RETIREMENT_QUEUE_CAPACITY: usize = 16;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
#[schema(value_type = usize)]
pub struct InstrumentId(pub usize);

impl From<usize> for InstrumentId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<InstrumentId> for usize {
    fn from(id: InstrumentId) -> Self {
        id.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
#[schema(value_type = usize)]
pub struct SynthId(pub usize);

impl From<usize> for SynthId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<SynthId> for usize {
    fn from(id: SynthId) -> Self {
        id.0
    }
}
