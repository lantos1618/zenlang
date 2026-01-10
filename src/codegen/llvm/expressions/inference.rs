use crate::codegen::llvm::{symbols, LLVMCompiler};
use crate::ast::{AstType, Expression};
use crate::error::CompileError;
use crate::intrinsics as compiler_intrinsics;

// ============================================================================
// HELPER FUNCTIONS - Shared logic for type inference
// ============================================================================

/// Look up a struct field type by struct name and field name.
/// Uses TypeContext from typechecker, falls back to codegen tables.
fn lookup_struct_field_type(
    compiler: &LLVMCompiler,
    struct_name: &str,
    member: &str,
) -> Result<AstType, CompileError> {
    // Try TypeContext first (from typechecker)
    if let Some(field_type) = compiler.get_struct_field_type(struct_name, member) {
        return Ok(field_type);
    }
    // Fall back to codegen's struct_types
    if let Some(struct_info) = compiler.struct_types.get(struct_name) {
        if let Some((_index, field_type)) = struct_info.fields.get(member) {
            return Ok(field_type.clone());
        }
    }
    // Return Void for unknown - let codegen handle the error
    Ok(AstType::Void)
}

pub fn infer_expression_type(compiler: &LLVMCompiler, expr: &Expression) -> Result<AstType, CompileError> {
    use crate::ast::BinaryOperator;
    match expr {
        // Literals - trivial type mapping
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
        Expression::String(_) | Expression::StringInterpolation { .. } => Ok(AstType::StaticString),

        // Variable lookup
        Expression::Identifier(name) => Ok(compiler.variables.get(name)
            .map(|v| v.ast_type.clone()).unwrap_or(AstType::I32)),

        // Range type
        Expression::Range { inclusive, .. } => Ok(AstType::Range {
            start_type: Box::new(AstType::I32), end_type: Box::new(AstType::I32), inclusive: *inclusive,
        }),

        // Enum variants
        Expression::EnumVariant { enum_name, variant, payload } =>
            infer_enum_variant_type(compiler, enum_name, variant, payload),

        // Function calls
        Expression::FunctionCall { name, .. } => infer_function_call_type(compiler, name),

        // Method calls
        Expression::MethodCall { object, method, .. } => infer_method_call_type(compiler, object, method),

        // Raise unwraps Result<T, E> to T
        Expression::Raise(obj) => {
            if let AstType::Generic { name, type_args } = compiler.infer_expression_type(obj)? {
                if compiler.well_known.is_result(&name) && type_args.len() == 2 {
                    return Ok(type_args[0].clone());
                }
            }
            Ok(AstType::Void)
        }

        // Pattern/question/conditional - infer from first arm
        Expression::PatternMatch { arms, .. } =>
            arms.first().map(|a| compiler.infer_expression_type(&a.body)).unwrap_or(Ok(AstType::Void)),
        Expression::Conditional { arms, .. } =>
            arms.first().map(|a| compiler.infer_expression_type(&a.body)).unwrap_or(Ok(AstType::Void)),
        Expression::QuestionMatch { arms, .. } => {
            for arm in arms { let t = compiler.infer_expression_type(&arm.body)?; if t != AstType::Void { return Ok(t); } }
            Ok(AstType::Void)
        }

        // Binary ops - comparison returns bool, arithmetic infers from operands
        Expression::BinaryOp { op, left, right } => match op {
            BinaryOperator::GreaterThan | BinaryOperator::LessThan | BinaryOperator::GreaterThanEquals |
            BinaryOperator::LessThanEquals | BinaryOperator::Equals | BinaryOperator::NotEquals |
            BinaryOperator::And | BinaryOperator::Or => Ok(AstType::Bool),
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply |
            BinaryOperator::Divide | BinaryOperator::Modulo => {
                let (l, r) = (compiler.infer_expression_type(left)?, compiler.infer_expression_type(right)?);
                Ok(if matches!(l, AstType::F32 | AstType::F64) || matches!(r, AstType::F32 | AstType::F64)
                    { AstType::F64 } else { AstType::I32 })
            }
            _ => Ok(AstType::Void),
        },

        // Block - type of last expression
        Expression::Block(stmts) => match stmts.last() {
            Some(crate::ast::Statement::Expression { expr, .. }) => compiler.infer_expression_type(expr),
            _ => Ok(AstType::Void),
        },

        // Option types
        Expression::Some(v) => {
            let inner = compiler.infer_expression_type(v)?;
            let name = compiler.well_known.get_variant_parent_name(compiler.well_known.some_name())
                .unwrap_or(compiler.well_known.option_name()).to_string();
            Ok(AstType::Generic { name, type_args: vec![inner] })
        }
        Expression::None => {
            let name = compiler.well_known.get_variant_parent_name(compiler.well_known.none_name())
                .unwrap_or(compiler.well_known.option_name()).to_string();
            let inner = compiler.generic_type_context.get("Option_Some_Type").cloned().unwrap_or(AstType::Void);
            Ok(AstType::Generic { name, type_args: vec![inner] })
        }

        // Struct literal - use TypeContext
        Expression::StructLiteral { name, .. } => {
            let fields = compiler.type_ctx.get_struct_fields(name).cloned()
                .or_else(|| compiler.struct_types.get(name).map(|s| s.fields.iter().map(|(n, (_, t))| (n.clone(), t.clone())).collect()))
                .unwrap_or_default();
            Ok(AstType::Struct { name: name.clone(), fields })
        }

        // Collection constructors
        Expression::VecConstructor { element_type, size, .. } =>
            Ok(AstType::Vec { element_type: Box::new(element_type.clone()), size: *size }),
        Expression::DynVecConstructor { element_types, .. } =>
            Ok(AstType::DynVec { element_types: element_types.clone(), allocator_type: None }),
        // TODO: "Array" should come from TypeContext, not hardcoded
        // The typechecker should register this when parsing stdlib/collections/array.zen
        Expression::ArrayConstructor { element_type } =>
            Ok(AstType::Generic { name: "Array".to_string(), type_args: vec![element_type.clone()] }),

        // Type cast returns target type
        Expression::TypeCast { target_type, .. } => Ok(target_type.clone()),

        // Closure - use declared type or infer
        Expression::Closure { return_type, body, .. } => {
            if let Some(t) = return_type { Ok(t.clone()) } else {
                // Try to infer from the closure body
                infer_closure_return_type(compiler, body)
            }
        }
        // Control flow
        Expression::Loop { .. } | Expression::CollectionLoop { .. } | Expression::Continue { .. } => Ok(AstType::Void),
        Expression::Return(e) => compiler.infer_expression_type(e),
        Expression::Break { value, .. } => value.as_ref().map(|v| compiler.infer_expression_type(v)).unwrap_or(Ok(AstType::Void)),

        // Member access - use TypeContext for struct fields
        Expression::MemberAccess { object, member } => {
            let obj_type = compiler.infer_expression_type(object)?;
            let struct_name = match &obj_type {
                AstType::Struct { name, .. } => Some(name.as_str()),
                AstType::Generic { name, .. } if compiler.type_ctx.has_struct(name) => Some(name.as_str()),
                t if t.is_ptr_type() => t.ptr_inner().and_then(|i| match i { AstType::Struct { name, .. } => Some(name.as_str()), _ => None }),
                _ => None,
            };
            struct_name.map(|n| lookup_struct_field_type(compiler, n, member)).unwrap_or(Ok(AstType::Void))
        }

        // Pointer operations
        Expression::CreateReference(i) | Expression::AddressOf(i) => Ok(AstType::ptr(infer_expression_type(compiler, i)?)),
        Expression::CreateMutableReference(i) => Ok(AstType::mut_ptr(infer_expression_type(compiler, i)?)),
        Expression::Dereference(i) | Expression::PointerDereference(i) =>
            Ok(infer_expression_type(compiler, i)?.ptr_inner().cloned().unwrap_or(AstType::Void)),
        Expression::PointerAddress(_) => Ok(AstType::Usize),
        Expression::PointerOffset { pointer, .. } => infer_expression_type(compiler, pointer),

        _ => Ok(AstType::Void),
    }
}

/// Infer function call return type - uses TypeContext, handles intrinsics and generics
fn infer_function_call_type(compiler: &LLVMCompiler, name: &str) -> Result<AstType, CompileError> {
    // Range constructors
    if name == "Range.new" || name == "Range.with_step" {
        return Ok(AstType::Struct {
            name: "Range".to_string(),
            fields: vec![("current".to_string(), AstType::I64), ("end".to_string(), AstType::I64), ("step".to_string(), AstType::I64)],
        });
    }
    // Compiler/builtin intrinsics
    if name.starts_with("compiler.") || name.starts_with("builtin.") {
        let method = &name[if name.starts_with("compiler.") { 9 } else { 8 }..];
        let base = method.find('<').map(|p| &method[..p]).unwrap_or(method);
        if let Some(t) = compiler_intrinsics::get_intrinsic_return_type(base) { return Ok(t); }
    }
    // Generic type constructor (e.g., HashMap<K,V>.new())
    if name.contains('<') && name.contains('>') && !name.starts_with("compiler.") && !name.starts_with("builtin.") {
        if let Some(p) = name.find('<') {
            let base = &name[..p];
            let args = super::utils::parse_type_args_string(compiler, &name[p + 1..name.len() - 1])?;
            return Ok(AstType::Generic { name: base.to_string(), type_args: args });
        }
    }
    // TypeContext lookup
    if let Some(t) = compiler.get_function_return_type(name) { return Ok(t); }
    // Function pointer variable
    if let Ok((_, var_type)) = compiler.get_variable(name) {
        if let AstType::Function { return_type, .. } | AstType::FunctionPointer { return_type, .. } = var_type {
            return Ok(return_type.as_ref().clone());
        }
    }
    Ok(AstType::I32) // Default fallback
}

pub fn infer_closure_return_type(compiler: &LLVMCompiler, body: &Expression) -> Result<AstType, CompileError> {
    match body {
        Expression::QuestionMatch { arms, .. } => {
            // Find first non-void return type in any arm
            for arm in arms {
                if let Expression::Block(statements) = &arm.body {
                    for stmt in statements {
                        if let crate::ast::Statement::Return { expr, .. } = stmt {
                            let t = infer_expression_type(compiler, expr)?;
                            if t != AstType::Void { return Ok(t); }
                        }
                    }
                }
            }
            Ok(AstType::Void)
        }
        Expression::Block(statements) => {
            // Look for return or last expression
            for stmt in statements {
                if let crate::ast::Statement::Return { expr, .. } = stmt {
                    return infer_expression_type(compiler, expr);
                }
            }
            if let Some(last) = statements.last() {
                if let crate::ast::Statement::Expression { expr, .. } | crate::ast::Statement::Return { expr, .. } = last {
                    return infer_expression_type(compiler, expr);
                }
            }
            Ok(AstType::Void)
        }
        Expression::FunctionCall { name, args } => infer_well_known_call_type(compiler, name, args),
        _ => infer_expression_type(compiler, body),
    }
}

/// Infer type for well-known function calls (Result.Ok, Option.Some, etc.)
fn infer_well_known_call_type(compiler: &LLVMCompiler, name: &str, args: &[Expression]) -> Result<AstType, CompileError> {
    let wk = &compiler.well_known;
    let arg_type = |idx: usize| args.get(idx).map(|e| infer_expression_type(compiler, e)).transpose();

    match name {
        "Result.Ok" => {
            let t = arg_type(0)?.unwrap_or(AstType::Void);
            let result_name = wk.get_variant_parent_name(wk.ok_name()).unwrap_or(wk.result_name());
            Ok(AstType::Generic { name: result_name.to_string(), type_args: vec![t, AstType::StaticString] })
        }
        "Result.Err" => {
            let e = arg_type(0)?.unwrap_or_else(crate::ast::resolve_string_struct_type);
            let result_name = wk.get_variant_parent_name(wk.err_name()).unwrap_or(wk.result_name());
            Ok(AstType::Generic { name: result_name.to_string(), type_args: vec![AstType::I32, e] })
        }
        "Option.Some" => {
            let t = arg_type(0)?.unwrap_or(AstType::Void);
            let option_name = wk.get_variant_parent_name(wk.some_name()).unwrap_or(wk.option_name());
            Ok(AstType::Generic { name: option_name.to_string(), type_args: vec![t] })
        }
        "Option.None" => {
            let option_name = wk.get_variant_parent_name(wk.none_name()).unwrap_or(wk.option_name());
            Ok(AstType::Generic { name: option_name.to_string(), type_args: vec![AstType::Generic { name: "T".to_string(), type_args: vec![] }] })
        }
        _ => compiler.get_function_return_type(name).map(Ok).unwrap_or(Ok(AstType::I32))
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
    compiler: &LLVMCompiler, variant: &str, payload: &Option<Box<Expression>>
) -> Result<AstType, CompileError> {
    let wk = &compiler.well_known;
    let parent = wk.get_variant_parent_name(variant).unwrap_or(wk.option_name()).to_string();

    let inner = if wk.is_some(variant) {
        payload.as_ref().map(|p| infer_expression_type(compiler, p)).transpose()?
    } else { None };

    let t = inner
        .or_else(|| compiler.generic_type_context.get("Option_Some_Type").cloned())
        .unwrap_or_else(|| AstType::Generic { name: "T".to_string(), type_args: vec![] });

    Ok(AstType::Generic { name: parent, type_args: vec![t] })
}

/// Infer type for Result variants (Ok/Err)
fn infer_result_variant_type(
    compiler: &LLVMCompiler, variant: &str, payload: &Option<Box<Expression>>
) -> Result<AstType, CompileError> {
    let wk = &compiler.well_known;
    let parent = wk.get_variant_parent_name(variant).unwrap_or(wk.result_name()).to_string();
    let ctx = &compiler.generic_type_context;

    let payload_type = payload.as_ref().map(|p| infer_expression_type(compiler, p)).transpose()?;

    let (ok_t, err_t) = if wk.is_ok(variant) && payload_type.is_some() {
        (payload_type.unwrap(), ctx.get("Result_Err_Type").cloned().unwrap_or(AstType::StaticString))
    } else if wk.is_err(variant) && payload_type.is_some() {
        (ctx.get("Result_Ok_Type").cloned().unwrap_or(AstType::Void), payload_type.unwrap())
    } else {
        (ctx.get("Result_Ok_Type").cloned().unwrap_or(AstType::Void),
         ctx.get("Result_Err_Type").cloned().unwrap_or(AstType::Void))
    };

    Ok(AstType::Generic { name: parent, type_args: vec![ok_t, err_t] })
}

/// Infer type for custom enum variants (uses TypeContext first)
fn infer_custom_enum_type(
    compiler: &LLVMCompiler,
    enum_name: &str,
) -> Result<AstType, CompileError> {
    // Try TypeContext first
    if let Some(variants) = compiler.type_ctx.get_enum_variants(enum_name) {
        let ast_variants = variants.iter()
            .map(|(name, payload)| crate::ast::EnumVariant { name: name.clone(), payload: payload.clone() })
            .collect();
        return Ok(AstType::Enum { name: enum_name.to_string(), variants: ast_variants });
    }
    // Fall back to symbol table
    if let Some(symbols::Symbol::EnumType(enum_info)) = compiler.symbols.lookup(enum_name) {
        let variants = enum_info.variants.iter()
            .map(|v| crate::ast::EnumVariant { name: v.name.clone(), payload: v.payload.clone() })
            .collect();
        return Ok(AstType::Enum { name: enum_name.to_string(), variants });
    }
    Ok(AstType::EnumType { name: enum_name.to_string() })
}

/// Infer type for method call expressions
fn infer_method_call_type(compiler: &LLVMCompiler, object: &Expression, method: &str) -> Result<AstType, CompileError> {
    // Compiler intrinsics
    if let Expression::Identifier(n) = object {
        if n == "compiler" {
            let base = method.find('<').map(|p| &method[..p]).unwrap_or(method);
            if let Some(t) = compiler_intrinsics::get_intrinsic_return_type(base) { return Ok(t); }
        }
    }
    // Raise method unwraps Result<T,E> to T
    if method == "raise" {
        if let AstType::Generic { name, type_args } = compiler.infer_expression_type(object)? {
            if compiler.well_known.is_result(&name) && type_args.len() == 2 { return Ok(type_args[0].clone()); }
        }
        return Ok(AstType::Void);
    }
    // Constructors
    let base = method.find('<').map(|p| &method[..p]).unwrap_or(method);
    if matches!(base, "new" | "init" | "with_step") {
        return infer_constructor_type(compiler, object, method);
    }
    infer_common_method_type(compiler, object, method)
}

/// Infer type for constructor methods (new/init)
fn infer_constructor_type(compiler: &LLVMCompiler, object: &Expression, method: &str) -> Result<AstType, CompileError> {
    let Expression::Identifier(name) = object else { return Ok(AstType::Void); };

    // Generic type args in method (HashMap.new<K,V>())
    if let Some(p) = method.find('<') {
        let args = super::utils::parse_type_args_string(compiler, &method[p + 1..method.len() - 1])?;
        return Ok(AstType::Generic { name: name.to_string(), type_args: args });
    }
    // Generic type in name (HashMap<K,V>.new())
    if let Some(p) = name.find('<') {
        let args = super::utils::parse_type_args_string(compiler, &name[p + 1..name.len() - 1])?;
        return Ok(AstType::Generic { name: name[..p].to_string(), type_args: args });
    }
    // Type inference from TypeContext.structs
    let make_struct = |n: &str| AstType::Struct { name: n.to_string(), fields: vec![] };

    // Check if type exists in TypeContext.structs (from stdlib parsing)
    if compiler.type_ctx.structs.contains_key(name) {
        return Ok(make_struct(name));
    }

    // Fallback for known types
    Ok(match name.as_str() {
        "GPA" | "AsyncPool" | "String" | "Range" => make_struct(name),
        _ => AstType::Void,
    })
}

/// Infer type for common methods by name.
///
/// Priority order:
/// 1. TypeContext (populated from stdlib .zen files by typechecker)
/// 2. Hardcoded fallbacks (temporary, should be removed as stdlib coverage improves)
fn infer_common_method_type(compiler: &LLVMCompiler, object: &Expression, method: &str) -> Result<AstType, CompileError> {
    let opt = compiler.well_known.get_variant_parent_name(compiler.well_known.some_name())
        .unwrap_or(compiler.well_known.option_name()).to_string();
    let wrap = |t| AstType::Generic { name: opt.clone(), type_args: vec![t] };

    // FIRST: Try TypeContext - this has method info populated from stdlib .zen files
    if let Ok(t) = infer_expression_type(compiler, object) {
        if let AstType::Struct { name, .. } | AstType::Generic { name, .. } = &t {
            // Try TypeContext method registry (populated by typechecker from impl blocks)
            if let Some(return_type) = compiler.type_ctx.get_method_return_type(name, method) {
                return Ok(return_type.clone());
            }
        }
    }

    // FALLBACK: Hardcoded method return types
    // TODO: Remove these as stdlib impl blocks are properly parsed and registered
    match method {
        "abs" | "min" | "max" => return compiler.infer_expression_type(object),
        "len" | "size" | "length" | "index_of" | "count" => return Ok(AstType::I64),
        "is_empty" | "contains" | "starts_with" | "ends_with" | "add" | "has_next" |
        "is_subset" | "is_superset" | "is_disjoint" => return Ok(AstType::Bool),
        "push" | "set" | "clear" => return Ok(AstType::Void),
        "substr" | "trim" | "to_upper" | "to_lower" => return Ok(AstType::StaticString),
        "char_at" => return Ok(AstType::I32),
        "split" => {
            // TODO: should come from TypeContext String::split return type
            return Ok(AstType::Generic { name: "Array".to_string(), type_args: vec![AstType::StaticString] });
        }
        "to_i32" => return Ok(wrap(AstType::I32)),
        "to_i64" | "next" => return Ok(wrap(AstType::I64)), // next returns Option<i64> for Range
        "to_f64" => return Ok(wrap(AstType::F64)),
        _ => {}
    }
    // Collection methods - infer from generic type args
    // NOTE: Removed IntrinsicLayout-based dispatch. TypeContext.methods should handle this.
    // Collection method inference using centralized type categories
    use crate::type_context::{is_key_value_collection, is_single_element_collection};
    if matches!(method, "get" | "remove" | "insert" | "pop") {
        if let Some(t) = compiler.infer_expression_type(object).ok() {
            match &t {
                AstType::Generic { name, type_args } if !type_args.is_empty() => {
                    // Key-value collections: get/remove return Option<V> (type_args[1])
                    if is_key_value_collection(name) && type_args.len() >= 2 {
                        if matches!(method, "get" | "remove") {
                            return Ok(wrap(type_args[1].clone()));
                        }
                    }
                    // Single-element collections: get returns T, pop returns Option<T>
                    if is_single_element_collection(name) {
                        return match method {
                            "get" => Ok(type_args[0].clone()),
                            "pop" => Ok(wrap(type_args[0].clone())),
                            "remove" if name == "HashSet" || name == "Set" => Ok(AstType::Bool),
                            _ => Ok(AstType::Void),
                        };
                    }
                }
                AstType::DynVec { element_types, .. } if !element_types.is_empty() => return Ok(wrap(element_types[0].clone())),
                _ => {}
            }
        }
    }
    // Set operations return the same type
    if matches!(method, "union" | "intersection" | "difference" | "symmetric_difference") {
        if let Ok(AstType::Generic { name, type_args }) = compiler.infer_expression_type(object) {
            if is_single_element_collection(&name) {
                return Ok(AstType::Generic { name, type_args });
            }
        }
    }
    // UFC lookup - try TypeContext (with stdlib_types fallback built in)
    if let Ok(t) = infer_expression_type(compiler, object) {
        if let AstType::Struct { name, .. } | AstType::Generic { name, .. } = &t {
            if let Some(r) = compiler.get_function_return_type(&format!("{}.{}", name, method)) { return Ok(r); }
            if let Some(r) = compiler.type_ctx.get_method_return_type(name, method) { return Ok(r); }
        }
    }
    compiler.get_function_return_type(method).map(Ok).unwrap_or(Ok(AstType::Void))
}
