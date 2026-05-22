use super::super::*;

#[test]
fn codegen_intrinsic_enum_guards_live_in_focused_helper() {
    let root = read("tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools.rs");
    let intrinsics =
        read("tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/intrinsics.rs");

    for test_name in [
        "codegen_c_intrinsics_use_owned_name_enum",
        "codegen_c_syscall_intrinsics_live_in_focused_helper",
        "codegen_c_memory_intrinsics_live_in_focused_helper",
        "codegen_c_pointer_intrinsics_live_in_focused_helper",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "codegen_and_tools.rs should not own C intrinsic enum guard: {test_name}"
        );
        assert!(
            intrinsics.contains(&format!("fn {test_name}")),
            "C intrinsic enum guard should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 100,
        "codegen_and_tools.rs should stay focused on build graph and CLI enum guards"
    );
    assert!(
        root.contains("mod intrinsics;"),
        "codegen_and_tools.rs should include the focused intrinsic guard module"
    );
}
