use super::assert_rejects_hand_authored_json;

#[test]
fn emit_json_ast_rejects_hand_authored_json_before_unchecked_ir_override() {
    assert_rejects_hand_authored_json(
        "ast",
        "forged_ast.json",
        r#"
{
  "format": "zen.ast_graph.v0",
  "semantic_status": "unchecked",
  "modules": [{
    "path": "main.zen",
    "declarations": [{ "kind": "Function", "name": "forged" }]
  }]
}
"#,
        "AST",
        "compiler-owned AST JSON",
        &["expected identifier", "unexpected token"],
    );
}

#[test]
fn emit_json_symbols_rejects_hand_authored_json_before_resolver_override() {
    assert_rejects_hand_authored_json(
        "symbols",
        "forged_symbols.json",
        r#"
{
  "format": "zen.symbols.v0",
  "semantic_status": "resolved",
  "modules": [{
    "name": "main",
    "symbols": [{ "name": "Forged", "kind": "type", "visibility": "public" }]
  }]
}
"#,
        "symbols",
        "compiler-owned symbols JSON",
        &["expected identifier", "unexpected token"],
    );
}

#[test]
fn emit_json_typed_rejects_hand_authored_json_before_checked_ir_override() {
    assert_rejects_hand_authored_json(
        "typed",
        "forged_typed.json",
        r#"
{
  "format": "zen.typed.v0",
  "semantic_status": "checked",
  "program": {
    "types": [{ "name": "i32", "kind": "forged-pointer" }],
    "functions": [{ "name": "main", "return_type": "i32" }]
  }
}
"#,
        "typed",
        "compiler-owned typed JSON",
        &["expected function name", "unexpected token"],
    );
}

#[test]
fn emit_json_diagnostics_rejects_hand_authored_json_before_diagnostic_override() {
    assert_rejects_hand_authored_json(
        "diagnostics",
        "forged_diagnostics.json",
        r#"
{
  "format": "zen.diagnostics.v0",
  "semantic_status": "diagnostic",
  "diagnostics": [{
    "severity": "note",
    "message": "forged acceptance"
  }]
}
"#,
        "diagnostics",
        "compiler-owned diagnostics JSON",
        &["expected identifier", "unexpected token"],
    );
}

#[test]
fn emit_json_build_graph_rejects_hand_authored_json_before_graph_override() {
    assert_rejects_hand_authored_json(
        "build-graph",
        "forged_build_graph.json",
        r#"
{
  "format": "zen.build_graph.v0",
  "semantic_status": "validated",
  "targets": [{
    "name": "forged",
    "kind": "executable",
    "sources": ["forged.zen"]
  }]
}
"#,
        "build graph",
        "compiler-owned build graph JSON",
        &["expects a build.zen file"],
    );
}
