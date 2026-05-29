use super::*;

impl CIntrinsic {
    pub(super) const fn is_syscall(self) -> bool {
        matches!(
            self,
            CIntrinsic::Syscall0
                | CIntrinsic::Syscall1
                | CIntrinsic::Syscall2
                | CIntrinsic::Syscall3
                | CIntrinsic::Syscall4
                | CIntrinsic::Syscall5
                | CIntrinsic::Syscall6
        )
    }

    pub(super) fn emit_syscall(self, emitter: &mut CEmitter, args: &[TypedExpression]) -> String {
        debug_assert!(
            self.is_syscall(),
            "non-syscall intrinsic routed to syscall lowering"
        );
        // Every SyscallN lowers to `syscall(<number>, <arg>...)`; the variant
        // only fixes the arity, which is just the argument count.
        let emitted_args: Vec<_> = args
            .iter()
            .map(|arg| emitter.emit_expr_inline(arg))
            .collect();
        format!("syscall({})", emitted_args.join(", "))
    }
}
