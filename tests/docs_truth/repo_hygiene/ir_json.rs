use super::*;

#[test]
fn mir_json_schema_types_live_in_focused_helper() {
    let mir_lowering = read("src/ir_json/mir.rs");
    let mir_schema = read("src/ir_json/mir/schema.rs");

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
