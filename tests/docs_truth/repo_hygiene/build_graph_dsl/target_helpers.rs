use super::*;

#[test]
fn build_target_field_spelling_lives_in_focused_helper() {
    let dsl = read("src/build_graph/lowering/dsl.rs");
    let target_fields = read("src/build_graph/lowering/dsl/target_fields.rs");

    assert!(
        !dsl.contains("enum BuildTargetField"),
        "build graph DSL root should not own target field enum spelling"
    );
    for required in [
        "pub(in crate::build_graph) enum BuildTargetField",
        "const ALL: &[BuildTargetField]",
        ".find(|field| field.as_str() == value)",
        "impl fmt::Display for BuildTargetField",
        "impl FromStr for BuildTargetField",
    ] {
        assert!(
            target_fields.contains(required),
            "target field spelling should live in focused helper: {required}"
        );
    }
    assert!(
        dsl.contains("mod target_fields;")
            && dsl.contains("pub(super) use target_fields::BuildTargetField;"),
        "build graph DSL root should load and re-export target field spelling"
    );
}

#[test]
fn build_target_field_extraction_lives_in_focused_helper() {
    let targets = read("src/build_graph/lowering/targets.rs");
    let fields = read("src/build_graph/lowering/target_fields.rs");

    assert!(
        targets.lines().count() < 220,
        "build target construction should stay focused on target shapes"
    );
    for helper in [
        "required_string_field",
        "required_one_of_string_fields",
        "optional_string_field",
        "required_string_array_field",
        "optional_string_array_field",
        "field_value",
    ] {
        assert!(
            !targets.contains(&format!("fn {helper}")),
            "build target field extraction should live in target_fields.rs: {helper}"
        );
        assert!(
            fields.contains(&format!("fn {helper}")),
            "target_fields.rs should own build target field extraction: {helper}"
        );
    }
}

#[test]
fn build_target_field_validation_lives_in_focused_helper() {
    let lowering = read("src/build_graph/lowering.rs");
    let targets = read("src/build_graph/lowering/targets.rs");
    let validation = read("src/build_graph/lowering/target_validation.rs");

    assert!(
        targets.lines().count() < 180,
        "build target construction should stay focused on target shapes"
    );
    for helper in ["validate_target_fields", "allowed_fields"] {
        assert!(
            !targets.contains(&format!("fn {helper}")),
            "build target field validation should live in target_validation.rs: {helper}"
        );
        assert!(
            validation.contains(&format!("fn {helper}")),
            "target_validation.rs should own build target field validation: {helper}"
        );
    }
    assert!(
        lowering.contains("mod target_validation;"),
        "build graph lowering should include the focused target-validation helper"
    );
}
