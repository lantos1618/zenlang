use super::*;

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
