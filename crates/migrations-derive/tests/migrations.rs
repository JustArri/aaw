use migrations_derive::Migrations;
use serde::Deserialize;

#[derive(Deserialize)]
struct ExampleV1 {
    original_name: String,
}

#[derive(Default, Migrations)]
#[migrations(version = 2, version_type = u32)]
struct Example {
    renamed: String,
    introduced: u8,
}

impl From<ExampleV1> for ExampleV2 {
    fn from(value: ExampleV1) -> Self {
        Self { renamed: value.original_name, introduced: 42 }
    }
}

#[test]
fn runs_manual_migration_between_versions() {
    let versioned: ExampleVersioned =
        serde_json::from_str(r#"{"version":1,"original_name":"value"}"#).expect("deserialize v1");

    assert_eq!(versioned.version(), Some(1));
    assert_eq!(versioned.next_version(), Some(2));

    let versioned = versioned.migrate_to_next();
    assert_eq!(versioned.version(), Some(2));

    let current = versioned.migrate();
    assert_eq!(current.renamed, "value");
    assert_eq!(current.introduced, 42);
}

#[test]
fn serializes_and_deserializes_current_version() {
    let value = serde_json::to_value(Example { renamed: "value".into(), introduced: 7 })
        .expect("serialize current");

    assert_eq!(value["version"], 2);
    assert_eq!(value["introduced"], 7);

    let current: Example = serde_json::from_value(value).expect("deserialize current");
    assert_eq!(current.renamed, "value");
    assert_eq!(current.introduced, 7);
}

#[test]
fn derives_current_schema() {
    assert_eq!(<Example as utoipa::ToSchema>::name(), "Example");
}
