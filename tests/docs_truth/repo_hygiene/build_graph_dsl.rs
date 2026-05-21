use super::*;

mod target_helpers;

#[test]
fn build_graph_dsl_parsing_uses_enum_static_tables() {
    let dsl_root = read("src/build_graph/lowering/dsl.rs");
    let target_fields = read("src/build_graph/lowering/dsl/target_fields.rs");
    let dsl = format!("{dsl_root}\n{target_fields}");

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
fn build_graph_error_types_live_in_focused_helper() {
    let root = read("src/build_graph.rs");
    let errors = read("src/build_graph/errors.rs");

    for helper in [
        "enum BuildGraphError",
        "impl fmt::Display for BuildGraphError",
        "impl std::error::Error for BuildGraphError",
    ] {
        assert!(
            !root.contains(helper),
            "build graph root should not own error type/formatting helper: {helper}"
        );
        assert!(
            errors.contains(helper),
            "build graph error helper should live in errors.rs: {helper}"
        );
    }

    assert!(
        root.lines().count() < 220,
        "build_graph.rs should stay focused on graph shapes and construction"
    );
    assert!(
        root.contains("mod errors;") && root.contains("pub use errors::BuildGraphError;"),
        "build graph root should include and re-export the focused error helper"
    );
}

#[test]
fn build_graph_json_output_lives_in_focused_helper() {
    let root = read("src/build_graph.rs");
    let json = read("src/build_graph/json.rs");

    for helper in ["struct BuildGraphJson", "fn canonical_json"] {
        assert!(
            !root.contains(helper),
            "build graph root should not own JSON output helper: {helper}"
        );
        assert!(
            json.contains(helper),
            "build graph JSON output should live in focused helper: {helper}"
        );
    }
    assert!(
        root.lines().count() < 200,
        "build_graph.rs should stay focused on graph shapes and construction"
    );
    assert!(
        root.contains("mod json;"),
        "build graph root should include focused JSON output helper"
    );
}

#[test]
fn build_graph_host_effect_detection_lives_in_focused_helper() {
    let lowering = read("src/build_graph/lowering.rs");
    let traversal = read("src/build_graph/lowering/traversal.rs");
    let host_effects = read("src/build_graph/lowering/host_effects.rs");
    let lowering_source = format!("{lowering}\n{traversal}");

    assert!(
        lowering.lines().count() < 110,
        "build graph lowering should stay focused on build-function discovery and helper wiring"
    );
    for helper in [
        "declared_host_effect",
        "host_effect_arm_declares_fallback",
        "host_effect",
        "is_builder_os",
    ] {
        assert!(
            !lowering_source.contains(&format!("fn {helper}")),
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
        lowering.lines().count() < 110,
        "build graph lowering should stay focused on build-function discovery and helper wiring"
    );
    assert!(
        lowering.contains("mod traversal;"),
        "build graph lowering should include the focused AST traversal helper"
    );
    for helper in [
        "struct BuildProgramLowering",
        "enum BuildTargetAddContext",
        "fn collect_expr",
    ] {
        assert!(
            !lowering.contains(helper),
            "build graph lowering root should not own traversal helper: {helper}"
        );
        assert!(
            traversal.contains(helper),
            "build graph AST traversal helper should own: {helper}"
        );
    }
    assert!(
        traversal.contains("build_target_from_builder_add"),
        "build graph AST traversal should own target collection from builder.add calls"
    );
}

#[test]
fn build_graph_traversal_statement_and_builder_helpers_live_in_focused_modules() {
    let traversal = read("src/build_graph/lowering/traversal.rs");
    let statements = read("src/build_graph/lowering/traversal/statements.rs");
    let builder_calls = read("src/build_graph/lowering/traversal/builder_calls.rs");

    assert!(
        traversal.lines().count() < 190,
        "build graph traversal root should stay focused on expression traversal"
    );
    assert!(
        !traversal.contains("fn collect_statement"),
        "statement traversal should not live in traversal.rs"
    );
    assert!(
        statements.contains("fn collect_statement"),
        "statement traversal should live in traversal/statements.rs"
    );
    assert!(
        !traversal.contains("fn is_builder_add_call"),
        "builder.add detection should not live in traversal.rs"
    );
    assert!(
        builder_calls.contains("fn is_builder_add_call")
            && builder_calls.contains("BuildTargetDslIdent"),
        "builder.add detection should live in traversal/builder_calls.rs and use owned DSL identifiers"
    );
    for module_name in ["statements", "builder_calls"] {
        assert!(
            traversal.contains(&format!("mod {module_name};")),
            "build graph traversal should include focused helper: {module_name}"
        );
    }
}
