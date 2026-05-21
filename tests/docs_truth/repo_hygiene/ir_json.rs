use super::*;

#[test]
fn ast_and_symbols_json_graphs_live_in_focused_helpers() {
    let ir_json = read("src/ir_json.rs");
    let ast_graph = read("src/ir_json/ast_graph.rs");
    let symbols = read("src/ir_json/symbols.rs");

    for moved_schema_type in [
        "struct AstJsonGraph",
        "struct AstJsonModule",
        "struct AstJsonImport",
    ] {
        assert!(
            !ir_json.contains(moved_schema_type),
            "top-level IR JSON dispatch should not own AST graph schema definitions: {moved_schema_type}"
        );
        assert!(
            ast_graph.contains(moved_schema_type),
            "AST graph JSON schema definitions should live in the focused AST helper: {moved_schema_type}"
        );
    }

    for moved_schema_type in [
        "struct SymbolsJsonGraph",
        "struct SymbolsJsonModule",
        "struct SymbolJson",
    ] {
        assert!(
            !ir_json.contains(moved_schema_type),
            "top-level IR JSON dispatch should not own symbols schema definitions: {moved_schema_type}"
        );
        assert!(
            symbols.contains(moved_schema_type),
            "symbols JSON schema definitions should live in the focused symbols helper: {moved_schema_type}"
        );
    }

    assert!(
        ast_graph.contains("pub(super) fn ast_graph_to_json"),
        "AST graph JSON helper should own AST graph serialization entry point"
    );
    assert!(
        symbols.contains("pub(super) fn symbols_graph_to_json"),
        "symbols JSON helper should own symbols graph serialization entry point"
    );
    assert!(
        ir_json.contains("mod ast_graph;") && ir_json.contains("mod symbols;"),
        "IR JSON root should include focused AST and symbols helpers"
    );
    assert!(
        ir_json.lines().count() < 120,
        "ir_json.rs should stay focused on public JSON dispatch"
    );
}

#[test]
fn mir_json_schema_types_live_in_focused_helper() {
    let mir_lowering = read("src/ir_json/mir.rs");
    let mir_schema = read("src/ir_json/mir/schema.rs");
    let mir_expression = read("src/ir_json/mir/expression.rs");

    for moved_schema_type in [
        "struct MirJsonProgram",
        "struct MirFunction",
        "struct MirExpression",
        "struct MirPattern",
    ] {
        assert!(
            !mir_lowering.contains(moved_schema_type),
            "MIR JSON lowering should not own schema data type definitions: {moved_schema_type}"
        );
        assert!(
            mir_schema.contains(moved_schema_type),
            "MIR JSON schema type definitions should live in the focused schema helper: {moved_schema_type}"
        );
    }

    assert!(
        mir_schema.contains("use serde::Serialize"),
        "MIR JSON schema helper should own serialization derives"
    );
    assert!(
        mir_lowering.lines().count() < 220,
        "MIR JSON root lowering should stay focused on programs, functions, blocks, and patterns"
    );
    assert!(
        !mir_lowering.contains("fn mir_expression_kind"),
        "MIR expression kind lowering should live in expression.rs"
    );
    assert!(
        mir_expression.contains("pub(super) fn mir_expression"),
        "expression.rs should own MIR expression lowering"
    );
    assert!(
        mir_expression.contains("fn mir_expression_kind"),
        "expression.rs should own MIR expression kind classification"
    );
}

#[test]
fn diagnostics_json_schema_types_live_in_focused_helper() {
    let ir_json = read("src/ir_json.rs");
    let diagnostics_json = read("src/ir_json/diagnostics.rs");

    for moved_schema_type in [
        "struct DiagnosticsJson",
        "struct DiagnosticJson",
        "struct DiagnosticJsonSpan",
        "struct DiagnosticJsonSuggestedFix",
    ] {
        assert!(
            !ir_json.contains(moved_schema_type),
            "top-level IR JSON dispatch should not own diagnostics schema definitions: {moved_schema_type}"
        );
        assert!(
            diagnostics_json.contains(moved_schema_type),
            "diagnostics JSON schema definitions should live in the focused diagnostics helper: {moved_schema_type}"
        );
    }

    assert!(
        diagnostics_json.contains("pub fn diagnostics_to_json"),
        "diagnostics JSON helper should own diagnostics serialization entry point"
    );
}

#[test]
fn layout_json_schema_types_live_in_focused_helper() {
    let layout = read("src/ir_json/layout.rs");
    let schema = read("src/ir_json/layout/schema.rs");

    for moved_schema_type in [
        "struct LayoutJsonProgram",
        "struct LayoutJsonTarget",
        "struct LayoutJsonType",
        "struct LayoutJsonField",
        "struct LayoutJsonVariant",
    ] {
        assert!(
            !layout.contains(moved_schema_type),
            "layout JSON lowering should not own schema type definitions: {moved_schema_type}"
        );
        assert!(
            schema.contains(moved_schema_type),
            "layout JSON schema definitions should live in the focused schema helper: {moved_schema_type}"
        );
    }

    assert!(
        schema.contains("use serde::Serialize"),
        "layout JSON schema helper should own serialization derives"
    );
    assert!(
        layout.lines().count() < 240,
        "layout JSON lowering should stay focused on layout calculation"
    );
}
