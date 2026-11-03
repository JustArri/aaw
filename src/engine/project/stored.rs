use crate::engine::{
    drum_machine::MusicalTransport, instrument_graph::StoredInstrumentGraph,
    midi_app_controls::StoredMidiProjectControls, project::CtrProject,
    user_presets::StoredUserPresets,
};
use migrations_derive::Migrations;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StoredProjectV1 {
    pub revision: u32,
    pub name: String,
    pub instrument_graph: StoredInstrumentGraph,
    pub user_presets: StoredUserPresets,
    pub transport: MusicalTransport,
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 2, version_type = u32)]
pub struct StoredProject {
    pub revision: u32,
    pub name: String,
    pub instrument_graph: StoredInstrumentGraph,
    pub user_presets: StoredUserPresets,
    pub midi_app_controls: StoredMidiProjectControls,
    pub transport: MusicalTransport,
}

impl From<StoredProjectV1> for StoredProjectV2 {
    fn from(value: StoredProjectV1) -> Self {
        Self {
            revision: value.revision,
            name: value.name,
            instrument_graph: value.instrument_graph,
            user_presets: value.user_presets,
            midi_app_controls: StoredMidiProjectControls::default(),
            transport: value.transport,
        }
    }
}

impl From<&CtrProject> for StoredProject {
    fn from(value: &CtrProject) -> Self {
        Self {
            revision: value.revision,
            name: value.name.clone(),
            instrument_graph: (&value.instrument_graph).into(),
            user_presets: (&value.user_presets).into(),
            midi_app_controls: value.midi_app_controls.clone().into(),
            transport: value.transport,
        }
    }
}

impl From<CtrProject> for StoredProject {
    fn from(value: CtrProject) -> Self {
        Self {
            revision: value.revision,
            name: value.name,
            instrument_graph: value.instrument_graph.into(),
            user_presets: value.user_presets.into(),
            midi_app_controls: value.midi_app_controls.into(),
            transport: value.transport,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_v1_with_v2_defaults() {
        let mut value = serde_json::to_value(StoredProject::default()).expect("serialize project");
        let object = value.as_object_mut().expect("project should be an object");
        object.insert("version".into(), 1.into());
        object.remove("midi_app_controls");

        let versioned: StoredProjectVersioned =
            serde_json::from_value(value).expect("deserialize v1 project");
        assert_eq!(versioned.version(), Some(1));
        assert_eq!(versioned.next_version(), Some(2));

        let versioned = versioned.migrate_to_next();
        assert_eq!(versioned.version(), Some(2));
        assert_eq!(versioned.next_version(), None);

        let project = versioned.migrate();
        assert!(project.midi_app_controls.controls.is_empty());
    }

    #[test]
    fn serializes_current_project_as_v2() {
        let value = serde_json::to_value(StoredProject::default()).expect("serialize project");

        assert_eq!(value["version"], 2);
        assert!(value.get("midi_app_controls").is_some());
    }
}
