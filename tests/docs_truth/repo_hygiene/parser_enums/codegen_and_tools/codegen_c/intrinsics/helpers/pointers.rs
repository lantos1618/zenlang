use super::*;

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
