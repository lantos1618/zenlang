use super::*;
use names::CIntrinsic;

mod names;

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

        match intrinsic {
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
            CIntrinsic::Bswap16 => format!("__builtin_bswap16({})", self.arg0(args)),
            CIntrinsic::Bswap32 => format!("__builtin_bswap32({})", self.arg0(args)),
            CIntrinsic::Bswap64 => format!("__builtin_bswap64({})", self.arg0(args)),
            CIntrinsic::Ctlz => format!("((uint64_t)__builtin_clzll({}))", self.arg0(args)),
            CIntrinsic::Cttz => format!("((uint64_t)__builtin_ctzll({}))", self.arg0(args)),
            CIntrinsic::Ctpop => format!("((uint64_t)__builtin_popcountll({}))", self.arg0(args)),

            // -- Overflow-checked arithmetic ------------------------------
            CIntrinsic::AddOverflow => self.emit_overflow_op("add", args),
            CIntrinsic::SubOverflow => self.emit_overflow_op("sub", args),
            CIntrinsic::MulOverflow => self.emit_overflow_op("mul", args),

            // -- Type conversions -----------------------------------------
            CIntrinsic::TruncF64I64 => format!("((int64_t)({}))", self.arg0(args)),
            CIntrinsic::TruncF32I32 => format!("((int32_t)({}))", self.arg0(args)),
            CIntrinsic::SitofpI64F64 => format!("((double)({}))", self.arg0(args)),
            CIntrinsic::UitofpU64F64 => format!("((double)({}))", self.arg0(args)),

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
                // A StaticString lowers to `zen_str { ptr, len }`; its length is
                // known without a libc call.
                format!("(({}).len)", self.arg0(args))
            }
            CIntrinsic::StaticStringPtr => format!("((uint8_t*)({}).ptr)", self.arg0(args)),

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
            CIntrinsic::UnloadLibrary => format!("dlclose({})", self.arg0(args)),
            CIntrinsic::Dlerror => "((uint8_t*)dlerror())".into(),
            CIntrinsic::CallExternal => format!("((int64_t(*)(void))({}))()", self.arg0(args)),

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

            _ => unreachable!("syscall intrinsic should be handled before category lowering"),
        }
    }

    /// Emit the sole operand of a unary intrinsic. Most intrinsics are a
    /// single-argument C builtin wrapper; this keeps those arms one-liners.
    fn arg0(&mut self, args: &[TypedExpression]) -> String {
        self.emit_expr_inline(&args[0])
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
                Some(format!("free((void*)({}))", ptr))
            }
            CIntrinsic::RawReallocate => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let new_size = emitter.emit_expr_inline(&args[2]);
                Some(format!("realloc((void*)({}), {})", ptr, new_size))
            }
            CIntrinsic::Memcpy => {
                let dest = emitter.emit_expr_inline(&args[0]);
                let src = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!(
                    "memcpy((void*)({}), (const void*)({}), {})",
                    dest, src, n
                ))
            }
            CIntrinsic::Memmove => {
                let dest = emitter.emit_expr_inline(&args[0]);
                let src = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!(
                    "memmove((void*)({}), (const void*)({}), {})",
                    dest, src, n
                ))
            }
            CIntrinsic::Memset => {
                let dest = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!("memset((void*)({}), {}, {})", dest, val, n))
            }
            CIntrinsic::Memcmp => {
                let a = emitter.emit_expr_inline(&args[0]);
                let b = emitter.emit_expr_inline(&args[1]);
                let n = emitter.emit_expr_inline(&args[2]);
                Some(format!("memcmp({}, {}, {})", a, b, n))
            }
            CIntrinsic::Load => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let ty = CEmitter::c_type(result_ty);
                Some(format!("(*(({}*)({})))", ty, ptr))
            }
            CIntrinsic::Store => {
                let ptr = emitter.emit_expr_inline(&args[0]);
                let val = emitter.emit_expr_inline(&args[1]);
                let ty = CEmitter::c_type(&args[1].ty);
                Some(format!("(*(({}*)({})) = ({}))", ty, ptr, val))
            }
            CIntrinsic::Sizeof => {
                if !args.is_empty() {
                    let ty = CEmitter::c_type(&args[0].ty);
                    Some(format!("sizeof({})", ty))
                } else {
                    emitter.line("#error \"sizeof intrinsic reached codegen without type arg\"");
                    Some("0".into())
                }
            }
            CIntrinsic::Alignof => {
                if !args.is_empty() {
                    let ty = CEmitter::c_type(&args[0].ty);
                    Some(format!("_Alignof({})", ty))
                } else {
                    emitter.line("#error \"alignof intrinsic reached codegen without type arg\"");
                    Some("0".into())
                }
            }
            _ => None,
        }
    }

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
