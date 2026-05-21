use super::*;

#[test]
fn target_yaml_public_error_lives_in_focused_helper() {
    let target_yaml = read("src/target_yaml.rs");
    let error = read("src/target_yaml/error.rs");
    let validation = read("src/target_yaml/validation.rs");

    for helper in [
        "pub enum TargetYamlError",
        "impl std::fmt::Display for TargetYamlError",
        "impl std::error::Error for TargetYamlError",
        "impl From<std::io::Error> for TargetYamlError",
        "impl From<serde_yaml::Error> for TargetYamlError",
        "impl From<serde_json::Error> for TargetYamlError",
    ] {
        assert!(
            !target_yaml.contains(helper),
            "target_yaml.rs should not own public error plumbing: {helper}"
        );
        assert!(
            error.contains(helper),
            "target YAML error plumbing should live in focused helper: {helper}"
        );
    }

    assert!(
        !target_yaml.contains("fn validate_target_yaml"),
        "target_yaml.rs should not own schema validation"
    );
    assert!(
        validation.contains("pub(super) fn validate_target_yaml"),
        "target YAML schema validation should live in focused helper"
    );
    assert!(
        target_yaml.contains("mod validation;"),
        "target_yaml.rs should include the focused validation helper"
    );
    assert!(
        target_yaml.contains("mod error;"),
        "target_yaml.rs should include the focused error helper"
    );
    assert!(
        target_yaml.contains("pub use error::TargetYamlError;"),
        "target_yaml.rs should re-export the public target YAML error type"
    );
    assert!(
        target_yaml.lines().count() < 140,
        "target_yaml.rs should stay focused on target YAML schema validation and JSON emission"
    );
}
