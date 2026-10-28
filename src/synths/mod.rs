pub use dummy::{DEFAULT_VOICE_COUNT as DUMMY_DEFAULT_VOICE_COUNT, Dummy};
pub use error::{Error, Result};
pub use fluidsynth::{
    DEFAULT_VOICE_COUNT as FLUID_DEFAULT_VOICE_COUNT, SoundFontBankInfo, SoundFontInfo,
    SoundFontNoteRange, SoundFontPresetInfo, SoundFontProgram, Synth as Fluid,
};
pub use sfizz::{
    DEFAULT_VOICE_COUNT as SFIZZ_DEFAULT_VOICE_COUNT, MAX_VOICE_COUNT as SFIZZ_MAX_VOICE_COUNT,
    SfzInfo, SfzKeyRange, Synth as Sfizz,
};

mod dummy;
mod error;
mod fluidsynth;
mod sfizz;

pub(crate) fn path_to_c_string(path: &std::path::Path) -> Result<std::ffi::CString> {
    let text = path.to_str().ok_or_else(|| Error::NonUtf8Path(path.to_owned()))?;
    std::ffi::CString::new(text).map_err(|_| Error::PathContainsNul(path.to_owned()))
}
