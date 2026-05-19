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
