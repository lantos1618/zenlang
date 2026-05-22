use super::*;

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
