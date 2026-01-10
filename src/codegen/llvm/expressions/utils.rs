//! Utility functions for expression compilation.

use super::super::LLVMCompiler;
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use inkwell::{types::BasicTypeEnum, values::BasicValueEnum, AddressSpace};

// ============================================================================
// HELPER FUNCTIONS FOR REDUCING DUPLICATION
// ============================================================================

/// Track Result<T, E> type arguments in the compiler's generic type context
pub fn track_result_types<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    result_type: &AstType,
    type_args: &[AstType],
) {
    if type_args.len() != 2 {
        return;
    }
    compiler.track_generic_type("Result_Ok_Type".to_string(), type_args[0].clone());
    compiler.track_generic_type("Result_Err_Type".to_string(), type_args[1].clone());
    compiler.track_complex_generic(result_type, compiler.well_known.result_name());
    compiler.generic_tracker.track_generic_type(result_type, compiler.well_known.result_name());
}

/// Convert AstType to LLVM BasicTypeEnum for loading values
/// Returns None for types that need special handling (generics, structs, strings)
pub fn ast_type_to_basic_type<'ctx>(
    compiler: &LLVMCompiler<'ctx>,
    ast_type: &AstType,
) -> Option<BasicTypeEnum<'ctx>> {
    match ast_type {
        AstType::I8 | AstType::U8 => Some(compiler.context.i8_type().into()),
        AstType::I16 | AstType::U16 => Some(compiler.context.i16_type().into()),
        AstType::I32 | AstType::U32 => Some(compiler.context.i32_type().into()),
        AstType::I64 | AstType::U64 | AstType::Usize => Some(compiler.context.i64_type().into()),
        AstType::F32 => Some(compiler.context.f32_type().into()),
        AstType::F64 => Some(compiler.context.f64_type().into()),
        AstType::Bool => Some(compiler.context.bool_type().into()),
        _ => None, // Complex types need special handling
    }
}

/// Get the LLVM struct type for Result/Option (tag + payload pointer)
pub fn generic_enum_struct_type<'ctx>(compiler: &LLVMCompiler<'ctx>) -> inkwell::types::StructType<'ctx> {
    compiler.context.struct_type(
        &[
            compiler.context.i64_type().into(), // discriminant/tag
            compiler.context.ptr_type(AddressSpace::default()).into(), // payload pointer
        ],
        false,
    )
}

pub fn parse_type_args_string(
    compiler: &LLVMCompiler,
    type_params_str: &str,
) -> Result<Vec<AstType>, CompileError> {
    let mut type_args = Vec::new();
    let mut current = String::new();
    let mut angle_depth = 0;

    for ch in type_params_str.chars() {
        if ch == '<' {
            angle_depth += 1;
            current.push(ch);
        } else if ch == '>' {
            angle_depth -= 1;
            current.push(ch);
        } else if ch == ',' && angle_depth == 0 {
            // This comma separates type arguments
            if !current.is_empty() {
                type_args.push(parse_single_type_string(compiler, current.trim())?);
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    // Don't forget the last type argument
    if !current.is_empty() {
        type_args.push(parse_single_type_string(compiler, current.trim())?);
    }

    Ok(type_args)
}

/// Parse a single type string like "i32" or "Option<i32>" into AstType
pub fn parse_single_type_string(
    compiler: &LLVMCompiler,
    type_str: &str,
) -> Result<AstType, CompileError> {
    let trimmed = type_str.trim();

    // Check for basic types first
    match trimmed {
        "i8" => Ok(AstType::I8),
        "i16" => Ok(AstType::I16),
        "i32" => Ok(AstType::I32),
        "i64" => Ok(AstType::I64),
        "u8" => Ok(AstType::U8),
        "u16" => Ok(AstType::U16),
        "u32" => Ok(AstType::U32),
        "u64" => Ok(AstType::U64),
        "f32" => Ok(AstType::F32),
        "f64" => Ok(AstType::F64),
        "bool" => Ok(AstType::Bool),
        "string" => Ok(AstType::StaticLiteral),
        "StaticString" => Ok(AstType::StaticString),
        "String" => Ok(crate::ast::resolve_string_struct_type()), // Dynamic string type
        "void" => Ok(AstType::Void),
        _ => {
            // Check if it's a generic type like "Option<i32>"
            if let Some(angle_pos) = trimmed.find('<') {
                if trimmed.ends_with('>') {
                    let base_type = &trimmed[..angle_pos];
                    let inner_types_str = &trimmed[angle_pos + 1..trimmed.len() - 1];
                    let inner_types = parse_type_args_string(compiler, inner_types_str)?;

                    Ok(AstType::Generic {
                        name: base_type.to_string(),
                        type_args: inner_types,
                    })
                } else {
                    // Invalid generic type syntax
                    Ok(AstType::I32) // Default fallback
                }
            } else {
                // Unknown type, default to I32
                Ok(AstType::I32)
            }
        }
    }
}

/// Infer the return type of a closure from its body
pub fn compile_comptime_expression<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    // Evaluate the expression at compile time using the persistent evaluator
    match compiler.comptime_evaluator.evaluate_expression(expr) {
        Ok(value) => {
            // Convert the comptime value to a constant expression and compile it
            let const_expr = value.to_expression()?;
            compiler.compile_expression(&const_expr)
        }
        Err(e) => Err(CompileError::InternalError(
            format!("Comptime evaluation error: {}", e),
            compiler.get_current_span()
        ))
    }
}
