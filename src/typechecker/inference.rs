use crate::ast::{AstType, BinaryOperator, Expression};
use crate::error::{CompileError, Result};
use crate::stdlib_types::StdlibTypeRegistry;
use crate::type_context::{is_key_value_collection, is_single_element_collection};
use crate::typechecker::{EnumInfo, StructInfo, TypeChecker};
use crate::well_known::well_known;
use std::collections::HashMap;

// ============================================================================
// HELPER FUNCTIONS FOR REDUCING DUPLICATION
// ============================================================================

/// Extract the type name from common AstType variants (Struct, Generic, Enum)
fn extract_type_name(ast_type: &AstType) -> Option<&str> {
    match ast_type {
        AstType::Struct { name, .. } => Some(name.as_str()),
        AstType::Generic { name, .. } => Some(name.as_str()),
        AstType::Enum { name, .. } => Some(name.as_str()),
        _ => None,
    }
}

/// Check if a type is any string type (static or dynamic)
fn is_string_type(ast_type: &AstType) -> bool {
    matches!(ast_type, AstType::StaticString | AstType::StaticLiteral)
        || matches!(ast_type, AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name))
}

/// Infer the type of a binary operation
#[allow(dead_code)]
pub fn infer_binary_op_type(
    checker: &mut TypeChecker,
    left: &Expression,
    op: &BinaryOperator,
    right: &Expression,
) -> Result<AstType> {
    let mut left_type = checker.infer_expression_type(left)?;
    let mut right_type = checker.infer_expression_type(right)?;

    // Handle unresolved generic types by defaulting to appropriate concrete types
    // This happens when Option.None creates Option<T> or Result.Ok/Err creates Result<T,E>
    if let AstType::Generic { name, type_args } = &left_type {
        if name == "T" && type_args.is_empty() {
            left_type = AstType::I32;
        } else if name == "E" && type_args.is_empty() {
            // Error types default to String (dynamic with allocator)
            left_type = crate::ast::resolve_string_struct_type();
        }
    }
    if let AstType::Generic { name, type_args } = &right_type {
        if name == "T" && type_args.is_empty() {
            right_type = AstType::I32;
        } else if name == "E" && type_args.is_empty() {
            // Error types default to String (dynamic with allocator)
            right_type = crate::ast::resolve_string_struct_type();
        }
    }

    match op {
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Modulo => {
            // Numeric operations
            if left_type.is_numeric() && right_type.is_numeric() {
                // Promote to the larger type
                promote_numeric_types(&left_type, &right_type, checker.get_current_span())
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "Cannot apply {:?} to types {:?} and {:?}",
                        op, left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
        BinaryOperator::Equals
        | BinaryOperator::NotEquals
        | BinaryOperator::LessThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::LessThanEquals
        | BinaryOperator::GreaterThanEquals => {
            // Comparison operations return bool
            if types_comparable(&left_type, &right_type) {
                Ok(AstType::Bool)
            } else {
                Err(CompileError::TypeError(
                    format!("Cannot compare types {:?} and {:?}", left_type, right_type),
                    checker.get_current_span(),
                ))
            }
        }
        BinaryOperator::And | BinaryOperator::Or => {
            // Logical operations
            if matches!(left_type, AstType::Bool) && matches!(right_type, AstType::Bool) {
                Ok(AstType::Bool)
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "Logical operators require boolean operands, got {:?} and {:?}",
                        left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
        BinaryOperator::StringConcat => {
            // String concatenation - all string types can be concatenated
            if is_string_type(&left_type) && is_string_type(&right_type) {
                // Result is always String (dynamic) when concatenating
                Ok(crate::ast::resolve_string_struct_type())
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "String concatenation requires string operands, got {:?} and {:?}",
                        left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
    }
}

/// Infer the type of a member access expression
#[allow(dead_code)]
pub fn infer_member_type(
    object_type: &AstType,
    member: &str,
    structs: &HashMap<String, StructInfo>,
    enums: &HashMap<String, EnumInfo>,
    span: Option<crate::error::Span>,
) -> Result<AstType> {
    match object_type {
        AstType::Struct { name, .. } => {
            if let Some(struct_info) = structs.get(name) {
                for (field_name, field_type) in &struct_info.fields {
                    if field_name == member {
                        return Ok(field_type.clone());
                    }
                }
                Err(CompileError::TypeError(
                    format!("Struct '{}' has no field '{}'", name, member),
                    span,
                ))
            } else {
                Err(CompileError::TypeError(
                    format!("Unknown struct type: {}", name),
                    span,
                ))
            }
        }
        // Handle pointer to struct types
        t if t.is_ptr_type() => {
            // Dereference the pointer and check the inner type
            if let Some(inner) = t.ptr_inner() {
                infer_member_type(inner, member, structs, enums, span)
            } else {
                Err(CompileError::TypeError(
                    format!("Type '{}' is not a struct", t),
                    span,
                ))
            }
        }
        // Handle Generic types that represent structs
        AstType::Generic { name, .. } => {
            // Try to look up the struct info by name
            if let Some(struct_info) = structs.get(name) {
                for (field_name, field_type) in &struct_info.fields {
                    if field_name == member {
                        return Ok(field_type.clone());
                    }
                }
                Err(CompileError::TypeError(
                    format!("Struct '{}' has no field '{}'", name, member),
                    span,
                ))
            } else {
                Err(CompileError::TypeError(
                    format!("Type '{}' is not a struct or is not defined", name),
                    span,
                ))
            }
        }
        // Handle enum type constructors
        AstType::EnumType { name } => {
            // Check if the member is a valid variant of this enum
            if let Some(enum_info) = enums.get(name) {
                for (variant_name, _variant_type) in &enum_info.variants {
                    if variant_name == member {
                        // Return the enum type itself - the variant constructor creates an instance of the enum
                        let enum_variants = enum_info
                            .variants
                            .iter()
                            .map(|(name, payload)| crate::ast::EnumVariant {
                                name: name.clone(),
                                payload: payload.clone(),
                            })
                            .collect();
                        return Ok(AstType::Enum {
                            name: name.clone(),
                            variants: enum_variants,
                        });
                    }
                }
                Err(CompileError::TypeError(
                    format!("Enum '{}' has no variant '{}'", name, member),
                    span,
                ))
            } else {
                Err(CompileError::TypeError(
                    format!("Unknown enum type: {}", name),
                    span,
                ))
            }
        }
        AstType::StdModule => {
            // Handle stdlib module member access (e.g., math.pi, GPA.init)
            // TODO: Implement a proper registry of stdlib module members
            match member {
                "pi" => Ok(AstType::F64),
                "init" => {
                    // For allocator modules, init() returns an allocator type
                    // We'll use a generic type for now
                    Ok(AstType::Generic {
                        name: "Allocator".to_string(),
                        type_args: vec![],
                    })
                }
                _ => Err(CompileError::TypeError(
                    format!("Unknown stdlib module member: {}", member),
                    span,
                )),
            }
        }
        _ => Err(CompileError::TypeError(
            format!(
                "Cannot access member '{}' on type {:?}",
                member, object_type
            ),
            span,
        )),
    }
}

/// Promote two numeric types to their common type
#[allow(dead_code)]
fn promote_numeric_types(left: &AstType, right: &AstType, span: Option<crate::error::Span>) -> Result<AstType> {
    // If either is a float, promote to float
    if left.is_float() || right.is_float() {
        if matches!(left, AstType::F64) || matches!(right, AstType::F64) {
            Ok(AstType::F64)
        } else {
            Ok(AstType::F32)
        }
    }
    // Both are integers
    else if left.is_integer() && right.is_integer() {
        // Special case: if both are Usize, keep Usize
        if matches!(left, AstType::Usize) && matches!(right, AstType::Usize) {
            return Ok(AstType::Usize);
        }
        // If one is Usize and other is compatible unsigned, promote to Usize
        if (matches!(left, AstType::Usize) || matches!(right, AstType::Usize))
            && left.is_unsigned_integer()
            && right.is_unsigned_integer()
        {
            return Ok(AstType::Usize);
        }

        // Promote to the larger size
        let left_size = left.bit_size().unwrap_or(32);
        let right_size = right.bit_size().unwrap_or(32);
        let max_size = left_size.max(right_size);

        // If either is unsigned and the other is signed, need special handling
        if left.is_unsigned_integer() != right.is_unsigned_integer() {
            // For now, promote to signed of the appropriate size
            match max_size {
                8 => Ok(AstType::I8),
                16 => Ok(AstType::I16),
                32 => Ok(AstType::I32),
                64 => Ok(AstType::I64),
                _ => Ok(AstType::I32),
            }
        } else if left.is_unsigned_integer() {
            // Both unsigned
            match max_size {
                8 => Ok(AstType::U8),
                16 => Ok(AstType::U16),
                32 => Ok(AstType::U32),
                64 => Ok(AstType::U64),
                _ => Ok(AstType::U32),
            }
        } else {
            // Both signed
            match max_size {
                8 => Ok(AstType::I8),
                16 => Ok(AstType::I16),
                32 => Ok(AstType::I32),
                64 => Ok(AstType::I64),
                _ => Ok(AstType::I32),
            }
        }
    } else {
        Err(CompileError::TypeError(
            format!("Cannot promote types {:?} and {:?}", left, right),
            span,
        ))
    }
}

fn types_comparable(left: &AstType, right: &AstType) -> bool {
    if std::mem::discriminant(left) == std::mem::discriminant(right) {
        return true;
    }

    if left.is_numeric() && right.is_numeric() {
        return true;
    }

    if left.is_ptr_type() && right.is_ptr_type() {
        return true;
    }

    if is_string_type(left) && is_string_type(right) {
        return true;
    }

    if matches!(left, AstType::Bool) && matches!(right, AstType::Bool) {
        return true;
    }

    false
}

pub fn infer_identifier_type(checker: &mut TypeChecker, name: &str) -> Result<AstType> {
    if let Ok(var_type) = checker.get_variable_type(name) {
        return Ok(var_type);
    }

    // Built-in modules are always available without explicit import
    if crate::intrinsics::is_builtin_module(name) {
        return Ok(AstType::StdModule);
    }

    if let Some(sig) = checker.get_function_signatures().get(name) {
        return Ok(AstType::FunctionPointer {
            param_types: sig.params.iter().map(|(_, t)| t.clone()).collect(),
            return_type: Box::new(sig.return_type.clone()),
        });
    }

    // Check if name is a collection type
    if is_single_element_collection(name) || is_key_value_collection(name) {
        return Ok(AstType::Generic {
            name: name.to_string(),
            type_args: vec![],
        });
    }

    if name.contains('<') {
        if let Some(angle_pos) = name.find('<') {
            let base_type = &name[..angle_pos];

            let wk = &checker.well_known;
            if wk.is_immutable_ptr(base_type) {
                let (_, type_args) = TypeChecker::parse_generic_type_string(name);
                let inner = type_args.first().cloned().unwrap_or(AstType::U8);
                return Ok(AstType::ptr(inner));
            } else if wk.is_mutable_ptr(base_type) {
                let (_, type_args) = TypeChecker::parse_generic_type_string(name);
                let inner = type_args.first().cloned().unwrap_or(AstType::U8);
                return Ok(AstType::mut_ptr(inner));
            } else if wk.is_raw_ptr(base_type) {
                let (_, type_args) = TypeChecker::parse_generic_type_string(name);
                let inner = type_args.first().cloned().unwrap_or(AstType::U8);
                return Ok(AstType::raw_ptr(inner));
            } else if is_single_element_collection(base_type)
                || is_key_value_collection(base_type)
                || wk.is_option(base_type)
                || wk.is_result(base_type)
            {
                let (_, type_args) = TypeChecker::parse_generic_type_string(name);
                return Ok(AstType::Generic {
                    name: base_type.to_string(),
                    type_args,
                });
            }
        }
    }

    Err(CompileError::UndeclaredVariable(
        name.to_string(),
        checker.get_current_span(),
    ))
}

pub fn infer_function_call_type(
    checker: &mut TypeChecker,
    name: &str,
    args: &[Expression],
) -> Result<AstType> {
    use super::intrinsics;

    if name.contains('.') {
        let parts: Vec<&str> = name.splitn(2, '.').collect();
        if parts.len() == 2 {
            let module = parts[0];
            let func = parts[1];

            let base_func = if let Some(angle_pos) = func.find('<') {
                &func[..angle_pos]
            } else {
                func
            };

            if let Some(result) =
                intrinsics::check_compiler_intrinsic(module, base_func, args.len())
            {
                if module == "compiler" && base_func == "inline_c" && args.len() == 1 {
                    let arg_type = checker.infer_expression_type(&args[0])?;
                    match arg_type {
                        AstType::StaticString | AstType::StaticLiteral => {}
                        _ => {
                            return Err(CompileError::TypeError(
                                "compiler.inline_c() requires a string literal argument"
                                    .to_string(),
                                checker.get_current_span(),
                            ))
                        }
                    }
                }
                return result;
            }

            // Check TypeContext (with stdlib_types fallback built in)
            if let Some(return_type) = checker.type_context.get_function_return_type_with_fallback(module, base_func) {
                return Ok(return_type);
            }

            // Handle generic constructors like HashMap.new<K, V> or Vec.new<T>
            if base_func == "new" && (is_single_element_collection(module) || is_key_value_collection(module)) {
                // Parse type args from func, e.g., "new<i32, i32>"
                if let Some(angle_pos) = func.find('<') {
                    let type_args_str = &func[angle_pos + 1..func.len() - 1];
                    let type_args = parse_type_args(type_args_str);
                    return Ok(AstType::Generic {
                        name: module.to_string(),
                        type_args,
                    });
                }
            }
        }
    }

    if name == "cast" {
        return infer_cast_type(args, checker.get_current_span());
    }

    if name.contains('<') && name.contains('>') {
        if let Some(angle_pos) = name.find('<') {
            let base_type = &name[..angle_pos];
            // Check if this is a collection type
            if is_single_element_collection(base_type) || is_key_value_collection(base_type) {
                // Parse type args from the generic syntax, e.g. "HashMap<i32, i32>"
                let type_args_str = &name[angle_pos + 1..name.len() - 1];
                let type_args = parse_type_args(type_args_str);
                return Ok(AstType::Generic {
                    name: base_type.to_string(),
                    type_args,
                });
            }
        }
    }

    if let Some(sig) = checker.get_function_signatures().get(name) {
        return Ok(sig.return_type.clone());
    }

    match checker.get_variable_type(name) {
        Ok(AstType::FunctionPointer { return_type, .. }) => Ok(*return_type),
        Ok(_) => Err(CompileError::TypeError(
            format!("'{}' is not a function", name),
            checker.get_current_span(),
        )),
        Err(_) => Err(CompileError::TypeError(
            format!("Unknown function: {}", name),
            checker.get_current_span(),
        )),
    }
}

fn infer_cast_type(args: &[Expression], span: Option<crate::error::Span>) -> Result<AstType> {
    if args.len() == 2 {
        if let Expression::Identifier(type_name) = &args[1] {
            return match type_name.as_str() {
                "i8" => Ok(AstType::I8),
                "i16" => Ok(AstType::I16),
                "i32" => Ok(AstType::I32),
                "i64" => Ok(AstType::I64),
                "u8" => Ok(AstType::U8),
                "u16" => Ok(AstType::U16),
                "u32" => Ok(AstType::U32),
                "u64" => Ok(AstType::U64),
                "usize" => Ok(AstType::Usize),
                "f32" => Ok(AstType::F32),
                "f64" => Ok(AstType::F64),
                _ => Err(CompileError::TypeError(
                    format!(
                        "cast() target type '{}' is not a valid primitive type",
                        type_name
                    ),
                    span.clone(),
                )),
            };
        }
    }
    Err(CompileError::TypeError(
        "cast() expects 2 arguments: cast(value, type)".to_string(),
        span,
    ))
}

/// Parse type arguments from a string like "i32, i32" into Vec<AstType>
fn parse_type_args(type_args_str: &str) -> Vec<AstType> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut current = String::new();

    for ch in type_args_str.chars() {
        match ch {
            '<' => {
                depth += 1;
                current.push(ch);
            }
            '>' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    result.push(string_to_ast_type(trimmed));
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        result.push(string_to_ast_type(trimmed));
    }

    result
}

/// Convert a type name string to AstType
fn string_to_ast_type(s: &str) -> AstType {
    match s {
        "i8" => AstType::I8,
        "i16" => AstType::I16,
        "i32" => AstType::I32,
        "i64" => AstType::I64,
        "u8" => AstType::U8,
        "u16" => AstType::U16,
        "u32" => AstType::U32,
        "u64" => AstType::U64,
        "usize" => AstType::Usize,
        "f32" => AstType::F32,
        "f64" => AstType::F64,
        "bool" => AstType::Bool,
        "void" => AstType::Void,
        "StaticString" => AstType::StaticString,
        "String" => crate::ast::resolve_string_struct_type(),
        _ => {
            // Check for generic types like Option<i32>
            if let Some(angle_pos) = s.find('<') {
                let base = &s[..angle_pos];
                let inner = &s[angle_pos + 1..s.len() - 1];
                AstType::Generic {
                    name: base.to_string(),
                    type_args: parse_type_args(inner),
                }
            } else {
                // Assume it's a named type
                AstType::Generic {
                    name: s.to_string(),
                    type_args: vec![],
                }
            }
        }
    }
}

pub fn infer_struct_field_type(
    struct_type: &AstType,
    field: &str,
    structs: &HashMap<String, StructInfo>,
    enums: &HashMap<String, EnumInfo>,
    span: Option<crate::error::Span>,
) -> Result<AstType> {
    match struct_type {
        t if t.is_ptr_type() => {
            if let Some(inner) = t.ptr_inner() {
                match inner {
                    AstType::Struct { name, .. } => infer_member_type(
                        &AstType::Struct {
                            name: name.clone(),
                            fields: vec![],
                        },
                        field,
                        structs,
                        enums,
                        span,
                    ),
                    AstType::Generic { name, .. } => infer_member_type(
                        &AstType::Generic {
                            name: name.clone(),
                            type_args: vec![],
                        },
                        field,
                        structs,
                        enums,
                        span,
                    ),
                    _ => Err(CompileError::TypeError(
                        format!("Cannot access field '{}' on non-struct pointer type", field),
                        span,
                    )),
                }
            } else {
                Err(CompileError::TypeError(
                    format!("Cannot access field '{}' on type {:?}", field, struct_type),
                    span,
                ))
            }
        }
        AstType::Struct { .. } | AstType::Generic { .. } => {
            infer_member_type(struct_type, field, structs, enums, span)
        }
        _ => Err(CompileError::TypeError(
            format!("Cannot access field '{}' on type {:?}", field, struct_type),
            span,
        )),
    }
}

pub fn infer_enum_literal_type(
    checker: &mut TypeChecker,
    variant: &str,
    payload: &Option<Box<Expression>>,
) -> Result<AstType> {
    let wk = well_known();

    if wk.is_some(variant) {
        let inner_type = if let Some(p) = payload {
            checker.infer_expression_type(p)?
        } else {
            AstType::Void
        };
        Ok(AstType::Generic {
            name: wk.get_variant_parent_name(variant).unwrap().to_string(),
            type_args: vec![inner_type],
        })
    } else if wk.is_none(variant) {
        Ok(AstType::Generic {
            name: wk.get_variant_parent_name(variant).unwrap().to_string(),
            type_args: vec![AstType::Generic {
                name: "T".to_string(),
                type_args: vec![],
            }],
        })
    } else if wk.is_ok(variant) {
        let ok_type = if let Some(p) = payload {
            checker.infer_expression_type(p)?
        } else {
            AstType::Void
        };
        Ok(AstType::Generic {
            name: wk.get_variant_parent_name(variant).unwrap().to_string(),
            type_args: vec![ok_type, crate::ast::resolve_string_struct_type()],
        })
    } else if wk.is_err(variant) {
        let err_type = if let Some(p) = payload {
            checker.infer_expression_type(p)?
        } else {
            AstType::Void
        };
        Ok(AstType::Generic {
            name: wk.get_variant_parent_name(variant).unwrap().to_string(),
            type_args: vec![
                AstType::Generic {
                    name: "T".to_string(),
                    type_args: vec![],
                },
                err_type,
            ],
        })
    } else {
        Ok(AstType::Void)
    }
}

pub fn infer_raise_type(checker: &mut TypeChecker, expr: &Expression) -> Result<AstType> {
    let result_type = checker.infer_expression_type(expr)?;
    match result_type {
        AstType::Generic { ref name, ref type_args }
            if checker.well_known.is_result(name) && !type_args.is_empty() =>
        {
            Ok(type_args[0].clone())
        }
        _ => Err(CompileError::TypeError(
            format!(
                ".raise() can only be used on Result<T, E> types, found: {:?}",
                result_type
            ),
            checker.get_current_span(),
        )),
    }
}

pub fn infer_method_call_type(
    checker: &mut TypeChecker,
    object: &Expression,
    method: &str,
) -> Result<AstType> {
    use super::method_types;

    if let Expression::Identifier(name) = object {
        let base_name = name.split('<').next().unwrap_or(name);

        // Check TypeContext (with stdlib_types fallback built in)
        if let Some(return_type) = checker.type_context.get_method_return_type(base_name, method) {
            return Ok(return_type);
        }

        // Check for module functions
        if let Some(return_type) = checker.type_context.get_function_return_type_with_fallback(base_name, method) {
            return Ok(return_type);
        }

        // Extract base method name (e.g., "new" from "new<i32, i32>")
        let base_method = method.split('<').next().unwrap_or(method);

        if base_method == "new" {
            // Check if type args are in the method name (e.g., "new<i32, i32>")
            if method.contains('<') {
                // Parse type args from method, e.g., "new<i32, i32>" -> ["i32", "i32"]
                if let Some(angle_pos) = method.find('<') {
                    let type_args_str = &method[angle_pos + 1..method.len() - 1];
                    let type_args = parse_type_args(type_args_str);
                    return Ok(AstType::Generic {
                        name: name.to_string(),
                        type_args,
                    });
                }
            } else if is_single_element_collection(name.as_str()) || is_key_value_collection(name.as_str()) {
                // Collection type constructor without explicit type args - default to I32
                return Ok(AstType::Generic {
                    name: name.to_string(),
                    type_args: vec![AstType::I32],
                });
            } else if name.contains('<') {
                let (base_type, type_args) = TypeChecker::parse_generic_type_string(name);
                return Ok(AstType::Generic {
                    name: base_type,
                    type_args,
                });
            }
        }
    }

    let object_type = checker.infer_expression_type(object)?;

    let dereferenced_type = object_type.ptr_inner().map(|inner| inner.clone());

    let effective_type = dereferenced_type.as_ref().unwrap_or(&object_type);

    if let Some(func_type) = checker.get_function_signatures().get(method) {
        if !func_type.params.is_empty() {
            let (_, first_param_type) = &func_type.params[0];
            if first_param_type == effective_type || first_param_type == &object_type {
                return Ok(func_type.return_type.clone());
            }
        }
    }

    // Check TypeContext (with stdlib_types fallback built in)
    if let Some(type_name) = extract_type_name(effective_type) {
        if let Some(return_type) = checker.type_context.get_method_return_type(type_name, method) {
            return Ok(return_type);
        }
    }

    let is_string_struct =
        matches!(effective_type, AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name));
    if is_string_struct
        || *effective_type == AstType::StaticString
        || *effective_type == AstType::StaticLiteral
    {
        if let Some(return_type) = method_types::infer_string_method_type(method, is_string_struct)
        {
            return Ok(return_type);
        }
    }

    if method == "loop" {
        return Ok(AstType::Void);
    }

    // Collection method type inference based on type name
    if let AstType::Generic { name, type_args } = &object_type {
        match name.as_str() {
            "HashMap" | "BTreeMap" => {
                if let Some(return_type) = method_types::infer_hashmap_method_type(method, type_args) {
                    return Ok(return_type);
                }
            }
            "HashSet" | "Set" => {
                if let Some(return_type) = method_types::infer_hashset_method_type(method) {
                    return Ok(return_type);
                }
            }
            "Vec" | "Array" | "Stack" | "Queue" | "LinkedList" if !type_args.is_empty() => {
                if let Some(return_type) = method_types::infer_vec_method_type(method, &type_args[0]) {
                    return Ok(return_type);
                }
            }
            _ => {
                // Check well_known types (Result, Option)
                if checker.well_known.is_result(name) {
                    if let Some(return_type) = method_types::infer_result_method_type(method, type_args) {
                        return Ok(return_type);
                    }
                }
            }
        }
    }

    // Vec methods
    if let AstType::Vec { element_type, .. } = &object_type {
        if let Some(return_type) = method_types::infer_vec_method_type(method, element_type) {
            return Ok(return_type);
        }
    }

    // Pointer methods
    if dereferenced_type.is_none() {
        if let Some(inner) = object_type.ptr_inner() {
            if let Some(return_type) = method_types::infer_pointer_method_type(method, inner) {
                return Ok(return_type);
            }
        }
    }

    // Try trait/behavior method resolution
    if let Some(type_name) = extract_type_name(effective_type) {
        if let Some(method_info) = checker.resolve_trait_method(type_name, method) {
            return Ok(method_info.return_type);
        }
    }

    Ok(AstType::Void)
}

pub fn infer_enum_variant_type(
    enum_name: &str,
    variant: &str,
    enums: &HashMap<String, EnumInfo>,
) -> Result<AstType> {
    let wk = well_known();
    let enum_type_name = if enum_name.is_empty() {
        let mut found_enum = None;
        for (name, info) in enums {
            for (var_name, _) in &info.variants {
                if var_name == variant {
                    found_enum = Some(name.clone());
                    break;
                }
            }
            if found_enum.is_some() {
                break;
            }
        }
        found_enum.unwrap_or_else(|| {
            wk.get_variant_parent_name(variant)
                .unwrap_or(wk.option_name())
                .to_string()
        })
    } else {
        enum_name.to_string()
    };

    if let Some(enum_info) = enums.get(&enum_type_name) {
        let variants = enum_info
            .variants
            .iter()
            .map(|(name, payload)| crate::ast::EnumVariant {
                name: name.clone(),
                payload: payload.clone(),
            })
            .collect();
        Ok(AstType::Enum {
            name: enum_type_name,
            variants,
        })
    } else {
        if enum_type_name.contains('<') {
            let (base_name, type_args) =
                crate::typechecker::TypeChecker::parse_generic_type_string(&enum_type_name);

            if wk.is_immutable_ptr(&base_name) && type_args.len() == 1 {
                return Ok(AstType::ptr(type_args[0].clone()));
            } else if wk.is_mutable_ptr(&base_name) && type_args.len() == 1 {
                return Ok(AstType::mut_ptr(type_args[0].clone()));
            } else if wk.is_raw_ptr(&base_name) && type_args.len() == 1 {
                return Ok(AstType::raw_ptr(type_args[0].clone()));
            }

            return Ok(AstType::Generic {
                name: base_name,
                type_args,
            });
        }

        Ok(AstType::Generic {
            name: enum_type_name,
            type_args: vec![],
        })
    }
}

pub fn infer_closure_type(
    checker: &mut TypeChecker,
    params: &[(String, Option<AstType>)],
    return_type: &Option<AstType>,
    body: &Expression,
) -> Result<AstType> {
    let param_types: Vec<AstType> = params
        .iter()
        .map(|(_, opt_type)| opt_type.clone().unwrap_or(AstType::I32))
        .collect();

    if let Some(rt) = return_type {
        return Ok(AstType::FunctionPointer {
            param_types,
            return_type: Box::new(rt.clone()),
        });
    }

    checker.enter_scope();
    for (param_name, opt_type) in params {
        let param_type = opt_type.clone().unwrap_or(AstType::I32);
        let _ = checker.declare_variable(param_name, param_type, false);
    }

    let inferred_return = match body {
        Expression::Block(stmts) => {
            for stmt in stmts {
                match stmt {
                    crate::ast::Statement::VariableDeclaration { .. }
                    | crate::ast::Statement::VariableAssignment { .. } => {
                        let _ = checker.check_statement(stmt);
                    }
                    _ => {}
                }
            }

            let mut ret_type = Box::new(AstType::Void);
            for stmt in stmts {
                if let crate::ast::Statement::Return { expr: ret_expr, .. } = stmt {
                    if let Ok(rt) = checker.infer_expression_type(ret_expr) {
                        ret_type = Box::new(rt);
                        break;
                    }
                }
            }

            if matches!(*ret_type, AstType::Void) {
                if let Some(crate::ast::Statement::Expression {
                    expr: last_expr, ..
                }) = stmts.last()
                {
                    if let Ok(rt) = checker.infer_expression_type(last_expr) {
                        ret_type = Box::new(rt);
                    }
                }
            }

            ret_type
        }
        _ => {
            if let Ok(rt) = checker.infer_expression_type(body) {
                Box::new(rt)
            } else {
                Box::new(AstType::I32)
            }
        }
    };

    checker.exit_scope();

    Ok(AstType::FunctionPointer {
        param_types,
        return_type: inferred_return,
    })
}
