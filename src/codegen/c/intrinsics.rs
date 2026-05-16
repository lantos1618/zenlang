use super::*;

impl CEmitter {
    // ── Intrinsics ────────────────────────────────────────────

    pub(super) fn emit_intrinsic(
        &mut self,
        name: &str,
        args: &[TypedExpression],
        result_ty: &Type,
    ) -> String {
        match name {
            // -- Memory allocation ----------------------------------------
            "raw_allocate" => {
                let size = self.emit_expr_inline(&args[0]);
                format!("malloc({})", size)
            }
            "raw_deallocate" => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("free({})", ptr)
            }
            "raw_reallocate" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let new_size = self.emit_expr_inline(&args[2]);
                format!("realloc({}, {})", ptr, new_size)
            }

            // -- Memory operations ----------------------------------------
            "memcpy" => {
                let dest = self.emit_expr_inline(&args[0]);
                let src = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memcpy({}, {}, {})", dest, src, n)
            }
            "memmove" => {
                let dest = self.emit_expr_inline(&args[0]);
                let src = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memmove({}, {}, {})", dest, src, n)
            }
            "memset" => {
                let dest = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memset({}, {}, {})", dest, val, n)
            }
            "memcmp" => {
                let a = self.emit_expr_inline(&args[0]);
                let b = self.emit_expr_inline(&args[1]);
                let n = self.emit_expr_inline(&args[2]);
                format!("memcmp({}, {}, {})", a, b, n)
            }

            // -- Load / Store ---------------------------------------------
            "load" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let ty = self.c_type(result_ty);
                format!("(*(({}*)({})))", ty, ptr)
            }
            "store" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                let ty = self.c_type(&args[1].ty);
                format!("(*(({}*)({})) = ({}))", ty, ptr, val)
            }

            // -- Type introspection ---------------------------------------
            "sizeof" => {
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
            "alignof" => {
                if !args.is_empty() {
                    let ty = self.c_type(&args[0].ty);
                    format!("_Alignof({})", ty)
                } else {
                    self.line("#error \"alignof intrinsic reached codegen without type arg\"");
                    "0".into()
                }
            }

            // -- Pointer operations ---------------------------------------
            "int_to_ptr" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((void*)(uintptr_t)({}))", val)
            }
            "ptr_to_int" => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("((uintptr_t)({}))", ptr)
            }
            "gep" => {
                let base = self.emit_expr_inline(&args[0]);
                let offset = self.emit_expr_inline(&args[1]);
                format!("((uint8_t*)({}) + ({}))", base, offset)
            }
            "gep_struct" => {
                // Struct GEP: byte-offset into a struct by field index.
                // In C we just cast to uint8_t* and offset, since the
                // actual field layout matches.
                let ptr = self.emit_expr_inline(&args[0]);
                let idx = self.emit_expr_inline(&args[1]);
                format!("((uint8_t*)({}) + ({}))", ptr, idx)
            }
            "raw_ptr_offset" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let offset = self.emit_expr_inline(&args[1]);
                format!("((uint8_t*)({}) + ({}))", ptr, offset)
            }
            "raw_ptr_cast" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let ty = self.c_type(result_ty);
                format!("(({})({})", ty, ptr)
            }
            "null_ptr" | "nullptr" => "(NULL)".into(),
            "is_null" => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("(({}) == NULL)", ptr)
            }

            // -- Atomic operations ----------------------------------------
            "atomic_load" => {
                let ptr = self.emit_expr_inline(&args[0]);
                format!("__atomic_load_n({}, __ATOMIC_SEQ_CST)", ptr)
            }
            "atomic_store" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_store_n({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            "atomic_add" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_fetch_add({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            "atomic_sub" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_fetch_sub({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            "atomic_cas" => {
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
            "atomic_xchg" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let val = self.emit_expr_inline(&args[1]);
                format!("__atomic_exchange_n({}, {}, __ATOMIC_SEQ_CST)", ptr, val)
            }
            "fence" => "(__atomic_thread_fence(__ATOMIC_SEQ_CST), (void)0)".into(),

            // -- Syscalls -------------------------------------------------
            "syscall0" => {
                let num = self.emit_expr_inline(&args[0]);
                format!("syscall({})", num)
            }
            "syscall1" => {
                let num = self.emit_expr_inline(&args[0]);
                let a0 = self.emit_expr_inline(&args[1]);
                format!("syscall({}, {})", num, a0)
            }
            "syscall2" => {
                let num = self.emit_expr_inline(&args[0]);
                let a0 = self.emit_expr_inline(&args[1]);
                let a1 = self.emit_expr_inline(&args[2]);
                format!("syscall({}, {}, {})", num, a0, a1)
            }
            "syscall3" => {
                let num = self.emit_expr_inline(&args[0]);
                let a0 = self.emit_expr_inline(&args[1]);
                let a1 = self.emit_expr_inline(&args[2]);
                let a2 = self.emit_expr_inline(&args[3]);
                format!("syscall({}, {}, {}, {})", num, a0, a1, a2)
            }
            "syscall4" => {
                let num = self.emit_expr_inline(&args[0]);
                let a0 = self.emit_expr_inline(&args[1]);
                let a1 = self.emit_expr_inline(&args[2]);
                let a2 = self.emit_expr_inline(&args[3]);
                let a3 = self.emit_expr_inline(&args[4]);
                format!("syscall({}, {}, {}, {}, {})", num, a0, a1, a2, a3)
            }
            "syscall5" => {
                let num = self.emit_expr_inline(&args[0]);
                let a0 = self.emit_expr_inline(&args[1]);
                let a1 = self.emit_expr_inline(&args[2]);
                let a2 = self.emit_expr_inline(&args[3]);
                let a3 = self.emit_expr_inline(&args[4]);
                let a4 = self.emit_expr_inline(&args[5]);
                format!("syscall({}, {}, {}, {}, {}, {})", num, a0, a1, a2, a3, a4)
            }
            "syscall6" => {
                let num = self.emit_expr_inline(&args[0]);
                let a0 = self.emit_expr_inline(&args[1]);
                let a1 = self.emit_expr_inline(&args[2]);
                let a2 = self.emit_expr_inline(&args[3]);
                let a3 = self.emit_expr_inline(&args[4]);
                let a4 = self.emit_expr_inline(&args[5]);
                let a5 = self.emit_expr_inline(&args[6]);
                format!(
                    "syscall({}, {}, {}, {}, {}, {}, {})",
                    num, a0, a1, a2, a3, a4, a5
                )
            }

            // -- Debug / trap / panic -------------------------------------
            "trap" => "(__builtin_trap(), (void)0)".into(),
            "debugtrap" => "(__builtin_debugtrap(), (void)0)".into(),
            "unreachable" => "(__builtin_unreachable(), (void)0)".into(),
            "panic" => {
                let msg = self.emit_expr_inline(&args[0]);
                format!(
                    "(fprintf(stderr, \"panic: %.*s\\n\", (int)({msg}).len, ({msg}).ptr), abort(), (void)0)"
                )
            }

            // -- Bitwise operations ---------------------------------------
            "bswap16" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("__builtin_bswap16({})", val)
            }
            "bswap32" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("__builtin_bswap32({})", val)
            }
            "bswap64" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("__builtin_bswap64({})", val)
            }
            "ctlz" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((uint64_t)__builtin_clzll({}))", val)
            }
            "cttz" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((uint64_t)__builtin_ctzll({}))", val)
            }
            "ctpop" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((uint64_t)__builtin_popcountll({}))", val)
            }

            // -- Overflow-checked arithmetic ------------------------------
            "add_overflow" => self.emit_overflow_op("add", args),
            "sub_overflow" => self.emit_overflow_op("sub", args),
            "mul_overflow" => self.emit_overflow_op("mul", args),

            // -- Type conversions -----------------------------------------
            "trunc_f64_i64" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((int64_t)({}))", val)
            }
            "trunc_f32_i32" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((int32_t)({}))", val)
            }
            "sitofp_i64_f64" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((double)({}))", val)
            }
            "uitofp_u64_f64" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((double)({}))", val)
            }

            // -- IO (libc wrappers) ---------------------------------------
            "libc_write" => {
                let fd = self.emit_expr_inline(&args[0]);
                let buf = self.emit_expr_inline(&args[1]);
                let len = self.emit_expr_inline(&args[2]);
                format!("((int64_t)write({}, {}, {}))", fd, buf, len)
            }
            "libc_read" => {
                let fd = self.emit_expr_inline(&args[0]);
                let buf = self.emit_expr_inline(&args[1]);
                let len = self.emit_expr_inline(&args[2]);
                format!("((int64_t)read({}, {}, {}))", fd, buf, len)
            }

            // -- String operations ----------------------------------------
            "strlen" => {
                let s = self.emit_expr_inline(&args[0]);
                format!("strlen({})", s)
            }
            "static_string_ptr" => {
                let s = self.emit_expr_inline(&args[0]);
                format!("((uint8_t*)({}))", s)
            }

            // -- FFI / dynamic loading ------------------------------------
            "load_library" => {
                let path = self.emit_expr_inline(&args[0]);
                format!("dlopen({}, RTLD_LAZY)", path)
            }
            "get_symbol" => {
                let handle = self.emit_expr_inline(&args[0]);
                let sym = self.emit_expr_inline(&args[1]);
                format!("dlsym({}, {})", handle, sym)
            }
            "unload_library" => {
                let handle = self.emit_expr_inline(&args[0]);
                format!("dlclose({})", handle)
            }
            "dlerror" => "((uint8_t*)dlerror())".into(),
            "call_external" => {
                let fptr = self.emit_expr_inline(&args[0]);
                format!("((int64_t(*)(void))({}))()", fptr)
            }

            // -- Inline C -------------------------------------------------
            "inline_c" => {
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
            "discriminant" => {
                let val = self.emit_expr_inline(&args[0]);
                format!("((int32_t)(((int32_t*)({}))[0]))", val)
            }
            "set_discriminant" => {
                let ptr = self.emit_expr_inline(&args[0]);
                let disc = self.emit_expr_inline(&args[1]);
                format!("(((int32_t*)({})) [0] = ({}))", ptr, disc)
            }
            "get_payload" => {
                let val = self.emit_expr_inline(&args[0]);
                // Payload sits after the discriminant (4 bytes, aligned)
                format!("((uint8_t*)({}) + sizeof(int32_t))", val)
            }
            "set_payload" => {
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

            // -- Unknown --------------------------------------------------
            _ => {
                self.line(&format!("#error \"Unknown intrinsic: {}\"", name));
                "(void)0".into()
            }
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
