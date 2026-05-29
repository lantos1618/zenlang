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
        let emitted_args: Vec<_> = args
            .iter()
            .map(|arg| emitter.emit_expr_inline(arg))
            .collect();
        match self {
            CIntrinsic::Syscall0 => format!("syscall({})", emitted_args[0]),
            CIntrinsic::Syscall1 => format!("syscall({}, {})", emitted_args[0], emitted_args[1]),
            CIntrinsic::Syscall2 => {
                format!(
                    "syscall({}, {}, {})",
                    emitted_args[0], emitted_args[1], emitted_args[2]
                )
            }
            CIntrinsic::Syscall3 => {
                format!(
                    "syscall({}, {}, {}, {})",
                    emitted_args[0], emitted_args[1], emitted_args[2], emitted_args[3]
                )
            }
            CIntrinsic::Syscall4 => {
                format!(
                    "syscall({}, {}, {}, {}, {})",
                    emitted_args[0],
                    emitted_args[1],
                    emitted_args[2],
                    emitted_args[3],
                    emitted_args[4]
                )
            }
            CIntrinsic::Syscall5 => {
                format!(
                    "syscall({}, {}, {}, {}, {}, {})",
                    emitted_args[0],
                    emitted_args[1],
                    emitted_args[2],
                    emitted_args[3],
                    emitted_args[4],
                    emitted_args[5]
                )
            }
            CIntrinsic::Syscall6 => {
                format!(
                    "syscall({}, {}, {}, {}, {}, {}, {})",
                    emitted_args[0],
                    emitted_args[1],
                    emitted_args[2],
                    emitted_args[3],
                    emitted_args[4],
                    emitted_args[5],
                    emitted_args[6]
                )
            }
            _ => unreachable!("non-syscall intrinsic routed to syscall lowering"),
        }
    }
}
