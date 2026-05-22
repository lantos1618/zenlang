use super::*;
use names::CIntrinsic;

mod atomics;
mod core;
mod memory;
mod names;
mod pointers;
mod syscalls;

impl CEmitter {
    // ── Intrinsics ────────────────────────────────────────────

    pub(super) fn emit_intrinsic(
        &mut self,
        name: &str,
        args: &[TypedExpression],
        result_ty: &Type,
    ) -> String {
        let Ok(intrinsic) = name.parse::<CIntrinsic>() else {
            self.line(&format!("#error \"Unknown intrinsic: {}\"", name));
            return "(void)0".into();
        };

        if intrinsic.is_syscall() {
            return intrinsic.emit_syscall(self, args);
        }
        if let Some(emitted) = intrinsic.emit_pointer(self, args, result_ty) {
            return emitted;
        }
        if let Some(emitted) = intrinsic.emit_memory(self, args, result_ty) {
            return emitted;
        }
        if let Some(emitted) = intrinsic.emit_atomic(self, args) {
            return emitted;
        }
        if let Some(emitted) = intrinsic.emit_core(self, args) {
            return emitted;
        }

        unreachable!("all C intrinsics should be handled by category lowering")
    }
}
