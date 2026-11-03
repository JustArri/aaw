use crate::{engine::midi_app_controls::StoredMidiEngineControls, midi::InputAutoConnection};
use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::fs;
use tracing::{info, warn};
use utoipa::ToSchema;

#[derive(Debug, Clone, Migrations)]
#[migrations(version = 1, version_type = u32)]
pub struct StoredEngineMidiConfiguration {
    pub name: String,
    pub midi_app_controls: StoredMidiEngineControls,
    pub input_auto_connections: HashMap<String, InputAutoConnection>,
}

impl Default for StoredEngineMidiConfiguration {
    fn default() -> Self {
        Self {
            name: "MIDI configuration".into(),
            midi_app_controls: StoredMidiEngineControls::default(),
            input_auto_connections: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CtrEngineMidiConfigurationStorage {
    pub id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CtrEngineMidiConfiguration {
    pub name: String,

    #[schema(required)]
    pub storage: Option<CtrEngineMidiConfigurationStorage>,
}

impl Default for CtrEngineMidiConfiguration {
    fn default() -> Self {
        Self { name: StoredEngineMidiConfiguration::default().name, storage: None }
    }
}

#[derive(Deserialize)]
pub struct EngineMidiConfigurationPreview {
    pub version: u32,
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StoredEngineMidiConfigurationError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum CachedEngineMidiConfigurationError {
    #[error("engine MIDI configuration with the same name already exists")]
    ConfigurationAlreadyExists,

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct EngineMidiConfigurations {
    pub dir: PathBuf,
    pub list: HashMap<uuid::Uuid, EngineMidiConfigurationPreview>,
}

impl EngineMidiConfigurations {
    pub fn configuration_path(dir: impl AsRef<Path>, id: uuid::Uuid) -> PathBuf {
        dir.as_ref().join(format!("{id}.json"))
    }

    pub async fn new(dir: PathBuf) -> Result<Self, std::io::Error> {
        let mut entries = fs::read_dir(&dir).await?;
        let mut list = HashMap::new();

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                continue;
            }

            let file_path = entry.path();
            let Some(stem) = file_path.file_stem() else {
                warn!(path = %file_path.display(), "missing engine MIDI configuration file name");
                continue;
            };
            let Ok(id) = uuid::Uuid::try_parse_ascii(stem.as_encoded_bytes()) else {
                warn!(name = %stem.display(), "invalid engine MIDI configuration file name: expected uuid");
                continue;
            };
            let content = match fs::read(&file_path).await {
                Ok(content) => content,
                Err(err) => {
                    warn!(path = %file_path.display(), %err, "failed to read engine MIDI configuration file");
                    continue;
                }
            };
            let preview = match serde_json::from_slice::<EngineMidiConfigurationPreview>(&content) {
                Ok(preview) => preview,
                Err(err) => {
                    warn!(path = %file_path.display(), %err, "invalid engine MIDI configuration file");
                    continue;
                }
            };
            list.insert(id, preview);
        }

        info!(configurations = list.len(), "scanned engine MIDI configurations directory");
        Ok(Self { dir, list })
    }

    pub async fn save_new(
        &mut self,
        configuration: StoredEngineMidiConfiguration,
    ) -> Result<uuid::Uuid, CachedEngineMidiConfigurationError> {
        if self.list.values().any(|preview| preview.name == configuration.name) {
            return Err(CachedEngineMidiConfigurationError::ConfigurationAlreadyExists);
        }

        let id = uuid::Uuid::new_v4();
        self.save_file::<CachedEngineMidiConfigurationError>(id, configuration).await?;
        Ok(id)
    }

    pub async fn save(
        &mut self,
        id: uuid::Uuid,
        configuration: StoredEngineMidiConfiguration,
    ) -> Result<(), StoredEngineMidiConfigurationError> {
        self.save_file::<StoredEngineMidiConfigurationError>(id, configuration).await
    }

    async fn save_file<E>(
        &mut self,
        id: uuid::Uuid,
        configuration: StoredEngineMidiConfiguration,
    ) -> Result<(), E>
    where
        E: From<serde_json::Error> + From<std::io::Error>,
    {
        let content = crate::to_pretty_string(&configuration)?;
        fs::write(Self::configuration_path(&self.dir, id), content).await?;
        self.list.insert(
            id,
            EngineMidiConfigurationPreview {
                version: StoredEngineMidiConfigurationVersioned::CURRENT_VERSION,
                name: configuration.name,
            },
        );
        Ok(())
    }

    pub async fn load(
        &mut self,
        id: uuid::Uuid,
    ) -> Result<StoredEngineMidiConfiguration, StoredEngineMidiConfigurationError> {
        let content = fs::read_to_string(Self::configuration_path(&self.dir, id)).await?;
        let configuration: StoredEngineMidiConfiguration = serde_json::from_str(&content)?;
        self.list.insert(
            id,
            EngineMidiConfigurationPreview {
                version: StoredEngineMidiConfigurationVersioned::CURRENT_VERSION,
                name: configuration.name.clone(),
            },
        );
        Ok(configuration)
    }

    pub async fn read_file(&self, id: uuid::Uuid) -> Result<Vec<u8>, std::io::Error> {
        fs::read(Self::configuration_path(&self.dir, id)).await
    }

    pub async fn delete(
        &mut self,
        id: uuid::Uuid,
    ) -> Result<(), StoredEngineMidiConfigurationError> {
        fs::remove_file(Self::configuration_path(&self.dir, id)).await?;
        self.list.remove(&id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_round_trips_both_engine_midi_parts() {
        let mut configuration =
            StoredEngineMidiConfiguration { name: "Stage".into(), ..Default::default() };
        configuration.input_auto_connections.insert(
            "controller-port".into(),
            InputAutoConnection { preferred_slot_id: Some(3), slot_id_fixed: true },
        );

        let value = serde_json::to_value(&configuration).expect("serialize configuration");
        assert_eq!(value["version"], 1);
        assert!(value.get("midi_app_controls").is_some());
        assert!(value.get("input_auto_connections").is_some());

        let restored: StoredEngineMidiConfiguration =
            serde_json::from_value(value).expect("deserialize configuration");
        let connection = restored
            .input_auto_connections
            .get("controller-port")
            .expect("restore auto-connection");
        assert_eq!(restored.name, "Stage");
        assert_eq!(connection.preferred_slot_id, Some(3));
        assert!(connection.slot_id_fixed);
    }
}
