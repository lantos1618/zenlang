use super::*;

#[test]
fn codegen_c_intrinsics_use_owned_name_enum() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let names = read("src/codegen/c/intrinsics/names.rs");
    let source = format!("{lowering}\n{names}");

    for forbidden in [
        "match name",
        r#""raw_allocate" =>"#,
        r#""raw_deallocate" =>"#,
        r#""raw_reallocate" =>"#,
        r#""memcpy" =>"#,
        r#""memmove" =>"#,
        r#""memset" =>"#,
        r#""memcmp" =>"#,
        r#""atomic_load" =>"#,
        r#""atomic_store" =>"#,
        r#""atomic_add" =>"#,
        r#""atomic_sub" =>"#,
        r#""atomic_cas" =>"#,
        r#""atomic_xchg" =>"#,
        r#""syscall0" =>"#,
        r#""syscall1" =>"#,
        r#""syscall2" =>"#,
        r#""syscall3" =>"#,
        r#""syscall4" =>"#,
        r#""syscall5" =>"#,
        r#""syscall6" =>"#,
    ] {
        assert!(
            !lowering.contains(forbidden),
            "C intrinsic lowering should parse through CIntrinsic, not raw spelling dispatch: {forbidden}"
        );
    }

    for required in [
        "enum CIntrinsic",
        "const ALL: &[CIntrinsic]",
        "impl fmt::Display for CIntrinsic",
        "impl FromStr for CIntrinsic",
        "name.parse::<CIntrinsic>()",
        "Self::RAW_ALLOCATE",
        "Self::ATOMIC_LOAD",
        "Self::SYSCALL6",
    ] {
        assert!(
            source.contains(required),
            "C intrinsic spelling should live in CIntrinsic: {required}"
        );
    }
}

#[test]
fn codegen_c_syscall_intrinsics_live_in_focused_helper() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let syscalls = read("src/codegen/c/intrinsics/syscalls.rs");

    for syscall_variant in [
        "CIntrinsic::Syscall0",
        "CIntrinsic::Syscall1",
        "CIntrinsic::Syscall2",
        "CIntrinsic::Syscall3",
        "CIntrinsic::Syscall4",
        "CIntrinsic::Syscall5",
        "CIntrinsic::Syscall6",
    ] {
        assert!(
            !lowering.contains(syscall_variant),
            "main C intrinsic dispatcher should route syscall lowering to a focused helper: {syscall_variant}"
        );
        assert!(
            syscalls.contains(syscall_variant),
            "C syscall intrinsic lowering should live in the focused syscall helper: {syscall_variant}"
        );
    }

    assert!(
        lowering.contains("intrinsic.emit_syscall(self, args)"),
        "main C intrinsic dispatcher should delegate syscall lowering through CIntrinsic"
    );
    assert!(
        syscalls.contains("pub(super) fn emit_syscall"),
        "syscall helper should own the C syscall lowering entry point"
    );
}

#[test]
fn codegen_c_memory_intrinsics_live_in_focused_helper() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let memory = read("src/codegen/c/intrinsics/memory.rs");

    for memory_variant in [
        "CIntrinsic::RawAllocate =>",
        "CIntrinsic::RawDeallocate =>",
        "CIntrinsic::RawReallocate =>",
        "CIntrinsic::Memcpy =>",
        "CIntrinsic::Memmove =>",
        "CIntrinsic::Memset =>",
        "CIntrinsic::Memcmp =>",
        "CIntrinsic::Load =>",
        "CIntrinsic::Store =>",
        "CIntrinsic::Sizeof =>",
        "CIntrinsic::Alignof =>",
    ] {
        assert!(
            !lowering.contains(memory_variant),
            "main C intrinsic dispatcher should route memory lowering to a focused helper: {memory_variant}"
        );
        assert!(
            memory.contains(memory_variant),
            "C memory intrinsic lowering should live in the focused memory helper: {memory_variant}"
        );
    }

    assert!(
        lowering.contains("intrinsic.emit_memory(self, args, result_ty)"),
        "main C intrinsic dispatcher should delegate memory lowering through CIntrinsic"
    );
    assert!(
        memory.contains("pub(super) fn emit_memory"),
        "memory helper should own the C memory lowering entry point"
    );
}

#[test]
fn build_graph_host_effect_methods_parse_dsl_ident_enum() {
    let lowering = read("src/build_graph/lowering.rs");
    let dsl = read("src/build_graph/lowering/dsl.rs");

    for forbidden in [
        "match method.as_str()",
        "method == BuildTargetDslIdent::Env.as_str()",
        "method == BuildTargetDslIdent::ReadFile.as_str()",
    ] {
        assert!(
            !lowering.contains(forbidden),
            "build graph host-effect method dispatch should parse through BuildTargetDslIdent: {forbidden}"
        );
    }
    assert!(
        lowering.contains("method.parse::<BuildTargetDslIdent>()"),
        "build graph host-effect method dispatch should parse method names through BuildTargetDslIdent"
    );
    assert!(
        dsl.contains("impl FromStr for BuildTargetDslIdent"),
        "BuildTargetDslIdent should own parsing for build DSL method names"
    );
}

#[test]
fn cli_emit_json_modes_use_owned_mode_enum() {
    let source = read("src/cli.rs");

    assert!(
        source.contains("enum EmitJsonMode"),
        "emit-json command routing should use an owned EmitJsonMode enum"
    );
    assert!(
        source.contains("mode.parse::<EmitJsonMode>()"),
        "emit-json command routing should parse modes through EmitJsonMode"
    );
    assert!(
        source.contains("EmitJsonMode::usage()"),
        "emit-json usage should be generated from EmitJsonMode"
    );
    assert!(
        source.contains(".find(|mode| mode.as_str() == value)"),
        "emit-json mode parsing should use the enum-owned ordered table"
    );
    assert!(
        source.contains("fn gate_message(self) -> Option<&'static str>"),
        "emit-json gated diagnostics should be owned by EmitJsonMode"
    );
    assert!(
        source.contains("mode.gate_message()"),
        "emit-json command routing should read gated diagnostics from EmitJsonMode"
    );
    assert!(
        !source.contains("<ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml>"),
        "emit-json usage should not duplicate the mode list as a raw string"
    );
}
