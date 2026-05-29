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
            // Byte-addressed pointer arithmetic: all three compute
            // `base + offset` over a uint8_t* view, differing only in name.
            CIntrinsic::Gep | CIntrinsic::GepStruct | CIntrinsic::RawPtrOffset => {
                let base = emitter.emit_expr_inline(&args[0]);
                let offset = emitter.emit_expr_inline(&args[1]);
                Some(format!("((uint8_t*)({}) + ({}))", base, offset))
            }
            CIntrinsic::RawPtrCast => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let ty = CEmitter::c_type(result_ty);
                Some(format!("(({})({}))", ty, ptr))
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
