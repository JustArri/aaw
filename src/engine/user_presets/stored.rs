use crate::engine::{
    graph::ControllerAddress,
    user_presets::{CtrUserPresets, UserPreset, UserPresetAction},
};
use migrations_derive::Migrations;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoredUserPresetV1 {
    pub name: String,
    pub controllers: Vec<ControllerAddress>,
}

#[derive(Deserialize)]
pub struct StoredUserPresetsV1 {
    pub entries: Vec<StoredUserPresetV1>,
}

#[derive(Default, Debug, Clone, Migrations)]
#[migrations(version = 2, version_type = u32)]
pub struct StoredUserPresets {
    pub entries: Vec<UserPreset>,
}

impl From<StoredUserPresetsV1> for StoredUserPresetsV2 {
    fn from(value: StoredUserPresetsV1) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|p| {
                    let mut actions = vec![UserPresetAction::DisableAllInstrumentControllers];

                    for controller in p.controllers {
                        actions.push(UserPresetAction::SetInstrumentControllerEnabled {
                            controller,
                            flag: true,
                        });
                    }

                    UserPreset { name: p.name, actions }
                })
                .collect(),
        }
    }
}

impl From<StoredUserPresetsV2> for StoredUserPresets {
    fn from(value: StoredUserPresetsV2) -> Self {
        Self { entries: value.entries }
    }
}

impl From<&CtrUserPresets> for StoredUserPresets {
    fn from(value: &CtrUserPresets) -> Self {
        Self { entries: value.entries.clone() }
    }
}

impl From<CtrUserPresets> for StoredUserPresets {
    fn from(value: CtrUserPresets) -> Self {
        Self { entries: value.entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::graph::InstrumentId;

    #[test]
    fn current_version_round_trips_clap_parameter_actions() {
        let stored = StoredUserPresets {
            entries: vec![UserPreset {
                name: "CLAP preset".into(),
                actions: vec![UserPresetAction::SetClapParameter {
                    instrument: InstrumentId(3),
                    parameter_id: 17,
                    value: 0.75,
                }],
            }],
        };

        let value = serde_json::to_value(stored).expect("serialize user presets");
        assert_eq!(value["version"], 2);

        let versioned: StoredUserPresetsVersioned =
            serde_json::from_value(value).expect("deserialize user presets");
        let restored = versioned.migrate();
        let UserPresetAction::SetClapParameter { instrument, parameter_id, value } =
            &restored.entries[0].actions[0]
        else {
            panic!("expected CLAP parameter action");
        };
        assert_eq!(*instrument, InstrumentId(3));
        assert_eq!(*parameter_id, 17);
        assert_eq!(*value, 0.75);
    }
}
