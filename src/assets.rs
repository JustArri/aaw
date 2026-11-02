use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

pub const FLUID_EXTENSIONS: &[&str] = &["sf2", "sf3"];
pub const SFIZZ_EXTENSIONS: &[&str] = &["sfz"];
pub const CLAP_EXTENSIONS: &[&str] = &["clap", "so"];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClapInstrumentAsset {
    pub file_path: PathBuf,
    pub plugin_id: String,
    pub name: Option<String>,
    pub version: Option<String>,
}

pub struct Assets {
    pub dir: PathBuf,
    pub fluid_list: Vec<PathBuf>,
    pub sfizz_list: Vec<PathBuf>,
    pub clap_instrument_list: Vec<ClapInstrumentAsset>,
}

impl Assets {
    pub async fn new(dir: PathBuf) -> Result<Self, std::io::Error> {
        let mut fluid_list = Vec::new();
        let mut sfizz_list = Vec::new();
        let mut clap_instrument_list = Vec::new();
        let mut pending_dirs = vec![dir.clone()];

        while let Some(current_dir) = pending_dirs.pop() {
            let mut entries = fs::read_dir(current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let file_type = entry.file_type().await?;

                if file_type.is_dir() {
                    pending_dirs.push(entry.path());
                    continue;
                }

                if !file_type.is_file() {
                    continue;
                }

                let path = entry.path();
                let Some(extension) = path.extension().and_then(|extension| extension.to_str())
                else {
                    continue;
                };
                let relative_path = path
                    .strip_prefix(&dir)
                    .expect("a directory entry must be inside the scanned assets directory");

                if extension_matches(FLUID_EXTENSIONS, extension) {
                    fluid_list.push(relative_path.to_owned());
                }

                if extension_matches(SFIZZ_EXTENSIONS, extension) {
                    sfizz_list.push(relative_path.to_owned());
                }

                if extension_matches(CLAP_EXTENSIONS, extension)
                    && let Some(list) = list_clap_instruments(&path)
                {
                    for mut plugin in list {
                        plugin.file_path = relative_path.to_owned();
                        clap_instrument_list.push(plugin);
                    }
                }
            }
        }

        fluid_list.sort_unstable();
        sfizz_list.sort_unstable();
        clap_instrument_list.sort_unstable();

        info!(
            fluid_assets = fluid_list.len(),
            sfizz_assets = sfizz_list.len(),
            clap_instrument_assets = clap_instrument_list.len(),
            "scanned assets directory"
        );

        Ok(Self { dir, fluid_list, sfizz_list, clap_instrument_list })
    }
}

fn extension_matches(extensions: &[&str], extension: &str) -> bool {
    extensions.iter().any(|candidate| extension.eq_ignore_ascii_case(candidate))
}

fn list_clap_instruments(path: &Path) -> Option<Vec<ClapInstrumentAsset>> {
    //TODO: use a separate process for safety?
    let entry = unsafe { clack_host::entry::PluginEntry::load(path).ok()? };
    let plugin_factory = entry.get_plugin_factory()?;

    Some(
        plugin_factory
            .plugin_descriptors()
            .filter(|d| d.features().any(|f| f == clack_host::plugin::features::INSTRUMENT))
            .filter_map(|descriptor| {
                let plugin_id = descriptor.id()?.to_str().ok()?.to_owned();
                let name = descriptor.name().and_then(|name| name.to_str().ok()).map(str::to_owned);
                let version = descriptor
                    .version()
                    .and_then(|version| version.to_str().ok())
                    .map(str::to_owned);
                Some(ClapInstrumentAsset { file_path: PathBuf::new(), plugin_id, name, version })
            })
            .collect(),
    )
}
