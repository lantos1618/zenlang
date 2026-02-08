//! Compiler intrinsics codegen - true LLVM primitives that must be in Rust

use crate::ast::{self, AstType};
use crate::codegen::llvm::{LLVMCompiler, Type};
use crate::error::CompileError;
use inkwell::intrinsics::Intrinsic;
use inkwell::module::Linkage;
use inkwell::types::IntType;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::AddressSpace;

// =============================================================================
// Helpers
// =============================================================================

/// Convert any integer to i64, extending or truncating as needed
fn to_i64<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    val: BasicValueEnum<'ctx>,
    signed: bool,
) -> Result<IntValue<'ctx>, CompileError> {
    if !val.is_int_value() {
        return Err(CompileError::TypeError(
            "Expected integer".to_string(),
            compiler.get_current_span(),
        ));
    }
    let int_val = val.into_int_value();
    let bits = int_val.get_type().get_bit_width();
    let i64_type = compiler.context.i64_type();
    Ok(if bits == 64 {
        int_val
    } else if bits < 64 {
        if signed {
            compiler
                .builder
                .build_int_s_extend(int_val, i64_type, "sext")?
        } else {
            compiler
                .builder
                .build_int_z_extend(int_val, i64_type, "zext")?
        }
    } else {
        compiler
            .builder
            .build_int_truncate(int_val, i64_type, "trunc")?
    })
}

/// Convert integer to specific bit width
fn to_int_width<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    val: IntValue<'ctx>,
    target: IntType<'ctx>,
    signed: bool,
) -> Result<IntValue<'ctx>, CompileError> {
    let src_bits = val.get_type().get_bit_width();
    let dst_bits = target.get_bit_width();
    Ok(if src_bits == dst_bits {
        val
    } else if src_bits < dst_bits {
        if signed {
            compiler.builder.build_int_s_extend(val, target, "sext")?
        } else {
            compiler.builder.build_int_z_extend(val, target, "zext")?
        }
    } else {
        compiler.builder.build_int_truncate(val, target, "trunc")?
    })
}

/// Safely extract return value from a function call
/// Use this instead of `.try_as_basic_value().left().unwrap()`
fn extract_call_result<'ctx>(
    call_result: inkwell::values::CallSiteValue<'ctx>,
    func_name: &str,
    compiler: &LLVMCompiler<'ctx>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    call_result.try_as_basic_value().left().ok_or_else(|| {
        CompileError::InternalError(
            format!("Call to '{}' must return a value", func_name),
            compiler.get_current_span(),
        )
    })
}

/// Safely convert a BasicValueEnum to PointerValue with descriptive error
fn as_ptr<'ctx>(
    val: BasicValueEnum<'ctx>,
    context: &str,
    compiler: &LLVMCompiler<'ctx>,
) -> Result<PointerValue<'ctx>, CompileError> {
    if val.is_pointer_value() {
        Ok(val.into_pointer_value())
    } else {
        Err(CompileError::TypeError(
            format!(
                "Expected pointer value for {}, got {:?}",
                context,
                val.get_type()
            ),
            compiler.get_current_span(),
        ))
    }
}

/// Safely convert a BasicValueEnum to IntValue with descriptive error
fn as_int<'ctx>(
    val: BasicValueEnum<'ctx>,
    context: &str,
    compiler: &LLVMCompiler<'ctx>,
) -> Result<IntValue<'ctx>, CompileError> {
    if val.is_int_value() {
        Ok(val.into_int_value())
    } else {
        Err(CompileError::TypeError(
            format!(
                "Expected integer value for {}, got {:?}",
                context,
                val.get_type()
            ),
            compiler.get_current_span(),
        ))
    }
}

/// Get or declare a libc function
fn get_or_declare_fn<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    name: &str,
    ret_type: Option<inkwell::types::BasicTypeEnum<'ctx>>,
    param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
) -> FunctionValue<'ctx> {
    compiler.module.get_function(name).unwrap_or_else(|| {
        let fn_type = match ret_type {
            Some(t) => match t {
                inkwell::types::BasicTypeEnum::IntType(i) => i.fn_type(param_types, false),
                inkwell::types::BasicTypeEnum::PointerType(p) => p.fn_type(param_types, false),
                _ => compiler.context.void_type().fn_type(param_types, false),
            },
            None => compiler.context.void_type().fn_type(param_types, false),
        };
        compiler
            .module
            .add_function(name, fn_type, Some(Linkage::External))
    })
}

/// Call an LLVM intrinsic (bswap, ctlz, cttz, ctpop)
fn call_int_intrinsic<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    name: &str,
    val: IntValue<'ctx>,
    extra_args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let span = compiler.get_current_span();
    let intrinsic = Intrinsic::find(name).ok_or_else(|| {
        CompileError::InternalError(format!("{} intrinsic not found", name), span.clone())
    })?;
    let int_type = val.get_type();
    let intrinsic_fn = intrinsic
        .get_declaration(&compiler.module, &[int_type.into()])
        .ok_or_else(|| {
            CompileError::InternalError(format!("Failed to get {} declaration", name), span.clone())
        })?;

    let mut args: Vec<BasicValueEnum> = vec![val.into()];
    args.extend_from_slice(extra_args);
    let args_meta: Vec<_> = args.iter().map(|a| (*a).into()).collect();

    compiler
        .builder
        .build_call(intrinsic_fn, &args_meta, "intrinsic")?
        .try_as_basic_value()
        .left()
        .ok_or_else(|| {
            CompileError::InternalError("Intrinsic should return value".to_string(), span)
        })
}

/// Extract data pointer from String struct or return pointer directly
fn extract_string_ptr<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    value: BasicValueEnum<'ctx>,
) -> Result<PointerValue<'ctx>, CompileError> {
    let span = compiler.get_current_span();
    match value {
        BasicValueEnum::PointerValue(ptr) => Ok(ptr),
        BasicValueEnum::StructValue(s) => {
            let ptr = compiler.builder.build_extract_value(s, 0, "str_data")?;
            match ptr {
                BasicValueEnum::PointerValue(p) => Ok(p),
                _ => Err(CompileError::InternalError(
                    "String field 0 not a pointer".to_string(),
                    span,
                )),
            }
        }
        _ => Err(CompileError::InternalError(
            format!("Expected String, got {:?}", value.get_type()),
            span,
        )),
    }
}

fn ptr_type<'ctx>(compiler: &LLVMCompiler<'ctx>) -> inkwell::types::PointerType<'ctx> {
    compiler.context.ptr_type(AddressSpace::default())
}

#[allow(dead_code)]
fn require_args(
    args: &[ast::Expression],
    expected: usize,
    name: &str,
    span: Option<crate::error::Span>,
) -> Result<(), CompileError> {
    if args.len() != expected {
        Err(CompileError::TypeError(
            format!("{} expects {} args, got {}", name, expected, args.len()),
            span,
        ))
    } else {
        Ok(())
    }
}

// =============================================================================
// Memory Allocation (libc wrappers)
// =============================================================================

pub fn compile_raw_allocate<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let size_val = compiler.compile_expression(&args[0])?;
    let size = to_i64(compiler, size_val, false)?;
    let malloc = get_or_declare_fn(
        compiler,
        "malloc",
        Some(ptr_type(compiler).into()),
        &[compiler.context.i64_type().into()],
    );
    let call = compiler.builder.build_call(malloc, &[size.into()], "ptr")?;
    extract_call_result(call, "malloc", compiler)
}

pub fn compile_raw_deallocate<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "raw_deallocate arg 0",
        compiler,
    )?;
    let _size = compiler.compile_expression(&args[1])?;
    let free = get_or_declare_fn(compiler, "free", None, &[ptr_type(compiler).into()]);
    compiler.builder.build_call(free, &[ptr.into()], "")?;
    Ok(compiler.context.i32_type().const_zero().into())
}

pub fn compile_raw_reallocate<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = compiler.compile_expression(&args[0])?;
    let _old = compiler.compile_expression(&args[1])?;
    let new_size_val = compiler.compile_expression(&args[2])?;
    let new_size = to_i64(compiler, new_size_val, false)?;
    let realloc = get_or_declare_fn(
        compiler,
        "realloc",
        Some(ptr_type(compiler).into()),
        &[
            ptr_type(compiler).into(),
            compiler.context.i64_type().into(),
        ],
    );
    let call = compiler
        .builder
        .build_call(realloc, &[ptr.into(), new_size.into()], "ptr")?;
    extract_call_result(call, "realloc", compiler)
}

// =============================================================================
// Pointer Operations
// =============================================================================

pub fn compile_raw_ptr_offset<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "raw_ptr_offset arg 0",
        compiler,
    )?;
    let offset_val = compiler.compile_expression(&args[1])?;
    let offset = to_i64(compiler, offset_val, true)?;
    let result = unsafe {
        compiler
            .builder
            .build_gep(compiler.context.i8_type(), ptr, &[offset], "offset")?
    };
    Ok(result.into())
}

pub fn compile_raw_ptr_cast<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    compiler.compile_expression(&args[0]) // No-op at LLVM level
}

pub fn compile_null_ptr<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    _args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    Ok(ptr_type(compiler).const_null().into())
}

pub fn compile_is_null<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "is_null arg 0",
        compiler,
    )?;
    let null = ptr_type(compiler).const_null();
    Ok(compiler
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, ptr, null, "is_null")?
        .into())
}

pub fn compile_ptr_to_int<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "ptr_to_int arg 0",
        compiler,
    )?;
    Ok(compiler
        .builder
        .build_ptr_to_int(ptr, compiler.ptr_sized_int_type(), "p2i")?
        .into())
}

pub fn compile_int_to_ptr<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let addr = as_int(
        compiler.compile_expression(&args[0])?,
        "int_to_ptr arg 0",
        compiler,
    )?;
    Ok(compiler
        .builder
        .build_int_to_ptr(addr, ptr_type(compiler), "i2p")?
        .into())
}

// =============================================================================
// Dynamic Library Loading
// =============================================================================

pub fn compile_load_library<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let path_val = compiler.compile_expression(&args[0])?;
    let path = extract_string_ptr(compiler, path_val)?;
    let dlopen = get_or_declare_fn(
        compiler,
        "dlopen",
        Some(ptr_type(compiler).into()),
        &[
            ptr_type(compiler).into(),
            compiler.context.i32_type().into(),
        ],
    );
    let rtld_lazy = compiler.context.i32_type().const_int(1, false);
    let call = compiler
        .builder
        .build_call(dlopen, &[path.into(), rtld_lazy.into()], "handle")?;
    extract_call_result(call, "dlopen", compiler)
}

pub fn compile_get_symbol<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let handle = compiler.compile_expression(&args[0])?;
    let name_val = compiler.compile_expression(&args[1])?;
    let name = extract_string_ptr(compiler, name_val)?;
    let dlsym = get_or_declare_fn(
        compiler,
        "dlsym",
        Some(ptr_type(compiler).into()),
        &[ptr_type(compiler).into(), ptr_type(compiler).into()],
    );
    let call = compiler
        .builder
        .build_call(dlsym, &[handle.into(), name.into()], "sym")?;
    extract_call_result(call, "dlsym", compiler)
}

pub fn compile_unload_library<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let handle = compiler.compile_expression(&args[0])?;
    let dlclose = get_or_declare_fn(
        compiler,
        "dlclose",
        Some(compiler.context.i32_type().into()),
        &[ptr_type(compiler).into()],
    );
    let call = compiler
        .builder
        .build_call(dlclose, &[handle.into()], "result")?;
    extract_call_result(call, "dlclose", compiler)
}

pub fn compile_dlerror<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    _args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let dlerror = get_or_declare_fn(compiler, "dlerror", Some(ptr_type(compiler).into()), &[]);
    let call = compiler.builder.build_call(dlerror, &[], "err")?;
    extract_call_result(call, "dlerror", compiler)
}

// =============================================================================
// Enum Introspection
// =============================================================================

pub fn compile_discriminant<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "discriminant arg 0",
        compiler,
    )?;
    Ok(compiler
        .builder
        .build_load(compiler.context.i32_type(), ptr, "disc")?)
}

pub fn compile_set_discriminant<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "set_discriminant arg 0",
        compiler,
    )?;
    let disc = compiler.compile_expression(&args[1])?;
    compiler.builder.build_store(ptr, disc)?;
    Ok(compiler.context.i32_type().const_zero().into())
}

pub fn compile_get_payload<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "get_payload arg 0",
        compiler,
    )?;
    // Payload after 4-byte discriminant
    let offset = compiler.context.i32_type().const_int(4, false);
    let payload = unsafe {
        compiler
            .builder
            .build_gep(compiler.context.i8_type(), ptr, &[offset], "payload")?
    };
    Ok(payload.into())
}

pub fn compile_set_payload<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let _ptr = compiler.compile_expression(&args[0])?;
    let _payload = compiler.compile_expression(&args[1])?;
    // Stub: full implementation needs payload size for memcpy
    Ok(compiler.context.i32_type().const_zero().into())
}

// =============================================================================
// GEP / Load / Store
// =============================================================================

pub fn compile_gep<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "gep arg 0",
        compiler,
    )?;
    let offset_val = compiler.compile_expression(&args[1])?;
    let offset = to_i64(compiler, offset_val, true)?;
    let result = unsafe {
        compiler
            .builder
            .build_gep(compiler.context.i8_type(), ptr, &[offset], "gep")?
    };
    Ok(result.into())
}

pub fn compile_gep_struct<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "gep_struct arg 0",
        compiler,
    )?;
    let idx = as_int(
        compiler.compile_expression(&args[1])?,
        "gep_struct arg 1",
        compiler,
    )?;
    // Approximate: field_index * 8 bytes
    let idx_i32 = to_int_width(compiler, idx, compiler.context.i32_type(), false)?;
    let offset = compiler.builder.build_int_mul(
        idx_i32,
        compiler.context.i32_type().const_int(8, false),
        "off",
    )?;
    let offset_i64 = to_int_width(compiler, offset, compiler.context.i64_type(), true)?;
    let result = unsafe {
        compiler
            .builder
            .build_gep(compiler.context.i8_type(), ptr, &[offset_i64], "gep_s")?
    };
    Ok(result.into())
}

pub fn compile_load<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
    type_arg: Option<&AstType>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "load arg 0",
        compiler,
    )?;
    let load_type = match type_arg {
        Some(ty) => compiler.to_llvm_type(ty)?,
        None => Type::Basic(compiler.context.i32_type().into()),
    };
    let basic = match load_type {
        Type::Basic(b) => b,
        _ => {
            return Err(CompileError::TypeError(
                "load: need basic type".to_string(),
                compiler.get_current_span(),
            ))
        }
    };
    Ok(compiler.builder.build_load(basic, ptr, "val")?)
}

pub fn compile_store<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
    _type_arg: Option<&AstType>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let ptr = as_ptr(
        compiler.compile_expression(&args[0])?,
        "store arg 0",
        compiler,
    )?;
    let val = compiler.compile_expression(&args[1])?;
    compiler.builder.build_store(ptr, val)?;
    Ok(compiler.context.i32_type().const_zero().into())
}

// =============================================================================
// Sizeof
// =============================================================================

pub fn compile_sizeof<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    type_arg: Option<&AstType>,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let size: u64 = match type_arg {
        Some(ty) => ty.byte_size().map(|s| s as u64).unwrap_or(match ty {
            t if t.is_ptr_type() => 8,
            AstType::Struct { fields, .. } => fields
                .iter()
                .map(|(_, ft)| ft.byte_size().unwrap_or(8) as u64)
                .sum(),
            _ => 8,
        }),
        None => 8,
    };
    Ok(compiler.context.i64_type().const_int(size, false).into())
}

// =============================================================================
// Memory Operations (libc)
// =============================================================================

pub fn compile_memset<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let dest = compiler.compile_expression(&args[0])?;
    let val_expr = as_int(
        compiler.compile_expression(&args[1])?,
        "memset arg 1",
        compiler,
    )?;
    let val = to_int_width(compiler, val_expr, compiler.context.i8_type(), false)?;
    let size_val = compiler.compile_expression(&args[2])?;
    let size = to_i64(compiler, size_val, false)?;
    let memset = get_or_declare_fn(
        compiler,
        "memset",
        Some(ptr_type(compiler).into()),
        &[
            ptr_type(compiler).into(),
            compiler.context.i8_type().into(),
            compiler.context.i64_type().into(),
        ],
    );
    compiler
        .builder
        .build_call(memset, &[dest.into(), val.into(), size.into()], "")?;
    Ok(compiler.context.i32_type().const_zero().into())
}

pub fn compile_memcpy<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let dest = compiler.compile_expression(&args[0])?;
    let src = compiler.compile_expression(&args[1])?;
    let size_val = compiler.compile_expression(&args[2])?;
    let size = to_i64(compiler, size_val, false)?;
    let memcpy = get_or_declare_fn(
        compiler,
        "memcpy",
        Some(ptr_type(compiler).into()),
        &[
            ptr_type(compiler).into(),
            ptr_type(compiler).into(),
            compiler.context.i64_type().into(),
        ],
    );
    compiler
        .builder
        .build_call(memcpy, &[dest.into(), src.into(), size.into()], "")?;
    Ok(compiler.context.i32_type().const_zero().into())
}

pub fn compile_memmove<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let dest = compiler.compile_expression(&args[0])?;
    let src = compiler.compile_expression(&args[1])?;
    let size_val = compiler.compile_expression(&args[2])?;
    let size = to_i64(compiler, size_val, false)?;
    let memmove = get_or_declare_fn(
        compiler,
        "memmove",
        Some(ptr_type(compiler).into()),
        &[
            ptr_type(compiler).into(),
            ptr_type(compiler).into(),
            compiler.context.i64_type().into(),
        ],
    );
    compiler
        .builder
        .build_call(memmove, &[dest.into(), src.into(), size.into()], "")?;
    Ok(compiler.context.i32_type().const_zero().into())
}

pub fn compile_memcmp<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let p1 = compiler.compile_expression(&args[0])?;
    let p2 = compiler.compile_expression(&args[1])?;
    let size_val = compiler.compile_expression(&args[2])?;
    let size = to_i64(compiler, size_val, false)?;
    let memcmp = get_or_declare_fn(
        compiler,
        "memcmp",
        Some(compiler.context.i32_type().into()),
        &[
            ptr_type(compiler).into(),
            ptr_type(compiler).into(),
            compiler.context.i64_type().into(),
        ],
    );
    let call = compiler
        .builder
        .build_call(memcmp, &[p1.into(), p2.into(), size.into()], "cmp")?;
    extract_call_result(call, "memcmp", compiler)
}

// =============================================================================
// Bitwise Intrinsics
// =============================================================================

fn compile_bswap<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
    bits: u32,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let span = compiler.get_current_span();
    let val = as_int(
        compiler.compile_expression(&args[0])?,
        &format!("bswap{} arg 0", bits),
        compiler,
    )?;
    let target_type = match bits {
        16 => compiler.context.i16_type(),
        32 => compiler.context.i32_type(),
        64 => compiler.context.i64_type(),
        _ => {
            return Err(CompileError::InternalError(
                "Invalid bswap width".to_string(),
                span,
            ))
        }
    };
    let converted = to_int_width(compiler, val, target_type, false)?;
    call_int_intrinsic(compiler, &format!("llvm.bswap.i{}", bits), converted, &[])
}

pub fn compile_bswap16<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    compile_bswap(compiler, args, 16)
}

pub fn compile_bswap32<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    compile_bswap(compiler, args, 32)
}

pub fn compile_bswap64<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    compile_bswap(compiler, args, 64)
}

fn compile_bit_count<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
    intrinsic_name: &str,
    has_zero_poison: bool,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let val = as_int(
        compiler.compile_expression(&args[0])?,
        intrinsic_name,
        compiler,
    )?;
    let val_i64 = to_int_width(compiler, val, compiler.context.i64_type(), false)?;
    let extra = if has_zero_poison {
        vec![compiler.context.bool_type().const_zero().into()]
    } else {
        vec![]
    };
    call_int_intrinsic(
        compiler,
        &format!("llvm.{}.i64", intrinsic_name),
        val_i64,
        &extra,
    )
}

pub fn compile_ctlz<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    compile_bit_count(compiler, args, "ctlz", true)
}

pub fn compile_cttz<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    compile_bit_count(compiler, args, "cttz", true)
}

pub fn compile_ctpop<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    compile_bit_count(compiler, args, "ctpop", false)
}

// =============================================================================
// Panic Intrinsic
// =============================================================================

pub fn compile_panic<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // panic(message: StringLiteral) -> !
    // Writes message to stderr then calls abort()

    // Get the message string
    let msg_val = compiler.compile_expression(&args[0])?;
    let msg_ptr = extract_string_ptr(compiler, msg_val)?;

    // Get string length - we need to walk until null terminator or use a reasonable max
    // For simplicity, use fputs which handles null-terminated strings

    // Get stderr file handle (on Unix, stderr is fd 2)
    // We'll use the C library's stderr via fputs for portability

    // Declare fputs: int fputs(const char *s, FILE *stream)
    let fputs = get_or_declare_fn(
        compiler,
        "fputs",
        Some(compiler.context.i32_type().into()),
        &[ptr_type(compiler).into(), ptr_type(compiler).into()],
    );

    // Declare stderr (extern FILE *stderr)
    let stderr_global = compiler.module.get_global("stderr").unwrap_or_else(|| {
        compiler
            .module
            .add_global(ptr_type(compiler), None, "stderr")
    });
    let stderr_ptr = compiler.builder.build_load(
        ptr_type(compiler),
        stderr_global.as_pointer_value(),
        "stderr",
    )?;

    // Print "panic: " prefix
    let prefix = compiler
        .builder
        .build_global_string_ptr("panic: ", "panic_prefix")?;
    compiler.builder.build_call(
        fputs,
        &[prefix.as_pointer_value().into(), stderr_ptr.into()],
        "",
    )?;

    // Print the message
    compiler
        .builder
        .build_call(fputs, &[msg_ptr.into(), stderr_ptr.into()], "")?;

    // Print newline
    let newline = compiler.builder.build_global_string_ptr("\n", "newline")?;
    compiler.builder.build_call(
        fputs,
        &[newline.as_pointer_value().into(), stderr_ptr.into()],
        "",
    )?;

    // Call abort() to terminate
    let abort = get_or_declare_fn(compiler, "abort", None, &[]);
    compiler.builder.build_call(abort, &[], "")?;

    // This is unreachable, but we need to return something
    // Mark as unreachable for LLVM optimization
    compiler.builder.build_unreachable()?;

    // Return a dummy value (will never be reached)
    Ok(compiler.context.i32_type().const_zero().into())
}

// =============================================================================
// Inline C Compilation
// =============================================================================

pub fn compile_inline_c<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    use std::io::Write;
    use std::process::Command;

    let span = compiler.get_current_span();

    // Extract C code from string literal argument
    let c_code = match &args[0] {
        ast::Expression::String(s) => s.clone(),
        _ => {
            return Err(CompileError::TypeError(
                "inline_c requires a string literal argument".to_string(),
                span,
            ))
        }
    };

    // Create temp files
    let temp_dir = std::env::temp_dir();
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let c_file = temp_dir.join(format!("zen_inline_{}.c", id));
    let bc_file = temp_dir.join(format!("zen_inline_{}.bc", id));

    // Write C code to temp file
    let mut file = std::fs::File::create(&c_file).map_err(|e| {
        CompileError::InternalError(format!("Failed to create temp C file: {}", e), span.clone())
    })?;
    file.write_all(c_code.as_bytes()).map_err(|e| {
        CompileError::InternalError(format!("Failed to write C code: {}", e), span.clone())
    })?;
    drop(file);

    // Compile C to LLVM bitcode using clang
    let output = Command::new("clang")
        .args([
            "-emit-llvm",
            "-c",
            "-O2",
            "-o",
            bc_file.to_str().unwrap_or(""),
            c_file.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| {
            CompileError::InternalError(
                format!("Failed to run clang (is it installed?): {}", e),
                span.clone(),
            )
        })?;

    // Clean up C file
    let _ = std::fs::remove_file(&c_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_file(&bc_file);
        return Err(CompileError::InternalError(
            format!("Clang compilation failed:\n{}", stderr),
            span.clone(),
        ));
    }

    // Load the bitcode and link into current module
    let memory_buffer =
        inkwell::memory_buffer::MemoryBuffer::create_from_file(&bc_file).map_err(|e| {
            let _ = std::fs::remove_file(&bc_file);
            CompileError::InternalError(format!("Failed to load bitcode: {:?}", e), span.clone())
        })?;

    let inline_module = compiler
        .context
        .create_module_from_ir(memory_buffer)
        .map_err(|e| {
            let _ = std::fs::remove_file(&bc_file);
            CompileError::InternalError(format!("Failed to parse bitcode: {:?}", e), span.clone())
        })?;

    // Clean up bitcode file
    let _ = std::fs::remove_file(&bc_file);

    // Link the inline module into our main module
    compiler.module.link_in_module(inline_module).map_err(|e| {
        CompileError::InternalError(format!("Failed to link inline C module: {:?}", e), span)
    })?;

    Ok(compiler.context.i32_type().const_zero().into())
}

pub fn compile_call_external<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    _args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    Err(CompileError::InternalError(
        "call_external not implemented - use inline_c to define C functions, then call them directly".to_string(),
        compiler.get_current_span(),
    ))
}

// =============================================================================
// Syscall Intrinsics (Linux x86-64)
// =============================================================================

/// Helper to build a syscall with inline assembly
fn build_syscall<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    syscall_num: IntValue<'ctx>,
    args: &[IntValue<'ctx>],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let i64_type = compiler.context.i64_type();

    // Build constraint strings based on number of arguments
    // Linux x86-64 syscall: number in rax, args in rdi, rsi, rdx, r10, r8, r9
    // Returns in rax, clobbers rcx and r11
    let (asm_template, constraints, num_inputs) = match args.len() {
        0 => ("syscall", "={rax},{rax},~{rcx},~{r11},~{memory}", 1),
        1 => ("syscall", "={rax},{rax},{rdi},~{rcx},~{r11},~{memory}", 2),
        2 => (
            "syscall",
            "={rax},{rax},{rdi},{rsi},~{rcx},~{r11},~{memory}",
            3,
        ),
        3 => (
            "syscall",
            "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}",
            4,
        ),
        4 => (
            "syscall",
            "={rax},{rax},{rdi},{rsi},{rdx},{r10},~{rcx},~{r11},~{memory}",
            5,
        ),
        5 => (
            "syscall",
            "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},~{rcx},~{r11},~{memory}",
            6,
        ),
        6 => (
            "syscall",
            "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}",
            7,
        ),
        _ => {
            return Err(CompileError::InternalError(
                "Too many syscall arguments".to_string(),
                compiler.get_current_span(),
            ))
        }
    };

    // Build input values array
    let mut inputs: Vec<BasicValueEnum> = vec![syscall_num.into()];
    for arg in args {
        inputs.push((*arg).into());
    }

    // Create inline assembly function type
    let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
        (0..num_inputs).map(|_| i64_type.into()).collect();
    let fn_type = i64_type.fn_type(&param_types, false);

    // Build the inline assembly call
    let asm_fn = compiler.context.create_inline_asm(
        fn_type,
        asm_template.to_string(),
        constraints.to_string(),
        true,  // has_side_effects
        false, // is_alignstack
        None,  // dialect (Intel/AT&T) - None uses default
        false, // can_throw
    );

    let input_metas: Vec<inkwell::values::BasicMetadataValueEnum> =
        inputs.iter().map(|v| (*v).into()).collect();
    let result =
        compiler
            .builder
            .build_indirect_call(fn_type, asm_fn, &input_metas, "syscall_result")?;

    Ok(result
        .try_as_basic_value()
        .left()
        .unwrap_or_else(|| i64_type.const_zero().into()))
}

pub fn compile_syscall0<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let num_val = compiler.compile_expression(&args[0])?;
    let num = to_i64(compiler, num_val, true)?;
    build_syscall(compiler, num, &[])
}

pub fn compile_syscall1<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let num_val = compiler.compile_expression(&args[0])?;
    let a0_val = compiler.compile_expression(&args[1])?;
    let num = to_i64(compiler, num_val, true)?;
    let a0 = to_i64(compiler, a0_val, true)?;
    build_syscall(compiler, num, &[a0])
}

pub fn compile_syscall2<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let num_val = compiler.compile_expression(&args[0])?;
    let a0_val = compiler.compile_expression(&args[1])?;
    let a1_val = compiler.compile_expression(&args[2])?;
    let num = to_i64(compiler, num_val, true)?;
    let a0 = to_i64(compiler, a0_val, true)?;
    let a1 = to_i64(compiler, a1_val, true)?;
    build_syscall(compiler, num, &[a0, a1])
}

pub fn compile_syscall3<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let num_val = compiler.compile_expression(&args[0])?;
    let a0_val = compiler.compile_expression(&args[1])?;
    let a1_val = compiler.compile_expression(&args[2])?;
    let a2_val = compiler.compile_expression(&args[3])?;
    let num = to_i64(compiler, num_val, true)?;
    let a0 = to_i64(compiler, a0_val, true)?;
    let a1 = to_i64(compiler, a1_val, true)?;
    let a2 = to_i64(compiler, a2_val, true)?;
    build_syscall(compiler, num, &[a0, a1, a2])
}

pub fn compile_syscall4<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let num_val = compiler.compile_expression(&args[0])?;
    let a0_val = compiler.compile_expression(&args[1])?;
    let a1_val = compiler.compile_expression(&args[2])?;
    let a2_val = compiler.compile_expression(&args[3])?;
    let a3_val = compiler.compile_expression(&args[4])?;
    let num = to_i64(compiler, num_val, true)?;
    let a0 = to_i64(compiler, a0_val, true)?;
    let a1 = to_i64(compiler, a1_val, true)?;
    let a2 = to_i64(compiler, a2_val, true)?;
    let a3 = to_i64(compiler, a3_val, true)?;
    build_syscall(compiler, num, &[a0, a1, a2, a3])
}

pub fn compile_syscall5<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let num_val = compiler.compile_expression(&args[0])?;
    let a0_val = compiler.compile_expression(&args[1])?;
    let a1_val = compiler.compile_expression(&args[2])?;
    let a2_val = compiler.compile_expression(&args[3])?;
    let a3_val = compiler.compile_expression(&args[4])?;
    let a4_val = compiler.compile_expression(&args[5])?;
    let num = to_i64(compiler, num_val, true)?;
    let a0 = to_i64(compiler, a0_val, true)?;
    let a1 = to_i64(compiler, a1_val, true)?;
    let a2 = to_i64(compiler, a2_val, true)?;
    let a3 = to_i64(compiler, a3_val, true)?;
    let a4 = to_i64(compiler, a4_val, true)?;
    build_syscall(compiler, num, &[a0, a1, a2, a3, a4])
}

pub fn compile_syscall6<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let num_val = compiler.compile_expression(&args[0])?;
    let a0_val = compiler.compile_expression(&args[1])?;
    let a1_val = compiler.compile_expression(&args[2])?;
    let a2_val = compiler.compile_expression(&args[3])?;
    let a3_val = compiler.compile_expression(&args[4])?;
    let a4_val = compiler.compile_expression(&args[5])?;
    let a5_val = compiler.compile_expression(&args[6])?;
    let num = to_i64(compiler, num_val, true)?;
    let a0 = to_i64(compiler, a0_val, true)?;
    let a1 = to_i64(compiler, a1_val, true)?;
    let a2 = to_i64(compiler, a2_val, true)?;
    let a3 = to_i64(compiler, a3_val, true)?;
    let a4 = to_i64(compiler, a4_val, true)?;
    let a5 = to_i64(compiler, a5_val, true)?;
    build_syscall(compiler, num, &[a0, a1, a2, a3, a4, a5])
}

// =============================================================================
// IO Operations (libc wrappers)
// =============================================================================

/// libc write(fd, buf, count) -> ssize_t
pub fn compile_libc_write<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let fd_val = compiler.compile_expression(&args[0])?;
    let buf_val = compiler.compile_expression(&args[1])?;
    let len_val = compiler.compile_expression(&args[2])?;

    let i32_type = compiler.context.i32_type();
    let i64_type = compiler.context.i64_type();
    let ptr_type = compiler.context.ptr_type(AddressSpace::default());

    // Get or declare libc write function
    let write_fn = get_or_declare_fn(
        compiler,
        "write",
        Some(i64_type.into()),
        &[i32_type.into(), ptr_type.into(), i64_type.into()],
    );

    // Convert fd to i32
    let fd = if fd_val.is_int_value() {
        to_int_width(compiler, fd_val.into_int_value(), i32_type, true)?
    } else {
        return Err(CompileError::TypeError(
            "libc_write: fd must be an integer".to_string(),
            compiler.get_current_span(),
        ));
    };

    // Get buffer pointer
    let buf = if buf_val.is_pointer_value() {
        buf_val.into_pointer_value()
    } else {
        return Err(CompileError::TypeError(
            "libc_write: buf must be a pointer".to_string(),
            compiler.get_current_span(),
        ));
    };

    // Convert len to i64 (size_t)
    let len = to_i64(compiler, len_val, false)?;

    let result = compiler.builder.build_call(
        write_fn,
        &[fd.into(), buf.into(), len.into()],
        "write_result",
    )?;

    extract_call_result(result, "write", compiler)
}

/// libc read(fd, buf, count) -> ssize_t
pub fn compile_libc_read<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    args: &[ast::Expression],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    let fd_val = compiler.compile_expression(&args[0])?;
    let buf_val = compiler.compile_expression(&args[1])?;
    let len_val = compiler.compile_expression(&args[2])?;

    let i32_type = compiler.context.i32_type();
    let i64_type = compiler.context.i64_type();
    let ptr_type = compiler.context.ptr_type(AddressSpace::default());

    // Get or declare libc read function
    let read_fn = get_or_declare_fn(
        compiler,
        "read",
        Some(i64_type.into()),
        &[i32_type.into(), ptr_type.into(), i64_type.into()],
    );

    // Convert fd to i32
    let fd = if fd_val.is_int_value() {
        to_int_width(compiler, fd_val.into_int_value(), i32_type, true)?
    } else {
        return Err(CompileError::TypeError(
            "libc_read: fd must be an integer".to_string(),
            compiler.get_current_span(),
        ));
    };

    // Get buffer pointer
    let buf = if buf_val.is_pointer_value() {
        buf_val.into_pointer_value()
    } else {
        return Err(CompileError::TypeError(
            "libc_read: buf must be a pointer".to_string(),
            compiler.get_current_span(),
        ));
    };

    // Convert len to i64 (size_t)
    let len = to_i64(compiler, len_val, false)?;

    let result = compiler.builder.build_call(
        read_fn,
        &[fd.into(), buf.into(), len.into()],
        "read_result",
    )?;

    extract_call_result(result, "read", compiler)
}
