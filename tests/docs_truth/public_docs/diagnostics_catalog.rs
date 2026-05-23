use super::super::*;

#[test]
fn diagnostics_catalog_documents_json_stable_codes() {
    let catalog = read("docs/DIAGNOSTICS.md");

    for required in [
        "# Zen Diagnostics Catalog",
        "JSON-Stable Codes",
        "tests/fixtures/ir_json/diagnostics_*.golden.json",
        "Representative JSON Evidence",
        "representative anchors, not an exhaustive fixture inventory",
        "E2000",
        "removed syntax or reserved syntax",
        "replace_removed_return_with_final_expression",
        "feature_gate",
        "E3500",
        "Sync/Async effect modes",
        "allocator-backed dynamic `String`",
        "std actor framework types",
        "std actor framework types/imports",
        "std allocator imports",
        "std Sync/Async runtime imports",
        "E0203",
        "gated compiler-owned intrinsic call",
        "comptime type matching",
        "reserved async scheduler intrinsics",
        "atomic intrinsics",
        "raw syscalls",
        "raw allocation intrinsics",
        "byte-memory intrinsics",
        "raw pointer intrinsics",
        "E3053",
        "gated range expression",
        "E3054",
        "gated Result propagation",
        "E3055",
        "gated task waiting",
        "E4006",
        "non-exhaustive bool match diagnostics",
        "add_missing_bool_match_arm",
        "resolver validation failure",
        "gated reserved type surfaces",
        "E5000",
        "generic inference conflict",
        "conflicting inferred type arguments for generic functions and methods",
        "E5001",
        "generic type-argument arity",
        "behavior references",
        "E5002",
        "type arguments were supplied to a non-generic",
        "E6004",
        "generic behavior-bound failure",
        "E6007",
        "explicit type association `.requires` failure",
        "E6010",
        "behavior implementation coherence failure",
        "diagnostics_return",
        "diagnostics_type_match_gate",
        "diagnostics_async_intrinsic_gate",
        "diagnostics_atomic_gate",
        "diagnostics_syscall_gate",
        "diagnostics_raw_allocate_gate",
        "diagnostics_byte_memory_gate",
        "diagnostics_raw_pointer_gate",
        "tests/fixtures/ir_json/diagnostics_range_gate.golden.json",
        "tests/fixtures/ir_json/diagnostics_raise_gate.golden.json",
        "tests/fixtures/ir_json/diagnostics_await_gate.golden.json",
        "tests/fixtures/ir_json/diagnostics_missing_bool_match_arm.golden.json",
        "duplicate generic association fixtures",
        "typed allocator/effect gates",
        "dynamic string gates",
        "actor/import/runtime gates",
        "generic function/method annotation fixtures",
        "nested generic fixtures",
        "struct/enum constructor fixtures",
        "behavior requires/impl/extends fixtures",
        "nongeneric annotation, constructor, function, builtin, module function, method, and `tests/fixtures/ir_json/diagnostics_nongeneric_requires_type_args.golden.json`",
        "tests/fixtures/ir_json/diagnostics_generic_result_method_bound.golden.json",
        "tests/fixtures/ir_json/diagnostics_generic_function_bound.golden.json",
        "tests/fixtures/ir_json/diagnostics_generic_requires_missing_impl.golden.json",
        "tests/fixtures/ir_json/diagnostics_generic_result_method_inference.golden.json",
        "tests/fixtures/ir_json/diagnostics_generic_function_inference.golden.json",
        "tests/fixtures/ir_json/diagnostics_generic_behavior_overlap.golden.json",
    ] {
        assert!(
            catalog.contains(required),
            "docs/DIAGNOSTICS.md is missing diagnostic catalog text: {required}"
        );
    }

    assert!(
        catalog.lines().count() <= 42,
        "diagnostics catalog should stay compact; keep exhaustive fixture inventories in tests and fixtures"
    );

    assert!(
        !catalog.contains("diagnostics_generic_method_type_arg_annotation_arity.golden.json"),
        "diagnostics catalog should not enumerate every arity fixture"
    );
}
