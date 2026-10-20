use std::path::PathBuf;
use tokio::fs;
use tracing::info;

pub const FLUID_EXTENSIONS: &[&str] = &["sf2", "sf3"];
pub const SFIZZ_EXTENSIONS: &[&str] = &["sfz"];

pub struct Assets {
    pub dir: PathBuf,
    pub fluid_list: Vec<PathBuf>,
    pub sfizz_list: Vec<PathBuf>,
}

impl Assets {
    pub async fn new(dir: PathBuf) -> Result<Self, std::io::Error> {
        let mut fluid_list = Vec::new();
        let mut sfizz_list = Vec::new();
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

                if FLUID_EXTENSIONS
                    .iter()
                    .any(|candidate| extension.eq_ignore_ascii_case(candidate))
                {
                    fluid_list.push(relative_path.to_owned());
                }

                if SFIZZ_EXTENSIONS
                    .iter()
                    .any(|candidate| extension.eq_ignore_ascii_case(candidate))
                {
                    sfizz_list.push(relative_path.to_owned());
                }
            }
        }

        fluid_list.sort_unstable();
        sfizz_list.sort_unstable();

        info!(
            fluid_assets = fluid_list.len(),
            sfizz_assets = sfizz_list.len(),
            "scanned assets directory"
        );

        Ok(Self { dir, fluid_list, sfizz_list })
    }
}
