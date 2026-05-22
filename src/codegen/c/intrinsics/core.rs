use super::*;

impl CIntrinsic {
    pub(super) fn emit_core(
        self,
        emitter: &mut CEmitter,
        args: &[TypedExpression],
    ) -> Option<String> {
        match self {
            // -- Debug / trap / panic -------------------------------------
            CIntrinsic::Trap => Some("(__builtin_trap(), (void)0)".into()),
            CIntrinsic::Debugtrap => Some("(__builtin_debugtrap(), (void)0)".into()),
            CIntrinsic::Unreachable => Some("(__builtin_unreachable(), (void)0)".into()),
            CIntrinsic::Panic => {
                let msg = emitter.emit_expr_inline(&args[0]);
                Some(format!(
                    "(fprintf(stderr, \"panic: %.*s\\n\", (int)({msg}).len, ({msg}).ptr), abort(), (void)0)"
                ))
            }

            // -- Bitwise operations ---------------------------------------
            CIntrinsic::Bswap16 => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("__builtin_bswap16({})", val))
            }
            CIntrinsic::Bswap32 => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("__builtin_bswap32({})", val))
            }
            CIntrinsic::Bswap64 => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("__builtin_bswap64({})", val))
            }
            CIntrinsic::Ctlz => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((uint64_t)__builtin_clzll({}))", val))
            }
            CIntrinsic::Cttz => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((uint64_t)__builtin_ctzll({}))", val))
            }
            CIntrinsic::Ctpop => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((uint64_t)__builtin_popcountll({}))", val))
            }

            // -- Overflow-checked arithmetic ------------------------------
            CIntrinsic::AddOverflow => Some(emit_overflow_op(emitter, "add", args)),
            CIntrinsic::SubOverflow => Some(emit_overflow_op(emitter, "sub", args)),
            CIntrinsic::MulOverflow => Some(emit_overflow_op(emitter, "mul", args)),

            // -- Type conversions -----------------------------------------
            CIntrinsic::TruncF64I64 => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((int64_t)({}))", val))
            }
            CIntrinsic::TruncF32I32 => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((int32_t)({}))", val))
            }
            CIntrinsic::SitofpI64F64 => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((double)({}))", val))
            }
            CIntrinsic::UitofpU64F64 => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((double)({}))", val))
            }

            // -- IO (libc wrappers) ---------------------------------------
            CIntrinsic::LibcWrite => {
                let fd = emitter.emit_expr_inline(&args[0]);
                let buf = emitter.emit_expr_inline(&args[1]);
                let len = emitter.emit_expr_inline(&args[2]);
                Some(format!("((int64_t)write({}, {}, {}))", fd, buf, len))
            }
            CIntrinsic::LibcRead => {
                let fd = emitter.emit_expr_inline(&args[0]);
                let buf = emitter.emit_expr_inline(&args[1]);
                let len = emitter.emit_expr_inline(&args[2]);
                Some(format!("((int64_t)read({}, {}, {}))", fd, buf, len))
            }

            // -- String operations ----------------------------------------
            CIntrinsic::Strlen => {
                let s = emitter.emit_expr_inline(&args[0]);
                Some(format!("strlen({})", s))
            }
            CIntrinsic::StaticStringPtr => {
                let s = emitter.emit_expr_inline(&args[0]);
                Some(format!("((uint8_t*)({}))", s))
            }

            // -- FFI / dynamic loading ------------------------------------
            CIntrinsic::LoadLibrary => {
                let path = emitter.emit_expr_inline(&args[0]);
                Some(format!("dlopen({}, RTLD_LAZY)", path))
            }
            CIntrinsic::GetSymbol => {
                let handle = emitter.emit_expr_inline(&args[0]);
                let sym = emitter.emit_expr_inline(&args[1]);
                Some(format!("dlsym({}, {})", handle, sym))
            }
            CIntrinsic::UnloadLibrary => {
                let handle = emitter.emit_expr_inline(&args[0]);
                Some(format!("dlclose({})", handle))
            }
            CIntrinsic::Dlerror => Some("((uint8_t*)dlerror())".into()),
            CIntrinsic::CallExternal => {
                let fptr = emitter.emit_expr_inline(&args[0]);
                Some(format!("((int64_t(*)(void))({}))()", fptr))
            }

            // -- Inline C -------------------------------------------------
            CIntrinsic::InlineC => {
                if let TypedExprKind::StringLiteral(code) = &args[0].kind {
                    emitter.line(code);
                } else {
                    emitter.line("#error \"inline_c requires a string literal\"");
                }
                Some("(void)0".into())
            }

            // -- Enum intrinsics ------------------------------------------
            CIntrinsic::Discriminant => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((int32_t)(((int32_t*)({}))[0]))", val))
            }
            CIntrinsic::SetDiscriminant => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let disc = emitter.emit_expr_inline(&args[1]);
                Some(format!("(((int32_t*)({})) [0] = ({}))", ptr, disc))
            }
            CIntrinsic::GetPayload => {
                let val = emitter.emit_expr_inline(&args[0]);
                Some(format!("((uint8_t*)({}) + sizeof(int32_t))", val))
            }
            CIntrinsic::SetPayload => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let payload = emitter.emit_expr_inline(&args[1]);
                Some(format!(
                    "(memcpy((uint8_t*)({}) + sizeof(int32_t), {}, 0), (void)0)",
                    ptr, payload
                ))
            }
            _ => None,
        }
    }
}

fn emit_overflow_op(emitter: &mut CEmitter, op: &str, args: &[TypedExpression]) -> String {
    let a = emitter.emit_expr_inline(&args[0]);
    let b = emitter.emit_expr_inline(&args[1]);
    let result_tmp = emitter.fresh_tmp();
    let overflow_tmp = emitter.fresh_tmp();
    emitter.line(&format!("int64_t {} = 0;", result_tmp));
    emitter.line(&format!(
        "bool {} = __builtin_{}_overflow({}, {}, &{});",
        overflow_tmp, op, a, b, result_tmp
    ));
    format!(
        "((OverflowResult){{ .result = {}, .overflow = {} }})",
        result_tmp, overflow_tmp
    )
}
