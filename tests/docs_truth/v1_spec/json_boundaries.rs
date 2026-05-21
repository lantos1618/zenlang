use super::super::*;

#[test]
fn v1_spec_records_json_boundaries_and_golden_coverage() {
    let spec = read("docs/V1_SPEC.md");

    for required in [
        "zen emit-json ast <file>",
        "zen emit-json symbols <file>",
        "zen emit-json typed <file>",
        "zen emit-json diagnostics <file>",
        "zen emit-json hir <file>",
        "zen emit-json mir <file>",
        "zen emit-json layout <file>",
        "zen emit-json build-graph <file>",
        "zen emit-json target-yaml <file>",
        "semantic acceptance must use typed",
        "Evidence is category-level here",
        "tests/fixtures/ir_json",
        "exhaustive fixture names live",
        "emit_json_ast_rejects_hand_authored_json_before_unchecked_ir_override",
        "emit_json_symbols_rejects_hand_authored_json_before_resolver_override",
        "emit_json_typed_rejects_hand_authored_json_before_checked_ir_override",
        "emit_json_build_graph_rejects_hand_authored_json_before_graph_override",
        "hand-authored IR rejection is pinned for AST, symbols, typed, diagnostics, HIR, MIR, layout, and build graph JSON",
        "compiler-owned generic JSON is pinned across AST, symbols, typed, HIR, MIR, layout, build graph, and target YAML",
        "emit_json_ast_module_graph_schema_matches_golden",
        "emit_json_symbols_generic_method_schema_matches_golden",
        "Box.get<T>",
        "Box<T>.impl",
        "Option<T>",
        "Result<T, E>",
        "self: Self",
        "Json<StaticString>",
        "Point.encode__Json_Point",
        "Json<Point>",
        "emit_json_diagnostics_removed_return_schema_matches_golden",
        "emit_json_diagnostics_behavior_derive_gate_schema_matches_golden",
        "emit_json_diagnostics_typed_allocator_effect_gate_schema_matches_golden",
        "emit_json_diagnostics_generic_function_arity_schema_matches_golden",
        "emit_json_typed_generic_method_schema_matches_golden",
        "schema_version: 0",
        "emit_json_hir_generic_method_worklist_schema_matches_golden",
        "emit_json_mir_generic_method_worklist_schema_matches_golden",
        "emit_json_layout_nested_generic_result_schema_matches_golden",
        "emit_json_target_yaml_backend_schema_matches_golden",
        "emit_json_build_graph_project_schema_matches_golden",
    ] {
        assert!(
            spec.contains(required),
            "docs/V1_SPEC.md is missing JSON boundary or golden coverage: {required}"
        );
    }
}
