use crate::{
    APP_VERSION,
    audio::AudioBlock,
    midi::MidiEvent,
    synths::{Error, Result},
};
use clack_host::{
    host::{HostHandlers, HostInfo, SharedHandler},
    plugin::PluginInstance,
};
use std::path::Path;

pub const DEFAULT_VOICE_COUNT: usize = 64;

struct AawHostShared;

impl<'a> SharedHandler<'a> for AawHostShared {
    /* ... */
    fn request_restart(&self) { /* ... */
    }
    fn request_process(&self) { /* ... */
    }
    fn request_callback(&self) { /* ... */
    }
}

struct AawHost;

impl HostHandlers for AawHost {
    type Shared<'a> = AawHostShared;

    type MainThread<'a> = ();
    type AudioProcessor<'a> = ();
}

struct AawPluginInstance(PluginInstance<AawHost>);

unsafe impl Send for AawPluginInstance {}

pub struct Synth {
    instance: AawPluginInstance,
}

impl Synth {
    pub async fn load(
        sample_rate: f64,
        voice_count: usize,
        path: impl AsRef<Path>,
        plugin_id: String,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let plugin_instance = tokio::task::spawn_blocking(move || {
            load_plugin_sync(&path, &plugin_id)
        })
        .await??;

        todo!()
    }

    pub fn silence(&mut self) {}

    pub fn handle_midi_event(&mut self, event: MidiEvent) {}

    pub fn render_audio_block(&mut self, output: &mut AudioBlock) -> Result<()> {
        Ok(())
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> Result<()> {
        Ok(())
    }

    pub fn set_max_block_size(&mut self, max_block_size: usize) -> Result<()> {
        Ok(())
    }
}

fn load_plugin_sync(path: impl AsRef<Path>, plugin_id: &str) -> Result<AawPluginInstance> {
    let entry = unsafe {
        clack_host::entry::PluginEntry::load(path.as_ref()).map_err(|e| Error::ClapPluginLoad {
            plugin_path: path.as_ref().to_owned(),
            plugin_id: plugin_id.to_string(),
            source: e,
        })?
    };

    let plugin_factory = entry.get_plugin_factory().ok_or_else(|| Error::InvalidClapPlugin {
        plugin_path: path.as_ref().to_owned(),
        plugin_id: plugin_id.to_string(),
    })?;

    let plugin_descriptor = plugin_factory
        .plugin_descriptors()
        .find(|d| d.id().is_some_and(|id| id.to_bytes() == plugin_id.as_bytes()))
        .ok_or_else(|| Error::MissingClapPlugin {
            plugin_path: path.as_ref().to_owned(),
            plugin_id: plugin_id.to_string(),
        })?;

    let instance = PluginInstance::<AawHost>::new(
        |_| AawHostShared,
        |_| (),
        &entry,
        plugin_descriptor.id().unwrap(),
        &host_info(),
    )
    .map_err(|e| Error::ClapPluginInstance {
        plugin_path: path.as_ref().to_owned(),
        plugin_id: plugin_id.to_string(),
        source: e,
    })?;

    Ok(AawPluginInstance(instance))
}

fn host_info() -> HostInfo {
    HostInfo::new("Arri's Audio Workstation", "Arri", "", APP_VERSION)
        .expect("HostInfo expected not to contain any NULL characters. This is a bug.")
}
