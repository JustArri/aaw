//! Safe ownership and rendering wrapper around FluidSynth.
use crate::audio::block::AudioBlock;
use crate::ffi::fluidsynth as sys;
use crate::midi::{MidiEvent, MidiEventKind};
use crate::synths::{Error, Result, path_to_c_string};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::Arc;
use utoipa::ToSchema;

thread_local! {
    static ACTIVE_SOUNDFONT: RefCell<Option<Arc<[u8]>>> = const { RefCell::new(None) };
}

#[derive(Clone)]
pub struct SoundFontAsset {
    source: Arc<PathBuf>,
    bytes: Arc<[u8]>,
}

impl std::fmt::Debug for SoundFontAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundFontAsset")
            .field("source", &self.source)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl SoundFontAsset {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|source| Error::AssetIo { path: path.clone(), source })?;
        Ok(Self { source: Arc::new(path), bytes: bytes.into() })
    }

    pub fn from_bytes(source: impl Into<PathBuf>, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self { source: Arc::new(source.into()), bytes: bytes.into() }
    }

    pub fn source(&self) -> &Path {
        self.source.as_ref()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn shared_bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }
}

struct MemoryFile {
    bytes: Arc<[u8]>,
    position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SoundFontInfo {
    pub name: String,
    pub banks: Vec<SoundFontBankInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SoundFontBankInfo {
    pub number: u16,
    pub presets: Vec<SoundFontPresetInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SoundFontPresetInfo {
    pub number: u8,
    pub name: String,
    pub note_ranges: Vec<SoundFontNoteRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SoundFontNoteRange {
    pub start: u8,
    pub end: u8,
}

pub struct Synth {
    synth: NonNull<sys::fluid_synth_t>,
    settings: NonNull<sys::fluid_settings_t>,
    soundfont_id: Option<i32>,
    _not_sync: PhantomData<Cell<()>>,
}

// SAFETY: the wrapper uniquely owns both native objects, all access requires
// `&mut self`, and moving an idle FluidSynth instance between threads is not a
// concurrent library call. `Cell` deliberately keeps the type !Sync.
unsafe impl Send for Synth {}

impl Synth {
    pub async fn load(sample_rate: f64, path: impl AsRef<Path>) -> Result<Self> {
        let asset = SoundFontAsset::load(path).await?;
        tokio::task::spawn_blocking(move || Self::from_asset(sample_rate, &asset))
            .await
            .map_err(Error::AssetTask)?
    }

    pub fn from_asset(sample_rate: f64, asset: &SoundFontAsset) -> Result<Self> {
        let mut synth = Self::new(sample_rate)?;
        synth.load_asset(asset)?;
        Ok(synth)
    }

    pub fn load_soundfont(&mut self, path: impl AsRef<Path>) -> Result<i32> {
        let path = path.as_ref();
        let c_path = path_to_c_string(path)?;
        let id = unsafe { sys::fluid_synth_sfload(self.synth.as_ptr(), c_path.as_ptr(), 1) };
        if id < 0 {
            Err(Error::InstrumentLoad(path.to_owned()))
        } else {
            self.soundfont_id = Some(id);
            Ok(id)
        }
    }

    pub fn load_asset(&mut self, asset: &SoundFontAsset) -> Result<i32> {
        let already_loading = ACTIVE_SOUNDFONT.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                true
            } else {
                *slot = Some(asset.shared_bytes());
                false
            }
        });
        if already_loading {
            return Err(Error::Native("a nested SoundFont load is not supported"));
        }

        let id = unsafe { sys::fluid_synth_sfload(self.synth.as_ptr(), c"memory.sf2".as_ptr(), 1) };
        ACTIVE_SOUNDFONT.with(|slot| *slot.borrow_mut() = None);
        if id < 0 {
            Err(Error::InstrumentLoad(asset.source().to_owned()))
        } else {
            self.soundfont_id = Some(id);
            Ok(id)
        }
    }

    pub fn soundfont_info(&mut self) -> Result<SoundFontInfo> {
        let soundfont_id =
            self.soundfont_id.ok_or(Error::Native("FluidSynth has no loaded SoundFont"))?;
        let soundfont = NonNull::new(unsafe {
            sys::fluid_synth_get_sfont_by_id(self.synth.as_ptr(), soundfont_id)
        })
        .ok_or(Error::Native("FluidSynth could not find the loaded SoundFont"))?;

        let name = copy_native_string(
            unsafe { sys::fluid_sfont_get_name(soundfont.as_ptr()) },
            "FluidSynth SoundFont has no name",
        )?;
        let mut banks = BTreeMap::<u16, Vec<SoundFontPresetInfo>>::new();

        unsafe { sys::fluid_sfont_iteration_start(soundfont.as_ptr()) };
        loop {
            let preset =
                NonNull::new(unsafe { sys::fluid_sfont_iteration_next(soundfont.as_ptr()) });
            let Some(preset) = preset else {
                break;
            };

            let bank = unsafe { sys::fluid_preset_get_banknum(preset.as_ptr()) };
            let bank = u16::try_from(bank)
                .ok()
                .filter(|bank| *bank <= 16_383)
                .ok_or(Error::Native("FluidSynth returned an invalid preset bank"))?;
            let number = unsafe { sys::fluid_preset_get_num(preset.as_ptr()) };
            let number = u8::try_from(number)
                .ok()
                .filter(|number| *number <= 127)
                .ok_or(Error::Native("FluidSynth returned an invalid preset number"))?;
            let name = copy_native_string(
                unsafe { sys::fluid_preset_get_name(preset.as_ptr()) },
                "FluidSynth preset has no name",
            )?;
            let note_ranges = preset_note_ranges(preset)?;

            banks.entry(bank).or_default().push(SoundFontPresetInfo { number, name, note_ranges });
        }

        let banks = banks
            .into_iter()
            .map(|(number, mut presets)| {
                presets.sort_unstable_by_key(|preset| preset.number);
                SoundFontBankInfo { number, presets }
            })
            .collect();

        Ok(SoundFontInfo { name, banks })
    }

    pub fn bank_select(&mut self, bank: u16) -> Result<()> {
        native_result(
            unsafe { sys::fluid_synth_bank_select(self.synth.as_ptr(), -1, bank.into()) },
            "FluidSynth all-notes-off failed",
        )
    }

    pub fn program_change(&mut self, preset_id: u8) -> Result<()> {
        native_result(
            unsafe { sys::fluid_synth_program_change(self.synth.as_ptr(), -1, preset_id.into()) },
            "FluidSynth all-notes-off failed",
        )
    }

    pub fn silence(&mut self) -> Result<()> {
        native_result(
            unsafe { sys::fluid_synth_all_sounds_off(self.synth.as_ptr(), -1) },
            "FluidSynth all-notes-off failed",
        )
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> Result<()> {
        if sample_rate == 0 {
            return Err(Error::InvalidArgument("sample rate must be greater than zero"));
        }
        unsafe {
            sys::fluid_synth_set_sample_rate(self.synth.as_ptr(), sample_rate as f32);
        }
        Ok(())
    }

    pub fn handle_midi_event(&mut self, event: MidiEvent) -> Result<()> {
        let channel = i32::from(event.channel);

        match event.kind {
            MidiEventKind::NoteOn { key, velocity } => self.note_on(event.channel, key, velocity),
            MidiEventKind::NoteOff { key, .. } => self.note_off(event.channel, key),
            MidiEventKind::ControlChange { controller, value } => native_result(
                unsafe {
                    sys::fluid_synth_cc(
                        self.synth.as_ptr(),
                        channel,
                        controller.into(),
                        value.into(),
                    )
                },
                "FluidSynth control-change failed",
            ),
            MidiEventKind::PitchBend { value } => native_result(
                unsafe {
                    sys::fluid_synth_pitch_bend(
                        self.synth.as_ptr(),
                        channel,
                        pitch_bend_to_unsigned(value),
                    )
                },
                "FluidSynth pitch-bend failed",
            ),
            MidiEventKind::ProgramChange { program } => native_result(
                unsafe {
                    sys::fluid_synth_program_change(self.synth.as_ptr(), channel, program.into())
                },
                "FluidSynth program-change failed",
            ),
            MidiEventKind::ChannelAftertouch { value } => native_result(
                unsafe {
                    sys::fluid_synth_channel_pressure(self.synth.as_ptr(), channel, value.into())
                },
                "FluidSynth aftertouch failed",
            ),
            MidiEventKind::PolyphonicAftertouch { key, value } => native_result(
                unsafe {
                    sys::fluid_synth_key_pressure(
                        self.synth.as_ptr(),
                        channel,
                        key.into(),
                        value.into(),
                    )
                },
                "FluidSynth polyphonic aftertouch failed",
            ),
        }
    }

    pub fn render_audio_block(&mut self, output: &mut AudioBlock) -> Result<()> {
        if output.channel_count() < 2 {
            return Err(Error::InvalidArgument("native synthesizers require a stereo AudioBlock"));
        }

        let frame_count = output.sample_count_per_channel();
        let samples = output.samples_mut();

        for channel in &mut samples[2..] {
            channel[..frame_count].fill(0.0);
        }

        let (left, remaining) = samples.split_at_mut(1);

        self.render(&mut left[0][..frame_count], &mut remaining[0][..frame_count])
    }

    fn note_on(&mut self, channel: u8, note: u8, velocity: u8) -> Result<()> {
        native_result(
            unsafe {
                sys::fluid_synth_noteon(
                    self.synth.as_ptr(),
                    channel.into(),
                    note.into(),
                    velocity.into(),
                )
            },
            "FluidSynth note-on failed",
        )
    }

    fn note_off(&mut self, channel: u8, note: u8) -> Result<()> {
        native_result(
            unsafe { sys::fluid_synth_noteoff(self.synth.as_ptr(), channel.into(), note.into()) },
            "FluidSynth note-off failed",
        )
    }

    fn render(&mut self, left: &mut [f32], right: &mut [f32]) -> Result<()> {
        if left.len() != right.len() {
            return Err(Error::InvalidArgument(
                "left and right output buffers must have equal lengths",
            ));
        }
        let frames = i32::try_from(left.len())
            .map_err(|_| Error::InvalidArgument("audio block is too large"))?;
        native_result(
            unsafe {
                sys::fluid_synth_write_float(
                    self.synth.as_ptr(),
                    frames,
                    left.as_mut_ptr().cast::<c_void>(),
                    0,
                    1,
                    right.as_mut_ptr().cast::<c_void>(),
                    0,
                    1,
                )
            },
            "FluidSynth rendering failed",
        )
    }

    fn new(sample_rate: f64) -> Result<Self> {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return Err(Error::InvalidArgument("sample rate must be finite and greater than zero"));
        }

        let settings = NonNull::new(unsafe { sys::new_fluid_settings() })
            .ok_or(Error::Native("FluidSynth could not allocate settings"))?;
        let setting_result = unsafe {
            sys::fluid_settings_setnum(
                settings.as_ptr(),
                c"synth.sample-rate".as_ptr(),
                sample_rate,
            )
        };
        if setting_result != 0 {
            unsafe { sys::delete_fluid_settings(settings.as_ptr()) };
            return Err(Error::Native("FluidSynth rejected the sample rate"));
        }
        // Memory-backed assets are released after loading, so FluidSynth must
        // eagerly retain their samples rather than reopening them on demand.
        let dynamic_loading_result = unsafe {
            sys::fluid_settings_setint(
                settings.as_ptr(),
                c"synth.dynamic-sample-loading".as_ptr(),
                0,
            )
        };
        if dynamic_loading_result != 0 {
            unsafe { sys::delete_fluid_settings(settings.as_ptr()) };
            return Err(Error::Native("FluidSynth could not disable dynamic sample loading"));
        }

        let synth = match NonNull::new(unsafe { sys::new_fluid_synth(settings.as_ptr()) }) {
            Some(synth) => synth,
            None => {
                unsafe { sys::delete_fluid_settings(settings.as_ptr()) };
                return Err(Error::Native("FluidSynth could not create a synthesizer"));
            }
        };

        let loader = NonNull::new(unsafe { sys::new_fluid_defsfloader(settings.as_ptr()) });
        let Some(loader) = loader else {
            unsafe {
                sys::delete_fluid_synth(synth.as_ptr());
                sys::delete_fluid_settings(settings.as_ptr());
            }
            return Err(Error::Native(
                "FluidSynth could not create the in-memory SoundFont loader",
            ));
        };
        let callback_result = unsafe {
            sys::fluid_sfloader_set_callbacks(
                loader.as_ptr(),
                Some(memory_open),
                Some(memory_read),
                Some(memory_seek),
                Some(memory_tell),
                Some(memory_close),
            )
        };
        if callback_result != 0 {
            unsafe {
                sys::delete_fluid_sfloader(loader.as_ptr());
                sys::delete_fluid_synth(synth.as_ptr());
                sys::delete_fluid_settings(settings.as_ptr());
            }
            return Err(Error::Native("FluidSynth rejected the in-memory file callbacks"));
        }
        // The synth owns and eventually destroys the loader after this call.
        unsafe { sys::fluid_synth_add_sfloader(synth.as_ptr(), loader.as_ptr()) };

        Ok(Self { synth, settings, soundfont_id: None, _not_sync: PhantomData })
    }
}

impl Drop for Synth {
    fn drop(&mut self) {
        unsafe {
            sys::delete_fluid_synth(self.synth.as_ptr());
            sys::delete_fluid_settings(self.settings.as_ptr());
        }
    }
}

fn native_result(code: i32, message: &'static str) -> Result<()> {
    if code == 0 { Ok(()) } else { Err(Error::Native(message)) }
}

fn pitch_bend_to_unsigned(value: i16) -> i32 {
    (i32::from(value) + 8192).clamp(0, 16_383)
}

fn copy_native_string(value: *const c_char, missing: &'static str) -> Result<String> {
    if value.is_null() {
        return Err(Error::Native(missing));
    }
    Ok(unsafe { CStr::from_ptr(value) }.to_string_lossy().into_owned())
}

fn preset_note_ranges(preset: NonNull<sys::fluid_preset_t>) -> Result<Vec<SoundFontNoteRange>> {
    let mut notes = [0_u8; 128];
    let result = unsafe { sys::aaw_fluid_preset_note_mask(preset.as_ptr(), notes.as_mut_ptr()) };
    if result != 0 {
        return Err(Error::Native("FluidSynth could not inspect preset note ranges"));
    }

    Ok(note_ranges_from_mask(notes))
}

fn note_ranges_from_mask(notes: [u8; 128]) -> Vec<SoundFontNoteRange> {
    let mut ranges = Vec::new();
    let mut start = None;

    for (note, available) in notes.into_iter().chain([0]).enumerate() {
        match (start, available != 0) {
            (None, true) => start = Some(note as u8),
            (Some(first), false) => {
                ranges.push(SoundFontNoteRange { start: first, end: (note - 1) as u8 });
                start = None;
            }
            _ => {}
        }
    }

    ranges
}

unsafe extern "C" fn memory_open(_filename: *const c_char) -> *mut c_void {
    ACTIVE_SOUNDFONT.with(|slot| {
        let Some(bytes) = slot.borrow().as_ref().map(Arc::clone) else {
            return std::ptr::null_mut();
        };
        Box::into_raw(Box::new(MemoryFile { bytes, position: 0 })).cast()
    })
}

unsafe extern "C" fn memory_read(buf: *mut c_void, count: i64, handle: *mut c_void) -> c_int {
    if buf.is_null() || handle.is_null() || count < 0 {
        return -1;
    }
    let file = unsafe { &mut *handle.cast::<MemoryFile>() };
    let Ok(count) = usize::try_from(count) else {
        return -1;
    };
    let Some(end) = file.position.checked_add(count) else {
        return -1;
    };
    if end > file.bytes.len() {
        return -1;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(file.bytes.as_ptr().add(file.position), buf.cast(), count);
    }
    file.position = end;
    0
}

unsafe extern "C" fn memory_seek(handle: *mut c_void, offset: i64, origin: c_int) -> c_int {
    if handle.is_null() {
        return -1;
    }
    let file = unsafe { &mut *handle.cast::<MemoryFile>() };
    let base = match origin {
        0 => 0_i128,
        1 => file.position as i128,
        2 => file.bytes.len() as i128,
        _ => return -1,
    };
    let position = base + i128::from(offset);
    if !(0..=file.bytes.len() as i128).contains(&position) {
        return -1;
    }
    file.position = position as usize;
    0
}

unsafe extern "C" fn memory_tell(handle: *mut c_void) -> i64 {
    if handle.is_null() {
        return -1;
    }
    let file = unsafe { &*handle.cast::<MemoryFile>() };
    i64::try_from(file.position).unwrap_or(-1)
}

unsafe extern "C" fn memory_close(handle: *mut c_void) -> c_int {
    if handle.is_null() {
        return -1;
    }
    drop(unsafe { Box::from_raw(handle.cast::<MemoryFile>()) });
    0
}

#[cfg(test)]
mod tests {
    use super::{SoundFontAsset, SoundFontNoteRange, Synth, note_ranges_from_mask};

    #[test]
    fn info_lists_banks_and_presets_from_the_loaded_soundfont() {
        let mut synth = Synth::new(48_000.0).unwrap();
        assert!(synth.soundfont_info().is_err());

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("lib/fluidsynth/sf2/VintageDreamsWaves-v2.sf2");
        let asset = SoundFontAsset::from_bytes(&path, std::fs::read(&path).unwrap());
        synth.load_asset(&asset).unwrap();
        let info = synth.soundfont_info().unwrap();

        assert!(!info.name.is_empty());
        assert!(!info.banks.is_empty());
        assert!(info.banks.windows(2).all(|banks| banks[0].number < banks[1].number));
        assert!(info.banks.iter().all(|bank| !bank.presets.is_empty()));
        assert!(info.banks.iter().all(|bank| {
            bank.presets.windows(2).all(|presets| presets[0].number <= presets[1].number)
        }));
        assert!(
            info.banks.iter().flat_map(|bank| &bank.presets).all(|preset| !preset.name.is_empty())
        );
        assert!(info.banks.iter().flat_map(|bank| &bank.presets).all(|preset| {
            !preset.note_ranges.is_empty()
                && preset.note_ranges.iter().all(|range| range.start <= range.end)
                && preset
                    .note_ranges
                    .windows(2)
                    .all(|ranges| ranges[0].end.saturating_add(1) < ranges[1].start)
        }));
    }

    #[test]
    fn note_mask_is_merged_into_inclusive_ranges() {
        let mut notes = [0; 128];
        notes[0] = 1;
        notes[2..=4].fill(1);
        notes[126..=127].fill(1);

        assert_eq!(
            note_ranges_from_mask(notes),
            [
                SoundFontNoteRange { start: 0, end: 0 },
                SoundFontNoteRange { start: 2, end: 4 },
                SoundFontNoteRange { start: 126, end: 127 },
            ]
        );
    }
}
