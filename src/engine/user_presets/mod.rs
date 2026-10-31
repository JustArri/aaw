pub use ctr::*;
pub use stored::*;

mod ctr;
mod stored;

pub fn prepare(stored: StoredUserPresets) -> CtrUserPresets {
    let entries = stored
        .entries
        .into_iter()
        .map(|stored| CtrUserPreset { name: stored.name, controllers: stored.controllers })
        .collect();
    CtrUserPresets { entries }
}
