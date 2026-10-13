pub use dummy::Dummy;
pub use error::{Error, Result};
pub use fluidsynth::{
    SoundFontBankInfo, SoundFontInfo, SoundFontNoteRange, SoundFontPresetInfo, Synth as Fluid,
};
pub use sfizz::{SfzInfo, SfzKeyRange, Synth as Sfizz};

mod dummy;
mod error;
mod fluidsynth;
mod sfizz;

pub(crate) fn path_to_c_string(path: &std::path::Path) -> Result<std::ffi::CString> {
    let text = path.to_str().ok_or_else(|| Error::NonUtf8Path(path.to_owned()))?;
    std::ffi::CString::new(text).map_err(|_| Error::PathContainsNul(path.to_owned()))
}
