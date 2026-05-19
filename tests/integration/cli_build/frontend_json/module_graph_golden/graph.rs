use super::normalized_fixture;
use super::normalized_module_graph_json;

#[test]
fn emit_json_ast_module_graph_schema_matches_golden() {
    let actual = normalized_module_graph_json("ast");
    let expected = normalized_fixture("tests/fixtures/ir_json/ast_module_graph.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_module_graph_schema_matches_golden() {
    let actual = normalized_module_graph_json("symbols");
    let expected = normalized_fixture("tests/fixtures/ir_json/symbols_module_graph.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}
