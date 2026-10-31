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
pub struct ControllerId(pub usize);

impl From<usize> for ControllerId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<ControllerId> for usize {
    fn from(id: ControllerId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ControllerAddress {
    #[schema(value_type = usize)]
    pub instrument_id: InstrumentId,

    #[schema(value_type = usize)]
    pub controller_id: ControllerId,
}
