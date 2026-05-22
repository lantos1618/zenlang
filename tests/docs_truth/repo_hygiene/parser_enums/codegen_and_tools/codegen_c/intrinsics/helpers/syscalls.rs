use super::*;

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
