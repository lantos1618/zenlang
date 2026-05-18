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
