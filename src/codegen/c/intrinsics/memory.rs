use super::*;

impl CIntrinsic {
    pub(super) fn emit_memory(
        self,
        emitter: &mut CEmitter,
        args: &[TypedExpression],
        result_ty: &Type,
    ) -> Option<String> {
        match self {
            CIntrinsic::RawAllocate => {
                let size = emitter.emit_expr_inline(&args[0]);
                Some(format!("malloc({})", size))
            }
            CIntrinsic::RawDeallocate => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                Some(format!("free({})", ptr))
            }
            CIntrinsic::RawReallocate => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let new_size = emitter.emit_expr_inline(&args[2]);
                Some(format!("realloc({}, {})", ptr, new_size))
            }
            CIntrinsic::Memcpy => {
                let dest = emitter.emit_expr_inline(&args[0]);
                let src = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!("memcpy({}, {}, {})", dest, src, n))
            }
            CIntrinsic::Memmove => {
                let dest = emitter.emit_expr_inline(&args[0]);
                let src = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!("memmove({}, {}, {})", dest, src, n))
            }
            CIntrinsic::Memset => {
                let dest = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!("memset({}, {}, {})", dest, val, n))
            }
            CIntrinsic::Memcmp => {
                let a = emitter.emit_expr_inline(&args[0]);
                let b = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!("memcmp({}, {}, {})", a, b, n))
            }
            CIntrinsic::Load => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let ty = emitter.c_type(result_ty);
                Some(format!("(*(({}*)({})))", ty, ptr))
            }
            CIntrinsic::Store => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                let ty = emitter.c_type(&args[1].ty);
                Some(format!("(*(({}*)({})) = ({}))", ty, ptr, val))
            }
            CIntrinsic::Sizeof => {
                if !args.is_empty() {
                    let ty = emitter.c_type(&args[0].ty);
                    Some(format!("sizeof({})", ty))
                } else {
                    emitter.line("#error \"sizeof intrinsic reached codegen without type arg\"");
                    Some("0".into())
                }
            }
            CIntrinsic::Alignof => {
                if !args.is_empty() {
                    let ty = emitter.c_type(&args[0].ty);
                    Some(format!("_Alignof({})", ty))
                } else {
                    emitter.line("#error \"alignof intrinsic reached codegen without type arg\"");
                    Some("0".into())
                }
            }
            _ => None,
        }
    }
}
