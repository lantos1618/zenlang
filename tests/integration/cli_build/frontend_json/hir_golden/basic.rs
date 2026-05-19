use super::assert_hir_source_golden;

#[test]
fn emit_json_hir_declaration_schema_matches_golden() {
    assert_hir_source_golden(
        r#"
Pair: {
    left: i32,
    right: i32,
}

MaybePair:
    None,
    Some(Pair)

threshold ::= 10

choose = (candidate: Pair, enabled: bool) MaybePair {
    enabled ?
        | true { MaybePair.Some(candidate) }
        | false { MaybePair.None }
}

main = () i32 { 0 }
"#,
        "hir_declarations_subject.zen",
        "tests/fixtures/ir_json/hir_declarations.golden.json",
        "declaration-rich program",
    );
}
