use super::*;

mod atomics;
mod core;
mod memory;
mod pointers;
mod syscalls;

#[test]
fn codegen_c_intrinsic_helper_guards_stay_split_by_category() {
    let root = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/helpers.rs",
    );
    let atomics = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/helpers/atomics.rs",
    );
    let core = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/helpers/core.rs",
    );
    let memory = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/helpers/memory.rs",
    );
    let pointers = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/helpers/pointers.rs",
    );
    let syscalls = read(
        "tests/docs_truth/repo_hygiene/parser_enums/codegen_and_tools/codegen_c/intrinsics/helpers/syscalls.rs",
    );

    assert!(
        root.lines().count() < 80,
        "intrinsics/helpers.rs should route focused category guard modules"
    );
    for module_name in ["atomics", "core", "memory", "pointers", "syscalls"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "intrinsics/helpers.rs should include focused module: {module_name}"
        );
    }
    assert!(
        atomics.contains("fn codegen_c_atomic_intrinsics_live_in_focused_helper"),
        "atomic intrinsic guards should live in helpers/atomics.rs"
    );
    assert!(
        core.contains("fn codegen_c_core_intrinsics_live_in_focused_helper"),
        "core intrinsic guards should live in helpers/core.rs"
    );
    assert!(
        memory.contains("fn codegen_c_memory_intrinsics_live_in_focused_helper"),
        "memory intrinsic guards should live in helpers/memory.rs"
    );
    assert!(
        pointers.contains("fn codegen_c_pointer_intrinsics_live_in_focused_helper"),
        "pointer intrinsic guards should live in helpers/pointers.rs"
    );
    assert!(
        syscalls.contains("fn codegen_c_syscall_intrinsics_live_in_focused_helper"),
        "syscall intrinsic guards should live in helpers/syscalls.rs"
    );
}
