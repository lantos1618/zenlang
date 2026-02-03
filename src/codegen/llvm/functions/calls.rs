use crate::ast::{self, AstType};
use crate::codegen::llvm::stdlib_codegen;
use crate::codegen::llvm::{LLVMCompiler, Type};
use crate::error::CompileError;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use inkwell::AddressSpace;

// --- Function Type Building ---

fn build_fn_type_from_params<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    param_types: &[AstType],
    return_type: &AstType,
) -> Result<FunctionType<'ctx>, CompileError> {
    let span = compiler.get_current_span();
    let param_types_basic: Result<Vec<_>, _> = param_types
        .iter()
        .map(|ty| {
            let llvm_ty = compiler.to_llvm_type(ty)?;
            match llvm_ty {
                Type::Basic(b) => Ok(b),
                Type::Struct(s) => Ok(s.into()),
                _ => Err(CompileError::InternalError(
                    format!("Unsupported function argument type: {:?}", ty),
                    span.clone(),
                )),
            }
        })
        .collect();
    let param_metadata: Vec<BasicMetadataTypeEnum> =
        param_types_basic?.iter().map(|ty| (*ty).into()).collect();
    let ret_type = compiler.to_llvm_type(return_type)?;
    build_fn_type_from_ret(compiler, ret_type, &param_metadata)
}

pub fn build_fn_type_from_ret<'ctx>(
    compiler: &LLVMCompiler<'ctx>,
    ret_type: Type<'ctx>,
    param_metadata: &[BasicMetadataTypeEnum<'ctx>],
) -> Result<FunctionType<'ctx>, CompileError> {
    match ret_type {
        Type::Basic(b) => Ok(match b {
            BasicTypeEnum::IntType(t) => t.fn_type(param_metadata, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(param_metadata, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(param_metadata, false),
            BasicTypeEnum::StructType(t) => t.fn_type(param_metadata, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(param_metadata, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(param_metadata, false),
            BasicTypeEnum::ScalableVectorType(t) => t.fn_type(param_metadata, false),
        }),
        Type::Struct(s) => Ok(s.fn_type(param_metadata, false)),
        Type::Void => Ok(compiler.context.void_type().fn_type(param_metadata, false)),
        _ => Err(CompileError::InternalError(
            "Function return type must be a basic type, struct or void".to_string(),
            compiler.get_current_span(),
        )),
    }
}

// --- Argument Compilation ---

fn compile_and_convert_args<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
    param_types: &[BasicMetadataTypeEnum<'ctx>],
) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, CompileError> {
    let mut compiled_args = Vec::with_capacity(args.len());
    for (i, arg) in args.iter().enumerate() {
        let mut val = compiler.compile_expression(arg)?;
        if i < param_types.len() {
            val = maybe_convert_ptr_to_string_struct(compiler, val, param_types[i])?;
            val = maybe_cast_int_arg(compiler, val, param_types[i])?;
        }
        compiled_args.push(val);
    }
    compiled_args
        .iter()
        .map(|arg| Ok(BasicMetadataValueEnum::from(*arg)))
        .collect()
}

fn maybe_cast_int_arg<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    val: BasicValueEnum<'ctx>,
    expected_type: BasicMetadataTypeEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if let (true, BasicMetadataTypeEnum::IntType(expected)) = (val.is_int_value(), expected_type) {
        let int_val = val.into_int_value();
        let (src, dst) = (int_val.get_type().get_bit_width(), expected.get_bit_width());
        if src != dst {
            return Ok(if src < dst {
                compiler
                    .builder
                    .build_int_s_extend(int_val, expected, "extend")?
                    .into()
            } else {
                compiler
                    .builder
                    .build_int_truncate(int_val, expected, "trunc")?
                    .into()
            });
        }
    }
    Ok(val)
}

// --- Function Type Extraction ---

fn get_function_type_from_ast<'a, 'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    var_type: &'a AstType,
) -> Result<(FunctionType<'ctx>, Option<&'a AstType>), CompileError> {
    match var_type {
        AstType::Function {
            args: param_types,
            return_type,
        }
        | AstType::FunctionPointer {
            param_types,
            return_type,
        } => {
            let fn_type = build_fn_type_from_params(compiler, param_types, return_type)?;
            Ok((fn_type, Some(return_type.as_ref())))
        }
        t if t.is_ptr_type()
            && t.ptr_inner()
                .map(|i| matches!(i, AstType::FunctionPointer { .. }))
                .unwrap_or(false) =>
        {
            if let Some(AstType::FunctionPointer {
                param_types,
                return_type,
            }) = t.ptr_inner()
            {
                let fn_type = build_fn_type_from_params(compiler, param_types, return_type)?;
                Ok((fn_type, Some(return_type.as_ref())))
            } else {
                Err(CompileError::InternalError(
                    "Expected function pointer type in pointer".to_string(),
                    compiler.current_span.clone(),
                ))
            }
        }
        // Handle type alias references (e.g., CompletionFn -> FunctionPointer)
        AstType::Generic { name, type_args } if type_args.is_empty() => {
            // Clone to avoid borrow conflict
            let aliased_type = compiler.type_ctx.type_aliases.get(name).cloned();
            if let Some(aliased) = aliased_type {
                match aliased {
                    AstType::Function {
                        args: param_types,
                        return_type,
                    }
                    | AstType::FunctionPointer {
                        param_types,
                        return_type,
                    } => {
                        let fn_type =
                            build_fn_type_from_params(compiler, &param_types, &return_type)?;
                        // Return None for lifetime safety - caller should look up return type separately if needed
                        Ok((fn_type, None))
                    }
                    _ => Err(CompileError::TypeMismatch {
                        expected: "function pointer".to_string(),
                        found: format!("{:?} (resolved from type alias {})", aliased, name),
                        span: compiler.current_span.clone(),
                    }),
                }
            } else {
                Err(CompileError::TypeMismatch {
                    expected: "function pointer".to_string(),
                    found: format!("{:?}", var_type),
                    span: compiler.current_span.clone(),
                })
            }
        }
        _ => Err(CompileError::TypeMismatch {
            expected: "function pointer".to_string(),
            found: format!("{:?}", var_type),
            span: compiler.current_span.clone(),
        }),
    }
}

fn track_generic_return_type(compiler: &mut LLVMCompiler, return_type: &AstType) {
    if let AstType::Generic { name, type_args } = return_type {
        if compiler.well_known.is_result(name) && type_args.len() == 2 {
            compiler.track_generic_type("Result_Ok_Type".to_string(), type_args[0].clone());
            compiler.track_generic_type("Result_Err_Type".to_string(), type_args[1].clone());
        } else if compiler.well_known.is_option(name) && type_args.len() == 1 {
            compiler.track_generic_type("Option_Some_Type".to_string(), type_args[0].clone());
        }
    }
}

// NOTE: Range constructors (Range.new, Range.with_step) are now in stdlib
// See stdlib/core/iterator.zen - no magic codegen needed

// --- Compiler Intrinsics Dispatcher ---

fn try_dispatch_compiler_intrinsic<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    module: &str,
    func: &str,
    args: &[ast::Expression],
) -> Option<Result<BasicValueEnum<'ctx>, CompileError>> {
    match module {
        "compiler" | "builtin" | "@builtin" => dispatch_compiler_function(compiler, func, args),
        // NOTE: "io" module is now implemented in stdlib/io/io.zen using intrinsics
        // The magic dispatch has been removed - io.* functions are now real Zen functions
        _ => None,
    }
}

fn dispatch_compiler_function<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    func: &str,
    args: &[ast::Expression],
) -> Option<Result<BasicValueEnum<'ctx>, CompileError>> {
    let base_func = func.split('<').next().unwrap_or(func);
    Some(match base_func {
        "inline_c" => stdlib_codegen::compile_inline_c(compiler, args),
        "raw_allocate" => stdlib_codegen::compile_raw_allocate(compiler, args),
        "raw_deallocate" => stdlib_codegen::compile_raw_deallocate(compiler, args),
        "raw_reallocate" => stdlib_codegen::compile_raw_reallocate(compiler, args),
        "raw_ptr_offset" => stdlib_codegen::compile_raw_ptr_offset(compiler, args),
        "raw_ptr_cast" => stdlib_codegen::compile_raw_ptr_cast(compiler, args),
        "null_ptr" | "nullptr" => stdlib_codegen::compile_null_ptr(compiler, args),
        "is_null" => stdlib_codegen::compile_is_null(compiler, args),
        "ptr_to_int" => stdlib_codegen::compile_ptr_to_int(compiler, args),
        "int_to_ptr" => stdlib_codegen::compile_int_to_ptr(compiler, args),
        "call_external" => stdlib_codegen::compile_call_external(compiler, args),
        "load_library" => stdlib_codegen::compile_load_library(compiler, args),
        "get_symbol" => stdlib_codegen::compile_get_symbol(compiler, args),
        "unload_library" => stdlib_codegen::compile_unload_library(compiler, args),
        "dlerror" => stdlib_codegen::compile_dlerror(compiler, args),
        "discriminant" => stdlib_codegen::compile_discriminant(compiler, args),
        "set_discriminant" => stdlib_codegen::compile_set_discriminant(compiler, args),
        "get_payload" => stdlib_codegen::compile_get_payload(compiler, args),
        "set_payload" => stdlib_codegen::compile_set_payload(compiler, args),
        "gep" => stdlib_codegen::compile_gep(compiler, args),
        "gep_struct" => stdlib_codegen::compile_gep_struct(compiler, args),
        "load" => {
            let type_arg = func.find('<').and_then(|pos| {
                crate::parser::parse_type_from_string(&func[pos + 1..func.len() - 1]).ok()
            });
            stdlib_codegen::compile_load(compiler, args, type_arg.as_ref())
        }
        "store" => {
            let type_arg = func.find('<').and_then(|pos| {
                crate::parser::parse_type_from_string(&func[pos + 1..func.len() - 1]).ok()
            });
            stdlib_codegen::compile_store(compiler, args, type_arg.as_ref())
        }
        "sizeof" => {
            let type_arg = func.find('<').and_then(|pos| {
                crate::parser::parse_type_from_string(&func[pos + 1..func.len() - 1]).ok()
            });
            stdlib_codegen::compile_sizeof(compiler, type_arg.as_ref())
        }
        "memset" => stdlib_codegen::compile_memset(compiler, args),
        "memcpy" => stdlib_codegen::compile_memcpy(compiler, args),
        "memmove" => stdlib_codegen::compile_memmove(compiler, args),
        "memcmp" => stdlib_codegen::compile_memcmp(compiler, args),
        "bswap16" => stdlib_codegen::compile_bswap16(compiler, args),
        "bswap32" => stdlib_codegen::compile_bswap32(compiler, args),
        "bswap64" => stdlib_codegen::compile_bswap64(compiler, args),
        "ctlz" => stdlib_codegen::compile_ctlz(compiler, args),
        "cttz" => stdlib_codegen::compile_cttz(compiler, args),
        "ctpop" => stdlib_codegen::compile_ctpop(compiler, args),
        "syscall0" => stdlib_codegen::compile_syscall0(compiler, args),
        "syscall1" => stdlib_codegen::compile_syscall1(compiler, args),
        "syscall2" => stdlib_codegen::compile_syscall2(compiler, args),
        "syscall3" => stdlib_codegen::compile_syscall3(compiler, args),
        "syscall4" => stdlib_codegen::compile_syscall4(compiler, args),
        "syscall5" => stdlib_codegen::compile_syscall5(compiler, args),
        "syscall6" => stdlib_codegen::compile_syscall6(compiler, args),
        "panic" => stdlib_codegen::compile_panic(compiler, args),
        // IO intrinsics (libc wrappers)
        "libc_write" => stdlib_codegen::compile_libc_write(compiler, args),
        "libc_read" => stdlib_codegen::compile_libc_read(compiler, args),
        _ => return None,
    })
}

// --- Direct and Indirect Calls ---

fn try_compile_direct_call<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    name: &str,
    args: &[ast::Expression],
) -> Result<Option<BasicValueEnum<'ctx>>, CompileError> {
    let Some(function) = compiler.module.get_function(name) else {
        return Ok(None);
    };
    let param_types = function.get_type().get_param_types();
    let args_metadata = compile_and_convert_args(compiler, args, &param_types)?;
    let call = compiler
        .builder
        .build_call(function, &args_metadata, "calltmp")?;

    // Check TypeContext first, then local cache
    let return_type = compiler
        .type_ctx
        .get_function_return_type(name)
        .or_else(|| compiler.function_types.get(name).cloned());
    if let Some(return_type) = return_type {
        track_generic_return_type(compiler, &return_type);
    }
    if function.get_type().get_return_type().is_none() {
        Ok(Some(compiler.context.i32_type().const_zero().into()))
    } else {
        Ok(Some(call.try_as_basic_value().left().ok_or_else(|| {
            CompileError::InternalError(
                "Function call did not return a value".to_string(),
                compiler.get_current_span(),
            )
        })?))
    }
}

fn try_compile_indirect_call<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    name: &str,
    args: &[ast::Expression],
) -> Result<Option<BasicValueEnum<'ctx>>, CompileError> {
    let Ok((alloca, var_type)) = compiler.get_variable(name) else {
        return Ok(None);
    };

    let function_ptr = compiler
        .builder
        .build_load(alloca.get_type(), alloca, "func_ptr")?;
    let (function_type, return_type_ref) = get_function_type_from_ast(compiler, &var_type)?;
    let args_metadata = compile_and_convert_args(compiler, args, &function_type.get_param_types())?;

    let casted_ptr = compiler.builder.build_pointer_cast(
        function_ptr.into_pointer_value(),
        compiler.context.ptr_type(AddressSpace::default()),
        "casted_func_ptr",
    )?;
    let call = compiler.builder.build_indirect_call(
        function_type,
        casted_ptr,
        &args_metadata,
        "indirect_call",
    )?;

    if let Some(ret_type) = return_type_ref {
        track_generic_return_type(compiler, ret_type);
    }
    if function_type.get_return_type().is_none() {
        Ok(Some(compiler.context.i32_type().const_zero().into()))
    } else {
        Ok(Some(call.try_as_basic_value().left().ok_or_else(|| {
            CompileError::InternalError(
                "Function call did not return a value".to_string(),
                compiler.get_current_span(),
            )
        })?))
    }
}

// --- Main Entry Point ---

pub fn compile_function_call<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    name: &str,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // NOTE: Collection constructors (Range.new, HashMap.new, etc.) are now in stdlib
    // They use the normal function resolution path below
    if let Some((module, func)) = name.split_once('.') {
        // Only dispatch compiler intrinsics - stdlib (io/fs/core/math) is in Zen
        if let Some(result) = try_dispatch_compiler_intrinsic(compiler, module, func, args) {
            return result;
        }
        if compiler.well_known.is_result(module) || compiler.well_known.is_option(module) {
            let payload = args.first().map(|a| Box::new(a.clone()));
            return compiler.compile_enum_variant(module, func, &payload);
        }
        // Try stdlib module function: io.println -> println
        // Stdlib functions are compiled with their simple name, not qualified
        if let Some(result) = try_compile_direct_call(compiler, func, args)? {
            return Ok(result);
        }
    }
    if name == "cast" {
        return compile_cast_builtin(compiler, args);
    }
    if let Some(result) = try_compile_direct_call(compiler, name, args)? {
        return Ok(result);
    }
    if let Some(result) = try_compile_indirect_call(compiler, name, args)? {
        return Ok(result);
    }
    Err(CompileError::UndeclaredFunction(
        name.to_string(),
        compiler.get_current_span(),
    ))
}

// --- Cast Builtin ---

fn compile_cast_builtin<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if args.len() != 2 {
        return Err(CompileError::TypeError(
            format!(
                "cast() expects 2 arguments (value, type), got {}",
                args.len()
            ),
            compiler.get_current_span(),
        ));
    }
    let source_type =
        crate::codegen::llvm::expressions::inference::infer_expression_type(compiler, &args[0])
            .unwrap_or(AstType::I32);
    let value = compiler.compile_expression(&args[0])?;
    let target_type = parse_cast_target_type(&args[1], compiler.get_current_span())?;
    let llvm_target = compiler.to_llvm_type(&target_type)?;
    let Type::Basic(target_basic) = llvm_target else {
        return Err(CompileError::TypeError(
            "cast() target must be a basic type".to_string(),
            compiler.get_current_span(),
        ));
    };
    perform_cast(compiler, value, target_basic, &source_type, &target_type)
}

fn parse_cast_target_type(
    expr: &ast::Expression,
    span: Option<crate::error::Span>,
) -> Result<AstType, CompileError> {
    let ast::Expression::Identifier(name) = expr else {
        return Err(CompileError::TypeError(
            "cast() second argument must be a type name (e.g., i32, f64)".to_string(),
            span.clone(),
        ));
    };

    // Use the centralized type parser instead of manual matching
    match crate::parser::parse_type_from_string(name) {
        Ok(ast_type) => {
            // Only allow numeric primitive types for cast()
            if ast_type.is_numeric() {
                Ok(ast_type)
            } else {
                Err(CompileError::TypeError(
                    format!("cast() target type '{}' is not a valid numeric type", name),
                    span,
                ))
            }
        }
        Err(e) => Err(CompileError::TypeError(
            format!("cast() target type '{}' is not a valid type: {}", name, e),
            span,
        )),
    }
}

fn perform_cast<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    value: BasicValueEnum<'ctx>,
    target: BasicTypeEnum<'ctx>,
    source_type: &AstType,
    target_type: &AstType,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match (value, target) {
        (BasicValueEnum::IntValue(v), BasicTypeEnum::IntType(t)) => {
            cast_int_to_int(compiler, v, t, source_type)
        }
        (BasicValueEnum::FloatValue(v), BasicTypeEnum::FloatType(t)) => {
            cast_float_to_float(compiler, v, t)
        }
        (BasicValueEnum::IntValue(v), BasicTypeEnum::FloatType(t)) => {
            Ok(if source_type.is_unsigned_integer() {
                compiler
                    .builder
                    .build_unsigned_int_to_float(v, t, "cast_uitofp")?
                    .into()
            } else {
                compiler
                    .builder
                    .build_signed_int_to_float(v, t, "cast_sitofp")?
                    .into()
            })
        }
        (BasicValueEnum::FloatValue(v), BasicTypeEnum::IntType(t)) => {
            Ok(if target_type.is_unsigned_integer() {
                compiler
                    .builder
                    .build_float_to_unsigned_int(v, t, "cast_fptoui")?
                    .into()
            } else {
                compiler
                    .builder
                    .build_float_to_signed_int(v, t, "cast_fptosi")?
                    .into()
            })
        }
        _ => Err(CompileError::TypeError(
            format!("Cannot cast {:?} to {:?}", value.get_type(), target),
            compiler.get_current_span(),
        )),
    }
}

fn cast_int_to_int<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    val: inkwell::values::IntValue<'ctx>,
    target: inkwell::types::IntType<'ctx>,
    source_type: &AstType,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let (src, dst) = (val.get_type().get_bit_width(), target.get_bit_width());
    Ok(if src == dst {
        val.into()
    } else if src < dst {
        if source_type.is_unsigned_integer() {
            compiler
                .builder
                .build_int_z_extend(val, target, "cast_zext")?
                .into()
        } else {
            compiler
                .builder
                .build_int_s_extend(val, target, "cast_sext")?
                .into()
        }
    } else {
        compiler
            .builder
            .build_int_truncate(val, target, "cast_trunc")?
            .into()
    })
}

fn cast_float_to_float<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    val: inkwell::values::FloatValue<'ctx>,
    target: inkwell::types::FloatType<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let src = if val.get_type() == compiler.context.f32_type() {
        32
    } else {
        64
    };
    let dst = if target == compiler.context.f32_type() {
        32
    } else {
        64
    };
    Ok(if src == dst {
        val.into()
    } else if src < dst {
        compiler
            .builder
            .build_float_ext(val, target, "cast_fext")?
            .into()
    } else {
        compiler
            .builder
            .build_float_trunc(val, target, "cast_ftrunc")?
            .into()
    })
}

// --- String Conversion ---

fn metadata_to_basic_type(ty: BasicMetadataTypeEnum) -> Option<BasicTypeEnum> {
    Some(match ty {
        BasicMetadataTypeEnum::ArrayType(t) => t.into(),
        BasicMetadataTypeEnum::FloatType(t) => t.into(),
        BasicMetadataTypeEnum::IntType(t) => t.into(),
        BasicMetadataTypeEnum::PointerType(t) => t.into(),
        BasicMetadataTypeEnum::StructType(t) => t.into(),
        BasicMetadataTypeEnum::VectorType(t) => t.into(),
        BasicMetadataTypeEnum::ScalableVectorType(t) => t.into(),
        BasicMetadataTypeEnum::MetadataType(_) => return None,
    })
}

fn maybe_convert_ptr_to_string_struct<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    val: BasicValueEnum<'ctx>,
    expected_type: BasicMetadataTypeEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    if !val.is_pointer_value() {
        return Ok(val);
    }
    let Some(basic_type) = metadata_to_basic_type(expected_type) else {
        return Ok(val);
    };
    if !basic_type.is_struct_type() {
        return Ok(val);
    }
    let struct_type = basic_type.into_struct_type();
    if !is_string_struct_type(compiler, struct_type) {
        return Ok(val);
    }
    build_string_struct_from_ptr(compiler, val.into_pointer_value(), struct_type)
}

fn is_string_struct_type<'ctx>(
    compiler: &LLVMCompiler<'ctx>,
    struct_type: inkwell::types::StructType<'ctx>,
) -> bool {
    if let Some(info) = compiler.struct_types.get("String") {
        if info.llvm_type == struct_type {
            return true;
        }
    }
    // Fallback: check struct layout (4 fields: ptr, i64, i64, ptr)
    if struct_type.count_fields() == 4 {
        let f = struct_type.get_field_types();
        return f.len() == 4
            && f[0].is_pointer_type()
            && f[1].is_int_type()
            && f[2].is_int_type()
            && f[3].is_pointer_type();
    }
    false
}

fn build_string_struct_from_ptr<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    ptr_val: inkwell::values::PointerValue<'ctx>,
    struct_type: inkwell::types::StructType<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let strlen_fn = compiler.module.get_function("strlen").unwrap_or_else(|| {
        let fn_type = compiler.context.i64_type().fn_type(
            &[compiler.context.ptr_type(AddressSpace::default()).into()],
            false,
        );
        compiler.module.add_function("strlen", fn_type, None)
    });

    let len_val = compiler
        .builder
        .build_call(strlen_fn, &[ptr_val.into()], "str_len")?
        .try_as_basic_value()
        .left()
        .ok_or_else(|| {
            CompileError::InternalError(
                "strlen should return a value".to_string(),
                compiler.get_current_span(),
            )
        })?
        .into_int_value();

    let null_ptr = compiler
        .context
        .ptr_type(AddressSpace::default())
        .const_null();
    let alloca = compiler
        .builder
        .build_alloca(struct_type, "string_struct")?;

    // Populate struct fields: { data, len, capacity, allocator }
    let values: [BasicValueEnum; 4] = [
        ptr_val.into(),
        len_val.into(),
        len_val.into(),
        null_ptr.into(),
    ];
    for (idx, value) in values.into_iter().enumerate() {
        let field_ptr = compiler
            .builder
            .build_struct_gep(struct_type, alloca, idx as u32, "")?;
        compiler.builder.build_store(field_ptr, value)?;
    }
    Ok(compiler
        .builder
        .build_load(struct_type, alloca, "string_val")?)
}
