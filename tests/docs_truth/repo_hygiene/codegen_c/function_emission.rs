use super::*;

#[test]
fn codegen_c_function_emission_lives_in_focused_helper() {
    let types = read("src/codegen/c/types.rs");
    let functions = read("src/codegen/c/functions.rs");
    let type_defs = read("src/codegen/c/types/type_defs.rs");
    let c_mod = read("src/codegen/c/mod.rs");

    for helper in [
        "emit_function_forward_decl",
        "emit_function",
        "format_params",
        "emit_global",
    ] {
        assert!(
            !types.contains(&format!("fn {helper}")),
            "C type emission should not own function/global helper: {helper}"
        );
        assert!(
            functions.contains(&format!("fn {helper}")),
            "C function/global emission should live in focused helper: {helper}"
        );
    }

    assert!(
        c_mod.contains("mod functions;"),
        "C codegen should load focused function/global emission helper"
    );
    assert!(
        types.contains("mod type_defs;"),
        "C type emission should load focused struct/enum definition helper"
    );
    assert!(
        !types.contains("fn emit_type_def"),
        "C type mapping/program emission should not own struct/enum definition helper"
    );
    assert!(
        type_defs.contains("fn emit_type_def"),
        "C struct/enum definition emission should live in focused helper"
    );
    assert!(
        types.lines().count() < 150,
        "types.rs should stay focused on C type mapping and program emission order"
    );
}
