use super::*;

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
