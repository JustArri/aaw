use anyhow::Context;
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;
use tracing::{error, info};

//TODO: implement Error type instead of anyhow's one?

pub async fn save<T: Serialize>(root: &Path, (path, subject): (impl AsRef<Path>, &str), value: &T) {
    async fn inner<T: Serialize>(full_path: &Path, subject: &str, value: &T) -> anyhow::Result<()> {
        info!(path = %full_path.display(), "saving {subject}...");
        if let Some(parent) = full_path.parent()
            && !parent.exists()
        {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("failed to create save directory '{}'", parent.display())
            })?;
        }
        let value = crate::to_pretty_string(value)
            .with_context(|| format!("failed to save file '{}'", full_path.display()))?;

        tokio::fs::write(full_path, value)
            .await
            .with_context(|| format!("failed to save file '{}'", full_path.display()))?;

        Ok(())
    }

    let full_path = root.join(&path);

    if inner(&full_path, subject, value).await.is_err() {
        error!(path = %full_path.display(), "failed to save {subject}");
    }
}

pub async fn load_opt<T: DeserializeOwned>(
    root: impl AsRef<Path>,
    (path, subject): (impl AsRef<Path>, &str),
) -> Option<T> {
    let full_path = root.as_ref().join(&path);

    info!(path = %full_path.display(), "loading {subject}...");
    let content = tokio::fs::read(&full_path).await.ok()?;
    let content = String::from_utf8_lossy(&content);

    match serde_json::from_str(&content) {
        Ok(value) => Some(value),
        Err(err) => {
            error!(path = %full_path.display(), %err, "failed to load {subject}");
            None
        }
    }
}
