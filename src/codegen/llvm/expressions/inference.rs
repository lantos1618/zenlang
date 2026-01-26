//! Type inference for codegen.
//!
//! ⚠️  ARCHITECTURAL WARNING - READ BEFORE MODIFYING ⚠️
//!
//! This file contains TECHNICAL DEBT that should be removed, not extended.
//!
//! DO NOT add hardcoded type names for stdlib types (Vec, HashMap, String, etc.)
//! These are Layer 3 types that should have NO special compiler handling.
//! See ARCHITECTURE.md in this directory and docs/design/SEPARATION_OF_CONCERNS.md
//!
//! The correct approach:
//! 1. Typechecker populates TypeContext with all type information
//! 2. Codegen queries TypeContext (compiler.type_ctx)
//! 3. NO hardcoded fallbacks for stdlib types
//!
//! Only Layer 1 primitives (i32, bool, intrinsics) and Layer 2 well-known types
//! (Option, Result, Ptr via compiler.well_known) should have special handling.

use crate::ast::{AstType, Expression};
use crate::codegen::llvm::{symbols, LLVMCompiler};
use crate::error::CompileError;
use crate::intrinsics as compiler_intrinsics;

// ============================================================================
// HELPER FUNCTIONS - Shared logic for type inference
// ============================================================================

/// Look up a struct field type by struct name and field name
fn lookup_struct_field_type(
    compiler: &LLVMCompiler,
    struct_name: &str,
    member: &str,
) -> Result<AstType, CompileError> {
    // Check TypeContext first (from typechecker)
    if let Some(field_type) = compiler.type_ctx.get_struct_field_type(struct_name, member) {
        return Ok(field_type);
    }
    // Fall back to local struct_types cache
    if let Some(struct_info) = compiler.struct_types.get(struct_name) {
        if let Some((_index, field_type)) = struct_info.fields.get(member) {
            Ok(field_type.clone())
        } else {
            Err(CompileError::TypeError(
                format!("Struct '{}' has no field '{}'", struct_name, member),
                None,
            ))
        }
    } else {
        Err(CompileError::TypeError(
            format!("Unknown struct type: {}", struct_name),
            None,
        ))
    }
}

pub fn infer_expression_type(
    compiler: &LLVMCompiler,
    expr: &Expression,
) -> Result<AstType, CompileError> {
    match expr {
        Expression::Integer8(_) => Ok(AstType::I8),
        Expression::Integer16(_) => Ok(AstType::I16),
        Expression::Integer32(_) => Ok(AstType::I32),
        Expression::Integer64(_) => Ok(AstType::I64),
        Expression::Unsigned8(_) => Ok(AstType::U8),
        Expression::Unsigned16(_) => Ok(AstType::U16),
        Expression::Unsigned32(_) => Ok(AstType::U32),
        Expression::Unsigned64(_) => Ok(AstType::U64),
        Expression::Float32(_) => Ok(AstType::F32),
        Expression::Float64(_) => Ok(AstType::F64),
        Expression::Boolean(_) => Ok(AstType::Bool),
        Expression::Unit => Ok(AstType::Void),
        Expression::String(_) => Ok(AstType::StaticString), // String literals are static strings
        Expression::Identifier(name) => {
            // Look up variable type
            if let Some(var_info) = compiler.variables.get(name) {
                Ok(var_info.ast_type.clone())
            } else {
                // If not found, it might be a pattern binding - default to I32
                // This is a heuristic that works for most cases
                Ok(AstType::I32)
            }
        }
        Expression::Range {
            start: _,
            end: _,
            inclusive,
        } => {
            // Range expressions have Range type
            Ok(AstType::Range {
                start_type: Box::new(AstType::I32),
                end_type: Box::new(AstType::I32),
                inclusive: *inclusive,
            })
        }
        Expression::EnumVariant {
            enum_name,
            variant,
            payload,
        } => infer_enum_variant_type(compiler, enum_name, variant, payload),
        Expression::FunctionCall {
            name, type_args, ..
        } => {
            // Check TypeContext for function return type (includes Type.method style)
            if let Some(return_type) = compiler.type_ctx.get_function_return_type(name) {
                return Ok(return_type);
            }
            // Check function_types map
            if let Some(return_type) = compiler.function_types.get(name) {
                return Ok(return_type.clone());
            }
            // Check StdlibTypeRegistry for Type.method calls
            if let Some(dot_pos) = name.find('.') {
                let type_name = &name[..dot_pos];
                let method_name = &name[dot_pos + 1..];
                let registry = crate::stdlib_types::stdlib_types();
                if let Some(return_type) = registry.get_method_return_type(type_name, method_name) {
                    return Ok(return_type.clone());
                }
            }

            // Handle both "compiler." and "builtin." prefixes for intrinsics
            if name.starts_with("compiler.") || name.starts_with("builtin.") {
                let prefix_len = if name.starts_with("compiler.") {
                    9
                } else {
                    8
                };
                let method = &name[prefix_len..];
                if let Some(return_type) = compiler_intrinsics::get_intrinsic_return_type(method) {
                    return Ok(return_type);
                }
            }

            // Use type_args from AST if available
            if !type_args.is_empty() {
                return Ok(AstType::Generic {
                    name: name.to_string(),
                    type_args: type_args.clone(),
                });
            }

            // Check TypeContext first (from typechecker), then fall back to local cache
            if let Some(return_type) = compiler.type_ctx.get_function_return_type(name) {
                Ok(return_type)
            } else if let Some(return_type) = compiler.function_types.get(name) {
                Ok(return_type.clone())
            } else if let Ok((_alloca, var_type)) = compiler.get_variable(name) {
                // It's a function pointer variable - get the return type
                match var_type {
                    AstType::Function { return_type, .. }
                    | AstType::FunctionPointer { return_type, .. } => {
                        Ok(return_type.as_ref().clone())
                    }
                    _ => {
                        Ok(AstType::I32) // Default for non-function variables
                    }
                }
            } else {
                // Default to I32 for unknown functions
                Ok(AstType::I32)
            }
        }
        Expression::Raise(object) => {
            let object_type = compiler.infer_expression_type(object)?;
            if let AstType::Generic { name, type_args } = object_type {
                if compiler.well_known.is_result(&name) && type_args.len() == 2 {
                    return Ok(type_args[0].clone());
                }
            }
            Ok(AstType::Void)
        }
        Expression::MethodCall { object, method, .. } => {
            infer_method_call_type(compiler, object, method)
        }
        Expression::PatternMatch { arms, .. } => {
            // Pattern match takes the type of its first arm's body
            if let Some(first_arm) = arms.first() {
                compiler.infer_expression_type(&first_arm.body)
            } else {
                Ok(AstType::Void)
            }
        }
        Expression::QuestionMatch { arms, .. } => {
            // Question match should return the common type of all arms
            // We iterate through arms to find the first non-void type
            for arm in arms {
                let arm_type = compiler.infer_expression_type(&arm.body)?;
                // Return the first non-void type we find
                if !matches!(arm_type, AstType::Void) {
                    // Handle special cases for pattern binding
                    if let Expression::Block(stmts) = &arm.body {
                        if stmts.len() == 1 {
                            if let crate::ast::Statement::Expression {
                                expr: Expression::Identifier(_),
                                ..
                            } = &stmts[0]
                            {
                                // This is likely a pattern binding variable, assume I32 for now
                                return Ok(AstType::I32);
                            }
                        }
                    }
                    // For arrow syntax with binary ops, check for pattern bindings
                    if let Expression::BinaryOp {
                        left,
                        op: _,
                        right: _,
                    } = &arm.body
                    {
                        if matches!(**left, Expression::Identifier(_)) {
                            // Assume the pattern binding will be an integer type
                            return Ok(AstType::I32);
                        }
                    }
                    return Ok(arm_type);
                }
            }
            // If all arms are void, return void
            Ok(AstType::Void)
        }
        Expression::Conditional { arms, .. } => {
            // Conditional takes the type of its first arm's body
            if let Some(first_arm) = arms.first() {
                compiler.infer_expression_type(&first_arm.body)
            } else {
                Ok(AstType::Void)
            }
        }
        Expression::BinaryOp { op, left, right } => {
            // Binary operations return different types based on the operator
            use crate::ast::BinaryOperator;
            match op {
                BinaryOperator::GreaterThan
                | BinaryOperator::LessThan
                | BinaryOperator::GreaterThanEquals
                | BinaryOperator::LessThanEquals
                | BinaryOperator::Equals
                | BinaryOperator::NotEquals
                | BinaryOperator::And
                | BinaryOperator::Or => Ok(AstType::Bool),
                BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Modulo => {
                    // Infer type based on operands
                    let left_type = compiler.infer_expression_type(left)?;
                    let right_type = compiler.infer_expression_type(right)?;

                    // If either operand is a float, the result is a float
                    if matches!(left_type, AstType::F32 | AstType::F64)
                        || matches!(right_type, AstType::F32 | AstType::F64)
                    {
                        Ok(AstType::F64)
                    } else {
                        // Default to I32 for integer operations
                        Ok(AstType::I32)
                    }
                }
                _ => Ok(AstType::Void),
            }
        }
        Expression::Block(stmts) => {
            // Block takes the type of its last expression
            if let Some(last_stmt) = stmts.last() {
                match last_stmt {
                    crate::ast::Statement::Expression { expr, .. } => {
                        // If the last statement is an expression, the block evaluates to its type
                        compiler.infer_expression_type(expr)
                    }
                    crate::ast::Statement::Return { .. } => {
                        // If the block ends with a return, it doesn't evaluate to a value
                        // Instead, it's void since control flow leaves the block
                        Ok(AstType::Void)
                    }
                    _ => Ok(AstType::Void),
                }
            } else {
                Ok(AstType::Void)
            }
        }
        Expression::Some(value) => {
            let inner_type = compiler.infer_expression_type(value)?;
            Ok(AstType::Generic {
                name: compiler
                    .well_known
                    .get_variant_parent_name(compiler.well_known.some_name())
                    .unwrap_or(compiler.well_known.option_name())
                    .to_string(),
                type_args: vec![inner_type],
            })
        }
        Expression::None => {
            let option_name = compiler
                .well_known
                .get_variant_parent_name(compiler.well_known.none_name())
                .unwrap_or(compiler.well_known.option_name());
            if let Some(t) = compiler.generic_type_context.get("Option_Some_Type") {
                Ok(AstType::Generic {
                    name: option_name.to_string(),
                    type_args: vec![t.clone()],
                })
            } else {
                Ok(AstType::Generic {
                    name: option_name.to_string(),
                    type_args: vec![AstType::Void],
                })
            }
        }
        Expression::StructLiteral { name, .. } => {
            // Get the struct type fields from registered types
            if let Some(struct_info) = compiler.struct_types.get(name) {
                let mut fields = Vec::new();
                for (field_name, (_index, field_type)) in &struct_info.fields {
                    fields.push((field_name.clone(), field_type.clone()));
                }
                Ok(AstType::Struct {
                    name: name.clone(),
                    fields,
                })
            } else {
                // Unknown struct type
                Ok(AstType::Struct {
                    name: name.clone(),
                    fields: vec![],
                })
            }
        }
        Expression::VecConstructor { element_type, .. } => {
            // Vec<T> constructor returns Vec generic type (stdlib struct)
            Ok(AstType::Generic {
                name: "Vec".to_string(),
                type_args: vec![element_type.clone()],
            })
        }
        Expression::DynVecConstructor { element_types, .. } => {
            // DynVec<T> constructor returns DynVec generic type (stdlib struct)
            let type_arg = element_types.first().cloned().unwrap_or(AstType::Void);
            Ok(AstType::Generic {
                name: "DynVec".to_string(),
                type_args: vec![type_arg],
            })
        }
        Expression::ArrayConstructor { element_type } => {
            // Array<T> constructor returns Generic Array type
            Ok(AstType::Generic {
                name: "Array".to_string(),
                type_args: vec![element_type.clone()],
            })
        }
        Expression::StringInterpolation { .. } => {
            // String interpolation always returns a string
            Ok(AstType::StaticString)
        }
        Expression::TypeCast { target_type, .. } => {
            // Type cast returns the target type
            Ok(target_type.clone())
        }
        Expression::Closure {
            return_type, body, ..
        } => {
            // Closure returns its declared return type, or infer from body
            if let Some(ret_type) = return_type {
                Ok(ret_type.clone())
            } else {
                // Try to infer from the closure body
                infer_closure_return_type(compiler, body)
            }
        }
        Expression::Loop { .. } | Expression::CollectionLoop { .. } => {
            // Loops typically return void, unless they have explicit break with value
            // For now, assume void return
            Ok(AstType::Void)
        }
        Expression::Return(expr) => {
            // Return expression returns the type of its inner expression
            compiler.infer_expression_type(expr)
        }
        Expression::Break { value, .. } => {
            // Break can return a value or void
            if let Some(val) = value {
                compiler.infer_expression_type(val)
            } else {
                Ok(AstType::Void)
            }
        }
        Expression::Continue { .. } => {
            // Continue always returns void
            Ok(AstType::Void)
        }
        Expression::MemberAccess { object, member } => {
            // Infer the type of the object first
            let object_type = compiler.infer_expression_type(object)?;

            // Handle struct field access using helper
            match &object_type {
                AstType::Struct { name, .. } => lookup_struct_field_type(compiler, name, member),
                // Handle pointer to struct types
                t if t.is_ptr_type() => {
                    if let Some(AstType::Struct { name, .. }) = t.ptr_inner() {
                        lookup_struct_field_type(compiler, name, member)
                    } else {
                        Ok(AstType::Void)
                    }
                }
                // Handle Generic types that might be structs
                AstType::Generic { name, .. } => {
                    // Check if this generic is actually a registered struct type
                    if compiler.struct_types.contains_key(name) {
                        lookup_struct_field_type(compiler, name, member)
                    } else {
                        // Not a struct, might be a module reference (@std.something)
                        Ok(AstType::Void)
                    }
                }
                _ => Ok(AstType::Void), // Will error during compilation if needed
            }
        }
        // Pointer operations
        Expression::CreateReference(inner) => {
            // expr.ref() -> Ptr<T>
            let inner_type = infer_expression_type(compiler, inner)?;
            Ok(AstType::ptr(inner_type))
        }
        Expression::CreateMutableReference(inner) => {
            // expr.mut_ref() -> MutPtr<T>
            let inner_type = infer_expression_type(compiler, inner)?;
            Ok(AstType::mut_ptr(inner_type))
        }
        Expression::AddressOf(inner) => {
            // &expr -> Ptr<T>
            let inner_type = infer_expression_type(compiler, inner)?;
            Ok(AstType::ptr(inner_type))
        }
        Expression::Dereference(inner) | Expression::PointerDereference(inner) => {
            // *ptr or ptr.val -> T (unwrap Ptr<T>/MutPtr<T>/RawPtr<T>)
            let ptr_type = infer_expression_type(compiler, inner)?;
            if let Some(inner_type) = ptr_type.ptr_inner() {
                Ok(inner_type.clone())
            } else {
                Ok(AstType::Void) // Not a pointer type
            }
        }
        Expression::PointerAddress(inner) => {
            // ptr.addr -> usize
            let _ = infer_expression_type(compiler, inner)?;
            Ok(AstType::Usize)
        }
        Expression::PointerOffset { pointer, .. } => {
            // ptr + offset -> same pointer type
            infer_expression_type(compiler, pointer)
        }
        _ => Ok(AstType::Void),
    }
}

pub fn infer_closure_return_type(
    compiler: &LLVMCompiler,
    body: &Expression,
) -> Result<AstType, CompileError> {
    match body {
        Expression::QuestionMatch { arms, .. } => {
            // For question match expressions, check the return types of all arms
            // and return the first non-void type found (they should all be the same)
            for arm in arms {
                if let Expression::Block(statements) = &arm.body {
                    for stmt in statements {
                        if let crate::ast::Statement::Return { expr: ret_expr, .. } = stmt {
                            let ret_type = infer_expression_type(compiler, ret_expr)?;
                            if ret_type != AstType::Void {
                                return Ok(ret_type);
                            }
                        }
                    }
                }
            }
            Ok(AstType::Void)
        }
        Expression::Block(statements) => {
            // Look for the last expression or a return statement
            for stmt in statements {
                if let crate::ast::Statement::Return { expr: ret_expr, .. } = stmt {
                    // Recursively infer the return type, especially for closures that just return a Result
                    return infer_expression_type(compiler, ret_expr);
                }
            }
            // Check if the last statement is an expression
            if let Some(last_stmt) = statements.last() {
                match last_stmt {
                    crate::ast::Statement::Expression { expr, .. } => {
                        return infer_expression_type(compiler, expr);
                    }
                    crate::ast::Statement::Return { expr, .. } => {
                        return infer_expression_type(compiler, expr);
                    }
                    _ => {}
                }
            }
            Ok(AstType::Void)
        }
        Expression::FunctionCall { name, args, .. } => {
            // Check if this is a Result or Option variant constructor using well_known
            // Parse "Type.Variant" pattern
            if let Some((type_name, variant_name)) = name.split_once('.') {
                let wk = &compiler.well_known;

                // Check for Result.Ok or Result.Err
                if wk.is_result(type_name) {
                    if variant_name == wk.ok_name() {
                        let t_type = if !args.is_empty() {
                            infer_expression_type(compiler, &args[0])?
                        } else {
                            AstType::Void
                        };
                        return Ok(AstType::Generic {
                            name: wk.result_name().to_string(),
                            type_args: vec![t_type, AstType::StaticString],
                        });
                    } else if variant_name == wk.err_name() {
                        let e_type = if !args.is_empty() {
                            infer_expression_type(compiler, &args[0])?
                        } else {
                            crate::ast::resolve_string_struct_type()
                        };
                        return Ok(AstType::Generic {
                            name: wk.result_name().to_string(),
                            type_args: vec![AstType::I32, e_type],
                        });
                    }
                }

                // Check for Option.Some or Option.None
                if wk.is_option(type_name) {
                    if variant_name == wk.some_name() {
                        let t_type = if !args.is_empty() {
                            infer_expression_type(compiler, &args[0])?
                        } else {
                            AstType::Void
                        };
                        return Ok(AstType::Generic {
                            name: wk.option_name().to_string(),
                            type_args: vec![t_type],
                        });
                    } else if variant_name == wk.none_name() {
                        return Ok(AstType::Generic {
                            name: wk.option_name().to_string(),
                            type_args: vec![AstType::Generic {
                                name: "T".to_string(),
                                type_args: vec![],
                            }],
                        });
                    }
                }
            }

            // Not a Result/Option variant - check TypeContext and local cache
            {
                // Check TypeContext first, then local cache
                if let Some(return_type) = compiler.type_ctx.get_function_return_type(name) {
                    Ok(return_type)
                } else if let Some(return_type) = compiler.function_types.get(name) {
                    Ok(return_type.clone())
                } else {
                    Ok(AstType::I32) // Default
                }
            }
        }
        _ => infer_expression_type(compiler, body),
    }
}

// ============================================================================
// HELPER FUNCTIONS FOR TYPE INFERENCE
// ============================================================================

/// Infer type for enum variant expressions (Option, Result, custom enums)
fn infer_enum_variant_type(
    compiler: &LLVMCompiler,
    enum_name: &str,
    variant: &str,
    payload: &Option<Box<Expression>>,
) -> Result<AstType, CompileError> {
    let wk = &compiler.well_known;

    if wk.is_option(enum_name) {
        infer_option_variant_type(compiler, variant, payload)
    } else if wk.is_result(enum_name) {
        infer_result_variant_type(compiler, variant, payload)
    } else {
        infer_custom_enum_type(compiler, enum_name)
    }
}

/// Infer type for Option variants (Some/None)
fn infer_option_variant_type(
    compiler: &LLVMCompiler,
    variant: &str,
    payload: &Option<Box<Expression>>,
) -> Result<AstType, CompileError> {
    let wk = &compiler.well_known;
    let parent_name = wk
        .get_variant_parent_name(variant)
        .unwrap_or(wk.option_name());

    if wk.is_some(variant) && payload.is_some() {
        if let Some(ref p) = payload {
            let inner_type = infer_expression_type(compiler, p)?;
            Ok(AstType::Generic {
                name: parent_name.to_string(),
                type_args: vec![inner_type],
            })
        } else {
            Ok(make_generic_option(parent_name))
        }
    } else {
        // None variant or Some without payload - try to get from context
        if let Some(t) = compiler.generic_type_context.get("Option_Some_Type") {
            Ok(AstType::Generic {
                name: parent_name.to_string(),
                type_args: vec![t.clone()],
            })
        } else {
            Ok(make_generic_option(parent_name))
        }
    }
}

/// Infer type for Result variants (Ok/Err)
fn infer_result_variant_type(
    compiler: &LLVMCompiler,
    variant: &str,
    payload: &Option<Box<Expression>>,
) -> Result<AstType, CompileError> {
    let wk = &compiler.well_known;
    let parent_name = wk
        .get_variant_parent_name(variant)
        .unwrap_or(wk.result_name());

    if wk.is_ok(variant) && payload.is_some() {
        if let Some(ref p) = payload {
            let inner_type = infer_expression_type(compiler, p)?;
            let err_type = compiler
                .generic_type_context
                .get("Result_Err_Type")
                .cloned()
                .unwrap_or(AstType::StaticString);
            Ok(AstType::Generic {
                name: parent_name.to_string(),
                type_args: vec![inner_type, err_type],
            })
        } else {
            Ok(make_generic_result(parent_name))
        }
    } else if wk.is_err(variant) && payload.is_some() {
        if let Some(ref p) = payload {
            let inner_type = infer_expression_type(compiler, p)?;
            let ok_type = compiler
                .generic_type_context
                .get("Result_Ok_Type")
                .cloned()
                .unwrap_or(AstType::Void);
            Ok(AstType::Generic {
                name: parent_name.to_string(),
                type_args: vec![ok_type, inner_type],
            })
        } else {
            Ok(make_generic_result(parent_name))
        }
    } else {
        // Get types from context or use defaults
        let ok_type = compiler
            .generic_type_context
            .get("Result_Ok_Type")
            .cloned()
            .unwrap_or(AstType::Void);
        let err_type = compiler
            .generic_type_context
            .get("Result_Err_Type")
            .cloned()
            .unwrap_or(AstType::Void);
        Ok(AstType::Generic {
            name: parent_name.to_string(),
            type_args: vec![ok_type, err_type],
        })
    }
}

/// Infer type for custom enum variants
fn infer_custom_enum_type(
    compiler: &LLVMCompiler,
    enum_name: &str,
) -> Result<AstType, CompileError> {
    // Check TypeContext first (from typechecker)
    if let Some(variants) = compiler.type_ctx.get_enum_variants(enum_name) {
        let variants = variants
            .iter()
            .map(|(name, payload)| crate::ast::EnumVariant {
                name: name.clone(),
                payload: payload.clone(),
            })
            .collect();
        return Ok(AstType::Enum {
            name: enum_name.to_string(),
            variants,
        });
    }

    // Fall back to symbol table
    if let Some(symbols::Symbol::EnumType(enum_info)) = compiler.symbols.lookup(enum_name) {
        let variants = enum_info
            .variants
            .iter()
            .map(|v| crate::ast::EnumVariant {
                name: v.name.clone(),
                payload: v.payload.clone(),
            })
            .collect();
        Ok(AstType::Enum {
            name: enum_name.to_string(),
            variants,
        })
    } else {
        Ok(AstType::EnumType {
            name: enum_name.to_string(),
        })
    }
}

/// Create a generic Option<T> type placeholder
fn make_generic_option(parent_name: &str) -> AstType {
    AstType::Generic {
        name: parent_name.to_string(),
        type_args: vec![AstType::Generic {
            name: "T".to_string(),
            type_args: vec![],
        }],
    }
}

/// Create a generic Result<T, E> type placeholder
fn make_generic_result(parent_name: &str) -> AstType {
    AstType::Generic {
        name: parent_name.to_string(),
        type_args: vec![
            AstType::Generic {
                name: "T".to_string(),
                type_args: vec![],
            },
            AstType::Generic {
                name: "E".to_string(),
                type_args: vec![],
            },
        ],
    }
}

/// Infer type for method call expressions
fn infer_method_call_type(
    compiler: &LLVMCompiler,
    object: &Expression,
    method: &str,
) -> Result<AstType, CompileError> {
    // Check for compiler intrinsics
    if let Expression::Identifier(name) = object {
        if name == "compiler" {
            let base_method = if let Some(angle_pos) = method.find('<') {
                &method[..angle_pos]
            } else {
                method
            };
            if let Some(return_type) = compiler_intrinsics::get_intrinsic_return_type(base_method) {
                return Ok(return_type);
            }
        }
    }

    // Handle raise method
    if method == "raise" {
        return infer_raise_method_type(compiler, object);
    }

    // Handle constructors (including generic constructors like new<K,V>)
    let base_method = if let Some(angle_pos) = method.find('<') {
        &method[..angle_pos]
    } else {
        method
    };
    if base_method == "new" || base_method == "init" || base_method == "with_step" {
        return infer_constructor_type(compiler, object, method);
    }

    // Handle common methods by name
    infer_common_method_type(compiler, object, method)
}

/// Infer type for raise method (unwraps Result)
fn infer_raise_method_type(
    compiler: &LLVMCompiler,
    object: &Expression,
) -> Result<AstType, CompileError> {
    let object_type = compiler.infer_expression_type(object)?;
    if let AstType::Generic { name, type_args } = &object_type {
        if compiler.well_known.is_result(name) && type_args.len() == 2 {
            return Ok(type_args[0].clone());
        }
    }
    Ok(AstType::Void)
}

/// Infer type for constructor methods (new/init)
fn infer_constructor_type(
    compiler: &LLVMCompiler,
    object: &Expression,
    method: &str,
) -> Result<AstType, CompileError> {
    if let Expression::Identifier(name) = object {
        // Check for type args in method name (e.g., HashMap.new<K, V>())
        if method.contains('<') {
            if let Some(angle_pos) = method.find('<') {
                let args_str = &method[angle_pos + 1..method.len() - 1];
                let type_args = super::utils::parse_type_args_string(compiler, args_str)?;
                return Ok(AstType::Generic {
                    name: name.to_string(),
                    type_args,
                });
            }
        }

        // Check for generic type constructor (e.g., HashMap<K,V>.new())
        if name.contains('<') {
            if let Some(angle_pos) = name.find('<') {
                let base_type = &name[..angle_pos];
                let args_str = &name[angle_pos + 1..name.len() - 1];
                let type_args = super::utils::parse_type_args_string(compiler, args_str)?;
                return Ok(AstType::Generic {
                    name: base_type.to_string(),
                    type_args,
                });
            }
        }

        // Check TypeContext for constructor return type (user-defined types)
        let base_method = if let Some(angle_pos) = method.find('<') {
            &method[..angle_pos]
        } else {
            method
        };

        // First check the dedicated constructors map
        if let Some(return_type) = compiler.type_ctx.get_constructor_type(name, base_method) {
            return Ok(return_type);
        }

        // Then check method return types
        if let Some(return_type) = compiler.type_ctx.get_method_return_type(name, base_method) {
            return Ok(return_type);
        }

        // Also check qualified function name
        let qualified = format!("{}.{}", name, base_method);
        if let Some(return_type) = compiler.type_ctx.get_function_return_type(&qualified) {
            return Ok(return_type);
        }

        // Check StdlibTypeRegistry for constructor return type
        let registry = crate::stdlib_types::stdlib_types();
        if let Some(return_type) = registry.get_method_return_type(name, base_method) {
            return Ok(return_type.clone());
        }

        // Check struct definitions (non-generic types like GPA, String)
        if compiler.type_ctx.has_struct(name) {
            if let Some(fields) = compiler.type_ctx.get_struct_fields(name) {
                return Ok(AstType::Struct {
                    name: name.to_string(),
                    fields: fields.clone(),
                });
            }
        }

        // For unknown types, return struct with empty fields
        // (typechecker should have caught any real errors)
        return Ok(AstType::Struct {
            name: name.to_string(),
            fields: vec![],
        });
    }
    Ok(AstType::Void)
}

/// Infer type for common methods by name
fn infer_common_method_type(
    compiler: &LLVMCompiler,
    object: &Expression,
    method: &str,
) -> Result<AstType, CompileError> {
    // Check TypeContext first for method return type
    if let Ok(object_type) = compiler.infer_expression_type(object) {
        if let Some(type_name) = object_type.get_type_name() {
            if let Some(return_type) = compiler.type_ctx.get_method_return_type(&type_name, method)
            {
                return Ok(return_type);
            }
        }
    }

    // Special case: numeric methods that return same type as input
    // These are true intrinsics - they work on any numeric type
    if matches!(method, "abs" | "min" | "max") {
        return compiler.infer_expression_type(object);
    }

    // Special case: collection access methods need type arg extraction
    // This handles generic return types like Option<T> where T comes from the collection
    if matches!(method, "get" | "remove" | "insert" | "pop") {
        let option_name = compiler
            .well_known
            .get_variant_parent_name(compiler.well_known.some_name())
            .unwrap_or(compiler.well_known.option_name());
        return infer_collection_access_type(compiler, object, method, option_name);
    }

    // Special case: set operations return the same set type
    if matches!(
        method,
        "union" | "intersection" | "difference" | "symmetric_difference"
    ) {
        return infer_set_operation_type(compiler, object);
    }

    // All other methods: use UFC lookup (TypeContext → StdlibTypeRegistry → function_types)
    infer_ufc_method_type(compiler, object, method)
}

/// Infer type for collection access methods (get, pop, etc.)
///
/// This handles generic return types where the return type depends on
/// the collection's type parameters (e.g., Vec<T>.get() -> Option<T>).
fn infer_collection_access_type(
    compiler: &LLVMCompiler,
    object: &Expression,
    method: &str,
    option_name: &str,
) -> Result<AstType, CompileError> {
    let object_type = compiler.infer_expression_type(object)?;

    // First try TypeContext for method return type
    if let Some(type_name) = object_type.get_type_name() {
        if let Some(return_type) = compiler.type_ctx.get_method_return_type(&type_name, method) {
            return Ok(return_type);
        }
    }

    // For generic collections, infer return type from type parameters
    // This is generic programming logic - works for any collection with matching structure
    if let AstType::Generic { name: _, type_args } = &object_type {
        // Two-param generics (like Map<K, V>): get/remove returns Option<V>
        if type_args.len() >= 2 && matches!(method, "get" | "remove") {
            return Ok(AstType::Generic {
                name: option_name.to_string(),
                type_args: vec![type_args[1].clone()],
            });
        }
        // Single-param generics (like List<T>): get/pop returns Option<T>
        if !type_args.is_empty() && matches!(method, "get" | "pop") {
            return Ok(AstType::Generic {
                name: option_name.to_string(),
                type_args: vec![type_args[0].clone()],
            });
        }
    }

    // DynVec special case - now a Generic type from stdlib
    if let AstType::Generic { name, type_args } = &object_type {
        if name == "DynVec" && !type_args.is_empty() && matches!(method, "get" | "pop") {
            return Ok(AstType::Generic {
                name: option_name.to_string(),
                type_args: vec![type_args[0].clone()],
            });
        }
    }

    // Fall back to UFC lookup
    infer_ufc_method_type(compiler, object, method)
}

/// Infer type for set operations (union, intersection, etc.)
///
/// Set operations return the same type as the receiver.
fn infer_set_operation_type(
    compiler: &LLVMCompiler,
    object: &Expression,
) -> Result<AstType, CompileError> {
    let object_type = compiler.infer_expression_type(object)?;

    // First try TypeContext for method return type
    if let Some(type_name) = object_type.get_type_name() {
        // Set operations don't have separate method names, check any of them
        if let Some(return_type) = compiler
            .type_ctx
            .get_method_return_type(&type_name, "union")
        {
            return Ok(return_type);
        }
    }

    // Set operations return the same generic type as input
    if let AstType::Generic { name, type_args } = object_type {
        return Ok(AstType::Generic { name, type_args });
    }

    Ok(AstType::Void)
}

/// Infer type via Uniform Function Call (UFC) lookup
fn infer_ufc_method_type(
    compiler: &LLVMCompiler,
    object: &Expression,
    method: &str,
) -> Result<AstType, CompileError> {
    let object_type = infer_expression_type(compiler, object)?;
    let type_name = match &object_type {
        AstType::Struct { name, .. } => Some(name.clone()),
        AstType::Generic { name, .. } => Some(name.clone()),
        _ => None,
    };

    if let Some(ref type_name) = type_name {
        // Check TypeContext first for method return type
        if let Some(return_type) = compiler.type_ctx.get_method_return_type(type_name, method) {
            return Ok(return_type);
        }

        let qualified_name = format!("{}.{}", type_name, method);
        if let Some(func_return_type) = compiler.type_ctx.get_function_return_type(&qualified_name)
        {
            return Ok(func_return_type);
        }
        if let Some(func_return_type) = compiler.function_types.get(&qualified_name) {
            return Ok(func_return_type.clone());
        }

        // Check stdlib_types for method return type
        let registry = crate::stdlib_types::stdlib_types();
        if let Some(return_type) = registry.get_method_return_type(type_name, method) {
            return Ok(return_type.clone());
        }
    }

    // Fall back to plain UFC - check TypeContext first
    if let Some(func_return_type) = compiler.type_ctx.get_function_return_type(method) {
        Ok(func_return_type)
    } else if let Some(func_return_type) = compiler.function_types.get(method) {
        Ok(func_return_type.clone())
    } else {
        Ok(AstType::Void)
    }
}
