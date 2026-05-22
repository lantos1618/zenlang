use super::*;

#[test]
fn build_graph_dsl_parsing_uses_enum_static_tables() {
    let dsl = read("src/build_graph/lowering/dsl.rs");

    for forbidden in [
        r#"match value {
            Self::NAME => Ok(Self::Name),"#,
        r#"Self::BUILDER => Ok(Self::Builder)"#,
        r#"Self::EXECUTABLE => Ok(Self::Executable)"#,
        r#"Self::OK => Ok(Self::Ok)"#,
    ] {
        assert!(
            !dsl.contains(forbidden),
            "build graph DSL parsing should use enum-owned static tables, not raw FromStr match arms: {forbidden}"
        );
    }

    for required in [
        "const ALL: &[BuildTargetField]",
        "const ALL: &[BuildTargetDslIdent]",
        "const ALL: &[BuildTargetDslKind]",
        "const ALL: &[HostEffectResultVariant]",
        ".find(|field| field.as_str() == value)",
        ".find(|ident| ident.as_str() == value)",
        ".find(|kind| kind.as_str() == value)",
        ".find(|variant| variant.as_str() == value)",
    ] {
        assert!(
            dsl.contains(required),
            "build graph DSL spelling should parse through enum static tables: {required}"
        );
    }
}

#[test]
fn build_graph_dependency_order_lives_in_focused_helper() {
    let root = read("src/build_graph.rs");
    let dependency_order = read("src/build_graph/dependency_order.rs");

    for helper in ["targets_in_dependency_order", "visit_target"] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "build graph root should not own dependency ordering helper: {helper}"
        );
        assert!(
            dependency_order.contains(&format!("fn {helper}")),
            "dependency ordering should live in focused build graph helper: {helper}"
        );
    }

    assert!(
        root.contains("mod dependency_order;"),
        "build graph root should include focused dependency ordering module"
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
fn build_graph_host_effect_detection_lives_in_focused_helper() {
    let lowering = read("src/build_graph/lowering.rs");
    let host_effects = read("src/build_graph/lowering/host_effects.rs");
    let traversal = read("src/build_graph/lowering/traversal.rs");

    assert!(
        lowering.lines().count() < 240,
        "build graph lowering should stay focused on traversal and target collection"
    );
    for helper in [
        "declared_host_effect",
        "host_effect_arm_declares_fallback",
        "host_effect",
        "is_builder_os",
    ] {
        assert!(
            !lowering.contains(&format!("fn {helper}")),
            "build graph host-effect detection should live in host_effects.rs: {helper}"
        );
        assert!(
            host_effects.contains(&format!("fn {helper}")),
            "host_effects.rs should own build graph host-effect detection: {helper}"
        );
    }
    assert!(
        lowering.contains("mod host_effects;"),
        "build graph lowering should include the focused host-effect helper"
    );
    assert!(
        traversal.contains("use super::host_effects::{declared_host_effect, host_effect};"),
        "build graph traversal should import host-effect detection from the focused helper"
    );
}

#[test]
fn build_graph_ast_traversal_lives_in_focused_helper() {
    let lowering = read("src/build_graph/lowering.rs");
    let traversal = read("src/build_graph/lowering/traversal.rs");

    assert!(
        lowering.lines().count() < 140,
        "build graph lowering root should stay focused on locating build() and delegating traversal"
    );
    for helper in [
        "struct BuildProgramLowering",
        "enum BuildTargetAddContext",
        "fn into_input",
        "fn collect_expr",
        "fn collect_statement",
        "fn is_builder_add_call",
    ] {
        assert!(
            !lowering.contains(helper),
            "build graph AST traversal should live in traversal.rs: {helper}"
        );
        assert!(
            traversal.contains(helper),
            "traversal.rs should own build graph AST traversal: {helper}"
        );
    }
    assert!(
        lowering.contains("mod traversal;"),
        "build graph lowering root should include the focused traversal module"
    );
    assert!(
        lowering.contains("use traversal::BuildProgramLowering;"),
        "build graph lowering root should delegate AST traversal through BuildProgramLowering"
    );
}
