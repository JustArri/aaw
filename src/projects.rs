use crate::engine::{StoredProject, StoredProjectVersioned};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::fs;
use tracing::{info, warn};

#[derive(Deserialize)]
pub struct ProjectPreview {
    pub version: u32,
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StoredProjectError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum CachedProjectError {
    #[error("project with the same name already exists")]
    ProjectAlreadyExists,

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct Projects {
    pub dir: PathBuf,
    pub list: HashMap<uuid::Uuid, ProjectPreview>,
}

impl Projects {
    pub fn project_path(projects_dir: impl AsRef<Path>, id: uuid::Uuid) -> PathBuf {
        projects_dir.as_ref().join(format!("{id}.json"))
    }

    pub async fn new(dir: PathBuf) -> Result<Self, std::io::Error> {
        let mut entries = fs::read_dir(&dir).await?;
        let mut list = HashMap::new();

        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;

            if file_type.is_dir() {
                continue;
            }

            let file_path = entry.path();

            let Some(stem) = file_path.file_stem() else {
                warn!(path = %file_path.display(), "missing project file name");
                continue;
            };

            let Ok(id) = uuid::Uuid::try_parse_ascii(stem.as_encoded_bytes()) else {
                warn!(name = %stem.display(), "invalid project file name: expected uuid");
                continue;
            };

            let content = match fs::read(file_path).await {
                Ok(content) => content,
                Err(err) => {
                    warn!("failed to read project file: {err}");
                    continue;
                }
            };

            let content = String::from_utf8_lossy(&content);

            let preview = match serde_json::from_str::<ProjectPreview>(&content) {
                Ok(preview) => preview,
                Err(err) => {
                    warn!("invalid project file: {err}");
                    continue;
                }
            };

            list.insert(id, preview);
        }

        info!(projects = list.len(), "scanned projects directory");

        Ok(Self { dir, list })
    }

    pub async fn save_new(
        &mut self,
        project: StoredProject,
    ) -> Result<uuid::Uuid, CachedProjectError> {
        if self.list.iter().any(|p| p.1.name == project.name) {
            return Err(CachedProjectError::ProjectAlreadyExists);
        }

        let id = uuid::Uuid::new_v4();
        let path = self.dir.join(format!("{id}.json"));

        let content = crate::to_pretty_string(&project)?;
        fs::write(&path, content).await?;

        self.list.insert(
            id,
            ProjectPreview {
                version: StoredProjectVersioned::CURRENT_VERSION,
                name: project.name.clone(),
            },
        );

        Ok(id)
    }

    pub async fn save(
        &mut self,
        id: uuid::Uuid,
        project: StoredProject,
    ) -> Result<(), StoredProjectError> {
        let path = self.dir.join(format!("{id}.json"));

        let content = crate::to_pretty_string(&project)?;
        fs::write(&path, content).await?;

        self.list.insert(
            id,
            ProjectPreview {
                version: StoredProjectVersioned::CURRENT_VERSION,
                name: project.name.clone(),
            },
        );

        Ok(())
    }

    pub async fn load(&mut self, id: uuid::Uuid) -> Result<StoredProject, StoredProjectError> {
        let path = Self::project_path(&self.dir, id);
        let content = fs::read_to_string(path).await?;
        let project: StoredProject = serde_json::from_str(&content)?;

        self.list.insert(
            id,
            ProjectPreview {
                version: StoredProjectVersioned::CURRENT_VERSION,
                name: project.name.clone(),
            },
        );

        Ok(project)
    }

    pub async fn delete(&mut self, id: uuid::Uuid) -> Result<(), StoredProjectError> {
        let path = Self::project_path(&self.dir, id);
        fs::remove_file(path).await?;
        let _ = self.list.remove(&id);
        Ok(())
    }
}
