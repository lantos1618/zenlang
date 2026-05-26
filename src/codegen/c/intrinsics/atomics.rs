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
            CIntrinsic::AtomicStore => Some(emit_atomic_ptr_val(emitter, args, "__atomic_store_n")),
            CIntrinsic::AtomicAdd => Some(emit_atomic_ptr_val(emitter, args, "__atomic_fetch_add")),
            CIntrinsic::AtomicSub => Some(emit_atomic_ptr_val(emitter, args, "__atomic_fetch_sub")),
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
                Some(emit_atomic_ptr_val(emitter, args, "__atomic_exchange_n"))
            }
            CIntrinsic::Fence => Some("(__atomic_thread_fence(__ATOMIC_SEQ_CST), (void)0)".into()),
            _ => None,
        }
    }
}

fn emit_atomic_ptr_val(
    emitter: &mut CEmitter,
    args: &[TypedExpression],
    intrinsic: &str,
) -> String {
    let ptr = emitter.emit_expr_inline(&args[0]);
    let val = emitter.emit_expr_inline(&args[1]);
    format!("{intrinsic}({ptr}, {val}, __ATOMIC_SEQ_CST)")
}
