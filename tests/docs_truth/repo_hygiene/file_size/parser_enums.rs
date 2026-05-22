use super::super::*;

#[test]
fn parser_enum_codegen_c_intrinsic_guards_live_in_focused_module() {
    let root = read("tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c.rs");
    let intrinsics = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics.rs",
    );

    assert!(
        root.lines().count() < 80,
        "codegen_c.rs should route focused parser-enum/codegen C hygiene guard modules"
    );
    assert!(
        root.contains("mod intrinsics;"),
        "codegen_c.rs should include the focused C intrinsic hygiene guard module"
    );
    assert!(
        intrinsics.contains("mod helpers;") && intrinsics.contains("mod names;"),
        "codegen_c/intrinsics.rs should route focused C intrinsic hygiene guard modules"
    );

    for moved_test in [
        "fn codegen_c_intrinsics_use_owned_name_enum",
        "fn codegen_c_syscall_intrinsics_live_in_focused_helper",
        "fn codegen_c_memory_intrinsics_live_in_focused_helper",
        "fn codegen_c_pointer_intrinsics_live_in_focused_helper",
        "fn codegen_c_atomic_intrinsics_live_in_focused_helper",
        "fn codegen_c_core_intrinsics_live_in_focused_helper",
    ] {
        assert!(
            !root.contains(moved_test),
            "C intrinsic hygiene guard should move out of codegen_c.rs: {moved_test}"
        );
    }
}

#[test]
fn parser_enum_codegen_c_intrinsic_guard_modules_stay_split_by_responsibility() {
    let root = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics.rs",
    );
    let names = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/names.rs",
    );
    let helpers = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/helpers.rs",
    );

    assert!(
        root.lines().count() < 80,
        "codegen_c/intrinsics.rs should route focused intrinsic hygiene guard modules"
    );
    for module in ["mod helpers;", "mod names;"] {
        assert!(
            root.contains(module),
            "codegen_c/intrinsics.rs should include focused module `{module}`"
        );
    }

    assert!(
        !root.contains("fn codegen_c_intrinsics_use_owned_name_enum"),
        "intrinsic spelling ownership guard should live in names.rs"
    );
    assert!(
        names.contains("fn codegen_c_intrinsics_use_owned_name_enum"),
        "names.rs should guard CIntrinsic spelling ownership"
    );

    for helper_guard in [
        "fn codegen_c_syscall_intrinsics_live_in_focused_helper",
        "fn codegen_c_memory_intrinsics_live_in_focused_helper",
        "fn codegen_c_pointer_intrinsics_live_in_focused_helper",
        "fn codegen_c_atomic_intrinsics_live_in_focused_helper",
        "fn codegen_c_core_intrinsics_live_in_focused_helper",
    ] {
        assert!(
            !root.contains(helper_guard),
            "intrinsic helper routing guard should move out of intrinsics.rs: {helper_guard}"
        );
        assert!(
            helpers.contains(helper_guard),
            "helpers.rs should keep intrinsic helper routing guard: {helper_guard}"
        );
    }
}
