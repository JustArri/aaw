pub use ctr::*;
pub use rt::*;
pub use stored::*;

mod ctr;
mod rt;
mod stored;

pub fn prepare_engine(
    stored: StoredMidiEngineControls,
) -> (CtrMidiEngineControls, RtMidiEngineControls) {
    let ctr = CtrMidiEngineControls::from(stored);
    let rt = RtMidiEngineControls::from(&ctr);
    (ctr, rt)
}

pub fn prepare_project(
    stored: StoredMidiProjectControls,
) -> (CtrMidiProjectControls, RtMidiProjectControls) {
    let ctr = CtrMidiProjectControls::from(stored);
    let rt = RtMidiProjectControls::from(&ctr);
    (ctr, rt)
}
