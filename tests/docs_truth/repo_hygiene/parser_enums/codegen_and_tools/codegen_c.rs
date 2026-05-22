use super::*;

#[test]
fn codegen_c_intrinsics_use_owned_name_enum() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let names = read("src/codegen/c/intrinsics/names.rs");
    let spelling = read("src/codegen/c/intrinsics/names/spelling.rs");
    let source = format!("{lowering}\n{names}\n{spelling}");

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
        "mod spelling;",
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

    assert!(
        names.lines().count() < 220,
        "names.rs should stay focused on the intrinsic enum, ordered table, and parse/display glue"
    );
    assert!(
        !names.contains("const ADD_OVERFLOW"),
        "intrinsic spelling constants should live in names/spelling.rs"
    );
    assert!(
        spelling.contains("pub(super) const fn as_str"),
        "intrinsic spelling helper should own string rendering"
    );
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
fn codegen_c_pointer_intrinsics_live_in_focused_helper() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let pointers = read("src/codegen/c/intrinsics/pointers.rs");

    for pointer_variant in [
        "CIntrinsic::IntToPtr =>",
        "CIntrinsic::PtrToInt =>",
        "CIntrinsic::Gep =>",
        "CIntrinsic::GepStruct =>",
        "CIntrinsic::RawPtrOffset =>",
        "CIntrinsic::RawPtrCast =>",
        "CIntrinsic::NullPtr | CIntrinsic::Nullptr =>",
        "CIntrinsic::IsNull =>",
    ] {
        assert!(
            !lowering.contains(pointer_variant),
            "main C intrinsic dispatcher should route pointer lowering to a focused helper: {pointer_variant}"
        );
        assert!(
            pointers.contains(pointer_variant),
            "C pointer intrinsic lowering should live in the focused pointer helper: {pointer_variant}"
        );
    }

    assert!(
        lowering.contains("intrinsic.emit_pointer(self, args, result_ty)"),
        "main C intrinsic dispatcher should delegate pointer lowering through CIntrinsic"
    );
    assert!(
        pointers.contains("pub(super) fn emit_pointer"),
        "pointer helper should own the C pointer lowering entry point"
    );
    assert!(
        lowering.lines().count() < 240,
        "C intrinsic dispatcher should stay compact after category helpers own detailed lowering"
    );
}

#[test]
fn codegen_c_atomic_intrinsics_live_in_focused_helper() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let atomics = read("src/codegen/c/intrinsics/atomics.rs");

    for atomic_variant in [
        "CIntrinsic::AtomicLoad =>",
        "CIntrinsic::AtomicStore =>",
        "CIntrinsic::AtomicAdd =>",
        "CIntrinsic::AtomicSub =>",
        "CIntrinsic::AtomicCas =>",
        "CIntrinsic::AtomicXchg =>",
        "CIntrinsic::Fence =>",
    ] {
        assert!(
            !lowering.contains(atomic_variant),
            "main C intrinsic dispatcher should route atomic lowering to a focused helper: {atomic_variant}"
        );
        assert!(
            atomics.contains(atomic_variant),
            "C atomic intrinsic lowering should live in the focused atomic helper: {atomic_variant}"
        );
    }

    assert!(
        lowering.contains("intrinsic.emit_atomic(self, args)"),
        "main C intrinsic dispatcher should delegate atomic lowering through CIntrinsic"
    );
    assert!(
        atomics.contains("pub(super) fn emit_atomic"),
        "atomic helper should own the C atomic lowering entry point"
    );
    assert!(
        lowering.lines().count() < 210,
        "C intrinsic dispatcher should stay compact after atomic lowering moves to a focused helper"
    );
}

#[test]
fn codegen_c_core_intrinsics_live_in_focused_helper() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let core = read("src/codegen/c/intrinsics/core.rs");

    for core_variant in [
        "CIntrinsic::Trap =>",
        "CIntrinsic::Bswap16 =>",
        "CIntrinsic::AddOverflow =>",
        "CIntrinsic::TruncF64I64 =>",
        "CIntrinsic::LibcWrite =>",
        "CIntrinsic::Strlen =>",
        "CIntrinsic::LoadLibrary =>",
        "CIntrinsic::InlineC =>",
        "CIntrinsic::Discriminant =>",
    ] {
        assert!(
            !lowering.contains(core_variant),
            "main C intrinsic dispatcher should route core lowering to a focused helper: {core_variant}"
        );
        assert!(
            core.contains(core_variant),
            "C core intrinsic lowering should live in the focused helper: {core_variant}"
        );
    }

    assert!(
        lowering.contains("intrinsic.emit_core(self, args)"),
        "main C intrinsic dispatcher should delegate core lowering through CIntrinsic"
    );
    assert!(
        core.contains("pub(super) fn emit_core") && core.contains("fn emit_overflow_op"),
        "core intrinsic helper should own core lowering and overflow helper"
    );
    assert!(
        lowering.lines().count() < 90,
        "C intrinsic dispatcher should stay focused on routing between intrinsic helpers"
    );
}
