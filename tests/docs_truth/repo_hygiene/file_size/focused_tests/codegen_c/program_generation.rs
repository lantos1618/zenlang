use super::*;

#[test]
fn program_generation_tests_stay_split_by_surface() {
    let root = read("src/codegen/c/tests/program_generation.rs");
    let aggregates = read("src/codegen/c/tests/program_generation/aggregates.rs");
    let functions_and_runtime =
        read("src/codegen/c/tests/program_generation/functions_and_runtime.rs");

    assert!(
        root.lines().count() < 90,
        "program_generation.rs should only route focused program-generation test modules and shared helpers"
    );
    for module in ["mod aggregates;", "mod functions_and_runtime;"] {
        assert!(
            root.contains(module),
            "program_generation.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn generates_struct") && !root.contains("fn generates_function"),
        "program generation root should not own concrete generation tests"
    );
    assert!(
        aggregates.contains("fn generates_struct")
            && aggregates.contains("fn generates_enum")
            && aggregates.contains("fn generates_enum_with_payload"),
        "aggregates.rs should cover struct and enum generation"
    );
    assert!(
        functions_and_runtime.contains("fn generates_function")
            && functions_and_runtime.contains("fn generates_entry_point")
            && functions_and_runtime
                .contains("fn runtime_separates_static_and_allocator_backed_strings")
            && functions_and_runtime.contains("fn generates_function_with_defers"),
        "functions_and_runtime.rs should cover functions, entry point, runtime strings, and defers"
    );
}
