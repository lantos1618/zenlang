use super::*;

impl CIntrinsic {
    pub(super) fn emit_pointer(
        self,
        emitter: &mut CEmitter,
        args: &[TypedExpression],
        result_ty: &Type,
    ) -> Option<String> {
        match self {
            CIntrinsic::IntToPtr => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((void*)(uintptr_t)({}))", val))
            }
            CIntrinsic::PtrToInt => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                Some(format!("((uintptr_t)({}))", ptr))
            }
            CIntrinsic::Gep => {
                let base = emitter.emit_expr_inline(&args[0]);
                let offset = emitter.emit_expr_inline(&args[1]);
                Some(format!("((uint8_t*)({}) + ({}))", base, offset))
            }
            CIntrinsic::GepStruct => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let idx = emitter.emit_expr_inline(&args[1]);
                Some(format!("((uint8_t*)({}) + ({}))", ptr, idx))
            }
            CIntrinsic::RawPtrOffset => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let offset = emitter.emit_expr_inline(&args[1]);
                Some(format!("((uint8_t*)({}) + ({}))", ptr, offset))
            }
            CIntrinsic::RawPtrCast => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let ty = emitter.c_type(result_ty);
                Some(format!("(({})({})", ty, ptr))
            }
            CIntrinsic::NullPtr | CIntrinsic::Nullptr => Some("(NULL)".into()),
            CIntrinsic::IsNull => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                Some(format!("(({}) == NULL)", ptr))
            }
            _ => None,
        }
    }
}
