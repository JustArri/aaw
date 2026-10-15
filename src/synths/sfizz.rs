use crate::audio::block::AudioBlock;
use crate::ffi::sfizz as sys;
use crate::midi::{MidiEvent, MidiEventKind};
use crate::synths::{Error, Result, path_to_c_string};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Clone)]
pub struct SfzAsset {
    source: Arc<PathBuf>,
    text: Arc<str>,
}

impl std::fmt::Debug for SfzAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SfzAsset")
            .field("source", &self.source)
            .field("text_len", &self.text.len())
            .finish()
    }
}

impl SfzAsset {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|source| Error::AssetIo { path: path.clone(), source })?;
        Ok(Self { source: Arc::new(path), text: text.into() })
    }

    pub fn from_text(source: impl Into<PathBuf>, text: impl Into<Arc<str>>) -> Self {
        Self { source: Arc::new(source.into()), text: text.into() }
    }

    pub fn source(&self) -> &Path {
        self.source.as_ref()
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SfzInfo {
    pub key_ranges: Vec<SfzKeyRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SfzKeyRange {
    pub start: u8,
    pub end: u8,
}

enum MessageReply {
    None,
    Bool(bool),
    KeyRange(i32, i32),
    Invalid,
}

struct MessageClient(NonNull<sys::sfizz_client_t>);

impl MessageClient {
    fn new(reply: &mut MessageReply) -> Result<Self> {
        NonNull::new(unsafe { sys::sfizz_create_client(std::ptr::from_mut(reply).cast()) })
            .map(Self)
            .ok_or(Error::Native("sfizz could not create a metadata client"))
    }

    fn query(&self, synth: NonNull<sys::sfizz_synth_t>, path: &CStr, reply: &mut MessageReply) {
        *reply = MessageReply::None;
        unsafe {
            sys::sfizz_send_message(
                synth.as_ptr(),
                self.0.as_ptr(),
                0,
                path.as_ptr(),
                c"".as_ptr(),
                std::ptr::null(),
            );
        }
    }
}

impl Drop for MessageClient {
    fn drop(&mut self) {
        unsafe { sys::sfizz_delete_client(self.0.as_ptr()) };
    }
}

pub struct Synth {
    synth: NonNull<sys::sfizz_synth_t>,
    max_block_size: usize,
    latest_event_delay: Option<usize>,
    _not_sync: PhantomData<Cell<()>>,
}

// SAFETY: sfizz explicitly permits its RT/CT task roles to move between
// threads when synchronized. This wrapper has unique ownership and only
// exposes mutation through `&mut self`; `Cell` keeps it !Sync.
unsafe impl Send for Synth {}

impl Synth {
    pub async fn load(
        sample_rate: f32,
        max_block_size: usize,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        let asset = SfzAsset::load(path).await?;
        tokio::task::spawn_blocking(move || Self::from_asset(sample_rate, max_block_size, &asset))
            .await
            .map_err(Error::AssetTask)?
    }

    pub fn from_asset(sample_rate: f32, max_block_size: usize, asset: &SfzAsset) -> Result<Self> {
        let mut synth = Self::new(sample_rate, max_block_size)?;
        synth.load_asset(asset)?;
        Ok(synth)
    }

    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let c_path = path_to_c_string(path)?;
        if unsafe { sys::sfizz_load_file(self.synth.as_ptr(), c_path.as_ptr()) } {
            Ok(())
        } else {
            Err(Error::InstrumentLoad(path.to_owned()))
        }
    }

    pub fn load_asset(&mut self, asset: &SfzAsset) -> Result<()> {
        let path = path_to_c_string(asset.source())?;
        let text = std::ffi::CString::new(asset.text())
            .map_err(|_| Error::InvalidArgument("SFZ source text must not contain a NUL byte"))?;
        if unsafe { sys::sfizz_load_string(self.synth.as_ptr(), path.as_ptr(), text.as_ptr()) } {
            Ok(())
        } else {
            Err(Error::InstrumentLoad(asset.source().to_owned()))
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> Result<()> {
        if sample_rate == 0 {
            return Err(Error::InvalidArgument("sample rate must be greater than zero"));
        }
        unsafe { sys::sfizz_set_sample_rate(self.synth.as_ptr(), sample_rate as f32) };
        Ok(())
    }

    pub fn silence(&mut self) {
        unsafe { sys::sfizz_all_sound_off(self.synth.as_ptr()) };
    }

    pub fn sfz_info(&mut self) -> Result<SfzInfo> {
        let region_count =
            usize::try_from(unsafe { sys::sfizz_get_num_regions(self.synth.as_ptr()) })
                .map_err(|_| Error::Native("sfizz returned an invalid region count"))?;
        let mut reply = MessageReply::None;
        let client = MessageClient::new(&mut reply)?;
        unsafe { sys::sfizz_set_receive_callback(client.0.as_ptr(), Some(receive_metadata)) };
        let mut keys = [false; 128];

        for region in 0..region_count {
            let enabled_path = message_path(region, "enabled")?;
            client.query(self.synth, &enabled_path, &mut reply);
            if !matches!(reply, MessageReply::Bool(true)) {
                continue;
            }

            let trigger_path = message_path(region, "trigger_on_note")?;
            client.query(self.synth, &trigger_path, &mut reply);
            if !matches!(reply, MessageReply::Bool(true)) {
                continue;
            }

            let range_path = message_path(region, "key_range")?;
            client.query(self.synth, &range_path, &mut reply);
            let MessageReply::KeyRange(start, end) = reply else {
                return Err(Error::Native("sfizz returned an invalid region key range"));
            };
            let start = u8::try_from(start)
                .ok()
                .filter(|key| *key < 128)
                .ok_or(Error::Native("sfizz returned an invalid region key range"))?;
            let end = u8::try_from(end)
                .ok()
                .filter(|key| *key < 128 && *key >= start)
                .ok_or(Error::Native("sfizz returned an invalid region key range"))?;
            keys[usize::from(start)..=usize::from(end)].fill(true);
        }

        Ok(SfzInfo { key_ranges: key_ranges_from_mask(keys) })
    }

    pub fn handle_midi_event(&mut self, event: MidiEvent) {
        let delay = 0;

        match event.kind {
            MidiEventKind::NoteOn { key, velocity } => self.note_on(delay, key, velocity),
            MidiEventKind::NoteOff { key, velocity } => self.note_off(delay, key, velocity),
            MidiEventKind::ControlChange { controller, value } => {
                unsafe {
                    sys::sfizz_send_cc(
                        self.synth.as_ptr(),
                        delay as i32,
                        controller.into(),
                        value.into(),
                    );
                }
                self.remember_event(delay);
            }
            MidiEventKind::PitchBend { value } => {
                unsafe {
                    sys::sfizz_send_pitch_wheel(
                        self.synth.as_ptr(),
                        delay as i32,
                        i32::from(value),
                    );
                }
                self.remember_event(delay);
            }
            MidiEventKind::ProgramChange { program } => {
                unsafe {
                    sys::sfizz_send_program_change(
                        self.synth.as_ptr(),
                        delay as i32,
                        program.into(),
                    );
                }
                self.remember_event(delay);
            }
            MidiEventKind::ChannelAftertouch { value } => {
                unsafe {
                    sys::sfizz_send_channel_aftertouch(
                        self.synth.as_ptr(),
                        delay as i32,
                        value.into(),
                    );
                }
                self.remember_event(delay);
            }
            MidiEventKind::PolyphonicAftertouch { key, value } => {
                unsafe {
                    sys::sfizz_send_poly_aftertouch(
                        self.synth.as_ptr(),
                        delay as i32,
                        key.into(),
                        value.into(),
                    );
                }
                self.remember_event(delay);
            }
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

    fn note_on(&mut self, delay: usize, note: u8, velocity: u8) {
        unsafe {
            sys::sfizz_send_note_on(
                self.synth.as_ptr(),
                delay as i32,
                note.into(),
                velocity.into(),
            );
        }
        self.remember_event(delay);
    }

    fn note_off(&mut self, delay: usize, note: u8, velocity: u8) {
        unsafe {
            sys::sfizz_send_note_off(
                self.synth.as_ptr(),
                delay as i32,
                note.into(),
                velocity.into(),
            );
        }
        self.remember_event(delay);
    }

    fn render(&mut self, left: &mut [f32], right: &mut [f32]) -> Result<()> {
        if left.len() != right.len() {
            return Err(Error::InvalidArgument(
                "left and right output buffers must have equal lengths",
            ));
        }
        if left.len() > self.max_block_size {
            return Err(Error::InvalidArgument(
                "audio block exceeds the configured maximum block size",
            ));
        }
        if self.latest_event_delay.is_some_and(|delay| delay >= left.len()) {
            return Err(Error::InvalidArgument("scheduled event falls outside the audio block"));
        }
        let frames = i32::try_from(left.len())
            .map_err(|_| Error::InvalidArgument("audio block is too large"))?;
        let mut channels = [left.as_mut_ptr(), right.as_mut_ptr()];
        unsafe {
            sys::sfizz_render_block(self.synth.as_ptr(), channels.as_mut_ptr(), 2, frames);
        }
        self.latest_event_delay = None;
        Ok(())
    }

    fn remember_event(&mut self, delay: usize) {
        self.latest_event_delay =
            Some(self.latest_event_delay.map_or(delay, |current| current.max(delay)));
    }

    fn new(sample_rate: f32, max_block_size: usize) -> Result<Self> {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return Err(Error::InvalidArgument("sample rate must be finite and greater than zero"));
        }
        let block_size = i32::try_from(max_block_size)
            .ok()
            .filter(|size| *size > 0)
            .ok_or(Error::InvalidArgument("maximum block size must be in 1..=i32::MAX"))?;

        let synth = NonNull::new(unsafe { sys::sfizz_create_synth() })
            .ok_or(Error::Native("sfizz could not create a synthesizer"))?;
        unsafe {
            sys::sfizz_set_sample_rate(synth.as_ptr(), sample_rate);
            sys::sfizz_set_samples_per_block(synth.as_ptr(), block_size);
        }

        Ok(Self { synth, max_block_size, latest_event_delay: None, _not_sync: PhantomData })
    }
}

impl Drop for Synth {
    fn drop(&mut self) {
        unsafe { sys::sfizz_free(self.synth.as_ptr()) };
    }
}

fn message_path(region: usize, property: &str) -> Result<CString> {
    CString::new(format!("/region{region}/{property}"))
        .map_err(|_| Error::Native("sfizz metadata path contains a NUL byte"))
}

fn key_ranges_from_mask(keys: [bool; 128]) -> Vec<SfzKeyRange> {
    let mut ranges = Vec::new();
    let mut start = None;

    for (key, available) in keys.into_iter().chain([false]).enumerate() {
        match (start, available) {
            (None, true) => start = Some(key as u8),
            (Some(first), false) => {
                ranges.push(SfzKeyRange { start: first, end: (key - 1) as u8 });
                start = None;
            }
            _ => {}
        }
    }

    ranges
}

unsafe extern "C" fn receive_metadata(
    data: *mut c_void,
    _delay: c_int,
    _path: *const c_char,
    signature: *const c_char,
    args: *const sys::sfizz_arg_t,
) {
    if data.is_null() || signature.is_null() {
        return;
    }

    let reply = unsafe { &mut *data.cast::<MessageReply>() };
    let signature = unsafe { CStr::from_ptr(signature) }.to_bytes();
    *reply = match signature {
        b"T" => MessageReply::Bool(true),
        b"F" => MessageReply::Bool(false),
        b"ii" if !args.is_null() => {
            MessageReply::KeyRange(unsafe { (*args).i }, unsafe { (*args.add(1)).i })
        }
        _ => MessageReply::Invalid,
    };
}

#[cfg(test)]
mod tests {
    use super::{SfzAsset, SfzKeyRange, Synth, key_ranges_from_mask};

    #[test]
    fn sfz_info_merges_enabled_note_triggered_region_ranges() {
        let asset = SfzAsset::from_text(
            "ranges.sfz",
            concat!(
                "<region> sample=*sine lokey=10 hikey=20\n",
                "<region> sample=*sine lokey=18 hikey=30\n",
                "<region> sample=*sine key=40\n",
            ),
        );
        let mut synth = Synth::from_asset(48_000.0, 64, &asset).unwrap();

        assert_eq!(
            synth.sfz_info().unwrap().key_ranges,
            [SfzKeyRange { start: 10, end: 30 }, SfzKeyRange { start: 40, end: 40 },]
        );
    }

    #[test]
    fn key_mask_is_merged_into_inclusive_ranges() {
        let mut keys = [false; 128];
        keys[0] = true;
        keys[2..=4].fill(true);
        keys[126..=127].fill(true);

        assert_eq!(
            key_ranges_from_mask(keys),
            [
                SfzKeyRange { start: 0, end: 0 },
                SfzKeyRange { start: 2, end: 4 },
                SfzKeyRange { start: 126, end: 127 },
            ]
        );
    }
}
