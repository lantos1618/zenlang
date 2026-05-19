use super::*;
use names::CIntrinsic;

mod names;
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

        match intrinsic {
            _ if intrinsic.is_syscall() => intrinsic.emit_syscall(self, args),

            // -- Memory allocation ----------------------------------------
            CIntrinsic::RawAllocate => {
                let size = self.emit_expr_inline(&args[0]);
                format!("malloc({})", size)
            }
            CIntrinsic::RawDeallocate => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("free({})", ptr)
            }
            CIntrinsic::RawReallocate => {
                let ptr = self.emit_expr_inline(&args[0]);
                let new_size = self.emit_expr_inline(&args[2]);
                format!("realloc({}, {})", ptr, new_size)
            }

            // -- Memory operations ----------------------------------------
            CIntrinsic::Memcpy => {
                let dest = self.emit_expr_inline(&args[0]);
                let src = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memcpy({}, {}, {})", dest, src, n)
            }
            CIntrinsic::Memmove => {
                let dest = self.emit_expr_inline(&args[0]);
                let src = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memmove({}, {}, {})", dest, src, n)
            }
            CIntrinsic::Memset => {
                let dest = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memset({}, {}, {})", dest, val, n)
            }
            CIntrinsic::Memcmp => {
                let a = self.emit_expr_inline(&args[0]);
                let b = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memcmp({}, {}, {})", a, b, n)
            }

            // -- Load / Store ---------------------------------------------
            CIntrinsic::Load => {
                let ptr = self.emit_expr_inline(&args[0]);
                let ty = self.c_type(result_ty);
                format!("(*(({}*)({})))", ty, ptr)
            }
            CIntrinsic::Store => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                let ty = self.c_type(&args[1].ty);
                format!("(*(({}*)({})) = ({}))", ty, ptr, val)
            }

            // -- Type introspection ---------------------------------------
            CIntrinsic::Sizeof => {
                if !args.is_empty() {
                    let ty = self.c_type(&args[0].ty);
                    format!("sizeof({})", ty)
                } else {
                    // sizeof<T>() with no runtime args — typechecker should
                    // have resolved this to a constant before reaching codegen.
                    self.line("#error \"sizeof intrinsic reached codegen without type arg\"");
                    "0".into()
                }
            }
            CIntrinsic::Alignof => {
                if !args.is_empty() {
                    let ty = self.c_type(&args[0].ty);
                    format!("_Alignof({})", ty)
                } else {
                    self.line("#error \"alignof intrinsic reached codegen without type arg\"");
                    "0".into()
                }
            }

            // -- Pointer operations ---------------------------------------
            CIntrinsic::IntToPtr => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((void*)(uintptr_t)({}))", val)
            }
            CIntrinsic::PtrToInt => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("((uintptr_t)({}))", ptr)
            }
            CIntrinsic::Gep => {
                let base = self.emit_expr_inline(&args[0]);
                let offset = self.emit_expr_inline(&args[1]);
                format!("((uint8_t*)({}) + ({}))", base, offset)
            }
            CIntrinsic::GepStruct => {
                // Struct GEP: byte-offset into a struct by field index.
                // In C we just cast to uint8_t* and offset, since the
                // actual field layout matches.
                let ptr = self.emit_expr_inline(&args[0]);
                let idx = self.emit_expr_inline(&args[1]);
                format!("((uint8_t*)({}) + ({}))", ptr, idx)
            }
            CIntrinsic::RawPtrOffset => {
                let ptr = self.emit_expr_inline(&args[0]);
                let offset = self.emit_expr_inline(&args[1]);
                format!("((uint8_t*)({}) + ({}))", ptr, offset)
            }
            CIntrinsic::RawPtrCast => {
                let ptr = self.emit_expr_inline(&args[0]);
                let ty = self.c_type(result_ty);
                format!("(({})({})", ty, ptr)
            }
            CIntrinsic::NullPtr | CIntrinsic::Nullptr => "(NULL)".into(),
            CIntrinsic::IsNull => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("(({}) == NULL)", ptr)
            }

            // -- Atomic operations ----------------------------------------
            CIntrinsic::AtomicLoad => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("__atomic_load_n({}, __ATOMIC_SEQ_CST)", ptr)
            }
            CIntrinsic::AtomicStore => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_store_n({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            CIntrinsic::AtomicAdd => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_fetch_add({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            CIntrinsic::AtomicSub => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_fetch_sub({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            CIntrinsic::AtomicCas => {
                let ptr = self.emit_expr_inline(&args[0]);
                let expected = self.emit_expr_inline(&args[1]);
                let desired = self.emit_expr_inline(&args[2]);
                let tmp = self.fresh_tmp();
                self.line(&format!("uint64_t {} = {};", tmp, expected));
                format!(
                    "__atomic_compare_exchange_n({}, &{}, {}, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST)",
                    ptr, tmp, desired
                )
            }
            CIntrinsic::AtomicXchg => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_exchange_n({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            CIntrinsic::Fence => "(__atomic_thread_fence(__ATOMIC_SEQ_CST), (void)0)".into(),

            // -- Debug / trap / panic -------------------------------------
            CIntrinsic::Trap => "(__builtin_trap(), (void)0)".into(),
            CIntrinsic::Debugtrap => "(__builtin_debugtrap(), (void)0)".into(),
            CIntrinsic::Unreachable => "(__builtin_unreachable(), (void)0)".into(),
            CIntrinsic::Panic => {
                let msg = self.emit_expr_inline(&args[0]);
                format!(
                    "(fprintf(stderr, \"panic: %.*s\\n\", (int)({msg}).len, ({msg}).ptr), abort(), (void)0)"
                )
            }

            // -- Bitwise operations ---------------------------------------
            CIntrinsic::Bswap16 => {
                let val = self.emit_expr_inline(&args[0]);
                format!("__builtin_bswap16({})", val)
            }
            CIntrinsic::Bswap32 => {
                let val = self.emit_expr_inline(&args[0]);
                format!("__builtin_bswap32({})", val)
            }
            CIntrinsic::Bswap64 => {
                let val = self.emit_expr_inline(&args[0]);
                format!("__builtin_bswap64({})", val)
            }
            CIntrinsic::Ctlz => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((uint64_t)__builtin_clzll({}))", val)
            }
            CIntrinsic::Cttz => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((uint64_t)__builtin_ctzll({}))", val)
            }
            CIntrinsic::Ctpop => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((uint64_t)__builtin_popcountll({}))", val)
            }

            // -- Overflow-checked arithmetic ------------------------------
            CIntrinsic::AddOverflow => self.emit_overflow_op("add", args),
            CIntrinsic::SubOverflow => self.emit_overflow_op("sub", args),
            CIntrinsic::MulOverflow => self.emit_overflow_op("mul", args),

            // -- Type conversions -----------------------------------------
            CIntrinsic::TruncF64I64 => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((int64_t)({}))", val)
            }
            CIntrinsic::TruncF32I32 => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((int32_t)({}))", val)
            }
            CIntrinsic::SitofpI64F64 => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((double)({}))", val)
            }
            CIntrinsic::UitofpU64F64 => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((double)({}))", val)
            }

            // -- IO (libc wrappers) ---------------------------------------
            CIntrinsic::LibcWrite => {
                let fd = self.emit_expr_inline(&args[0]);
                let buf = self.emit_expr_inline(&args[1]);
                let len = self.emit_expr_inline(&args[2]);
                format!("((int64_t)write({}, {}, {}))", fd, buf, len)
            }
            CIntrinsic::LibcRead => {
                let fd = self.emit_expr_inline(&args[0]);
                let buf = self.emit_expr_inline(&args[1]);
                let len = self.emit_expr_inline(&args[2]);
                format!("((int64_t)read({}, {}, {}))", fd, buf, len)
            }

            // -- String operations ----------------------------------------
            CIntrinsic::Strlen => {
                let s = self.emit_expr_inline(&args[0]);
                format!("strlen({})", s)
            }
            CIntrinsic::StaticStringPtr => {
                let s = self.emit_expr_inline(&args[0]);
                format!("((uint8_t*)({}))", s)
            }

            // -- FFI / dynamic loading ------------------------------------
            CIntrinsic::LoadLibrary => {
                let path = self.emit_expr_inline(&args[0]);
                format!("dlopen({}, RTLD_LAZY)", path)
            }
            CIntrinsic::GetSymbol => {
                let handle = self.emit_expr_inline(&args[0]);
                let sym = self.emit_expr_inline(&args[1]);
                format!("dlsym({}, {})", handle, sym)
            }
            CIntrinsic::UnloadLibrary => {
                let handle = self.emit_expr_inline(&args[0]);
                format!("dlclose({})", handle)
            }
            CIntrinsic::Dlerror => "((uint8_t*)dlerror())".into(),
            CIntrinsic::CallExternal => {
                let fptr = self.emit_expr_inline(&args[0]);
                format!("((int64_t(*)(void))({}))()", fptr)
            }

            // -- Inline C -------------------------------------------------
            CIntrinsic::InlineC => {
                // The arg should be a string literal containing raw C code.
                if let TypedExprKind::StringLiteral(code) = &args[0].kind {
                    self.line(code);
                    "(void)0".into()
                } else {
                    self.line("#error \"inline_c requires a string literal\"");
                    "(void)0".into()
                }
            }

            // -- Enum intrinsics ------------------------------------------
            CIntrinsic::Discriminant => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((int32_t)(((int32_t*)({}))[0]))", val)
            }
            CIntrinsic::SetDiscriminant => {
                let ptr = self.emit_expr_inline(&args[0]);
                let disc = self.emit_expr_inline(&args[1]);
                format!("(((int32_t*)({})) [0] = ({}))", ptr, disc)
            }
            CIntrinsic::GetPayload => {
                let val = self.emit_expr_inline(&args[0]);
                // Payload sits after the discriminant (4 bytes, aligned)
                format!("((uint8_t*)({}) + sizeof(int32_t))", val)
            }
            CIntrinsic::SetPayload => {
                let ptr = self.emit_expr_inline(&args[0]);
                let payload = self.emit_expr_inline(&args[1]);
                // This is a raw byte copy; caller must know the size.
                // For now emit a memcpy placeholder — the typechecker
                // should provide size info in practice.
                format!(
                    "(memcpy((uint8_t*)({}) + sizeof(int32_t), {}, 0), (void)0)",
                    ptr, payload
                )
            }
            _ => unreachable!("syscall intrinsic should be handled before category lowering"),
        }
    }

    pub(super) fn emit_overflow_op(&mut self, op: &str, args: &[TypedExpression]) -> String {
        let a = self.emit_expr_inline(&args[0]);
        let b = self.emit_expr_inline(&args[1]);
        let result_tmp = self.fresh_tmp();
        let overflow_tmp = self.fresh_tmp();
        self.line(&format!("int64_t {} = 0;", result_tmp));
        self.line(&format!(
            "bool {} = __builtin_{}_overflow({}, {}, &{});",
            overflow_tmp, op, a, b, result_tmp
        ));
        // Return as an OverflowResult struct literal
        format!(
            "((OverflowResult){{ .result = {}, .overflow = {} }})",
            result_tmp, overflow_tmp
        )
    }
}
