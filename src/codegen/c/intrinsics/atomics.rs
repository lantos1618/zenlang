use super::*;

impl CIntrinsic {
    pub(super) fn emit_atomic(
        self,
        emitter: &mut CEmitter,
        args: &[TypedExpression],
    ) -> Option<String> {
        match self {
            CIntrinsic::AtomicLoad => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                Some(format!("__atomic_load_n({}, __ATOMIC_SEQ_CST)", ptr))
            }
            CIntrinsic::AtomicStore => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                Some(format!(
                    "__atomic_store_n({}, {}, __ATOMIC_SEQ_CST)",
                    ptr, val
                ))
            }
            CIntrinsic::AtomicAdd => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                Some(format!(
                    "__atomic_fetch_add({}, {}, __ATOMIC_SEQ_CST)",
                    ptr, val
                ))
            }
            CIntrinsic::AtomicSub => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                Some(format!(
                    "__atomic_fetch_sub({}, {}, __ATOMIC_SEQ_CST)",
                    ptr, val
                ))
            }
            CIntrinsic::AtomicCas => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let expected = emitter.emit_expr_inline(&args[1]);
                let desired = emitter.emit_expr_inline(&args[2]);
                let tmp = emitter.fresh_tmp();
                emitter.line(&format!("uint64_t {} = {};", tmp, expected));
                Some(format!(
                    "__atomic_compare_exchange_n({}, &{}, {}, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST)",
                    ptr, tmp, desired
                ))
            }
            CIntrinsic::AtomicXchg => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                Some(format!(
                    "__atomic_exchange_n({}, {}, __ATOMIC_SEQ_CST)",
                    ptr, val
                ))
            }
            CIntrinsic::Fence => Some("(__atomic_thread_fence(__ATOMIC_SEQ_CST), (void)0)".into()),
            _ => None,
        }
    }
}
