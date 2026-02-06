use crate::ast::{AstType, Expression, Statement};
use crate::error::{CompileError, Result};
use crate::typechecker::{inference, TypeChecker};

/// Core expression type inference - delegates to the main infer_expression_type
/// This module contains the large match statement for inferring expression types
impl TypeChecker {
    pub fn infer_expression_type(&mut self, expr: &Expression) -> Result<AstType> {
        match expr {
            Expression::Integer32(_) => Ok(AstType::I32),
            Expression::Integer64(_) => Ok(AstType::I64),
            Expression::Float32(_) => Ok(AstType::F32),
            Expression::Float64(_) => Ok(AstType::F64),
            Expression::Boolean(_) => Ok(AstType::Bool),
            Expression::Unit => Ok(AstType::Void),
            Expression::String(_) => Ok(AstType::StaticString), // String literals are static strings
            Expression::Identifier(name) => inference::infer_identifier_type(self, name),
            Expression::BinaryOp { left, op, right } => {
                inference::infer_binary_op_type(self, left, op, right)
            }
            Expression::FunctionCall {
                name,
                type_args,
                args,
            } => inference::infer_function_call_type(self, name, type_args, args),
            Expression::MemberAccess { object, member } => {
                // Check if accessing @std namespace
                if let Expression::Identifier(name) = &**object {
                    if name == "@std" {
                        // Resolve @std.module access
                        return Ok(AstType::Generic {
                            name: format!("StdModule::{}", member),
                            type_args: vec![],
                        });
                    }
                }
                let object_type = self.infer_expression_type(object)?;
                inference::infer_member_type(
                    &object_type,
                    member,
                    &self.structs,
                    &self.enums,
                    self.get_current_span(),
                )
            }
            Expression::Comptime(inner) => self.infer_expression_type(inner),
            Expression::Range { .. } => Ok(AstType::Range {
                start_type: Box::new(AstType::I32),
                end_type: Box::new(AstType::I32),
                inclusive: false,
            }),
            Expression::StructLiteral { name, .. } => {
                // For struct literals, return the struct type
                // If the name contains '<', it's a generic struct like "Vec<T>"
                // Parse it to extract base name and type args, return Generic type
                if name.contains('<') {
                    let (base_name, type_args) = TypeChecker::parse_generic_type_string(name);
                    Ok(AstType::Generic {
                        name: base_name,
                        type_args,
                    })
                } else if let Some(struct_def) = self.structs.get(name) {
                    Ok(AstType::Struct {
                        name: name.clone(),
                        fields: struct_def.fields.clone(),
                    })
                } else {
                    // Check if it's a stdlib struct
                    if let Some(struct_info) = self.get_stdlib_struct(name) {
                        Ok(AstType::Struct {
                            name: name.clone(),
                            fields: struct_info.fields.clone(),
                        })
                    } else {
                        // It might be a generic struct that will be monomorphized
                        // For now, return a struct type with empty fields
                        Ok(AstType::Struct {
                            name: name.clone(),
                            fields: vec![],
                        })
                    }
                }
            }
            Expression::StdReference => {
                // Return a type representing @std
                Ok(AstType::Generic {
                    name: "Std".to_string(),
                    type_args: vec![],
                })
            }
            Expression::BuiltinReference => {
                // Return a type representing @builtin (raw compiler intrinsics)
                Ok(AstType::Generic {
                    name: "Builtin".to_string(),
                    type_args: vec![],
                })
            }
            Expression::ThisReference => {
                // Return a type representing @this
                Ok(AstType::Generic {
                    name: "This".to_string(),
                    type_args: vec![],
                })
            }
            Expression::StringInterpolation { .. } => {
                // String interpolation returns dynamic String (requires allocator)
                Ok(crate::ast::resolve_string_struct_type())
            }
            Expression::Closure {
                params,
                return_type,
                body,
            } => inference::infer_closure_type(self, params, return_type, body),
            Expression::ArrayIndex { array, .. } => {
                // Array indexing returns the element type
                let array_type = self.infer_expression_type(array)?;
                if let Some(elem_type) = array_type.ptr_inner() {
                    return Ok(elem_type.clone());
                }
                match array_type {
                    AstType::Slice(elem_type) => Ok(*elem_type),
                    AstType::FixedArray { element_type, .. } => Ok(*element_type),
                    _ => Err(CompileError::TypeError(
                        format!("Cannot index type {}: indexing requires an array, slice, or pointer type", array_type),
                        None,
                    )),
                }
            }
            Expression::AddressOf(inner) => {
                let inner_type = self.infer_expression_type(inner)?;
                Ok(AstType::ptr(inner_type))
            }
            Expression::Dereference(inner) => {
                let inner_type = self.infer_expression_type(inner)?;
                if let Some(elem_type) = inner_type.ptr_inner() {
                    return Ok(elem_type.clone());
                }
                Err(CompileError::TypeError(
                    format!("Cannot dereference non-pointer type {}: dereference (*) requires a pointer type (Ptr<T>, MutPtr<T>, or RawPtr<T>)", inner_type),
                    None,
                ))
            }
            Expression::PointerOffset { pointer, .. } => {
                // Pointer offset returns the same pointer type
                self.infer_expression_type(pointer)
            }
            Expression::StructField { struct_, field } => {
                let struct_type = self.infer_expression_type(struct_)?;
                inference::infer_struct_field_type(
                    &struct_type,
                    field,
                    &self.structs,
                    &self.enums,
                    self.get_current_span(),
                )
            }
            Expression::Integer8(_) => Ok(AstType::I8),
            Expression::Integer16(_) => Ok(AstType::I16),
            Expression::Unsigned8(_) => Ok(AstType::U8),
            Expression::Unsigned16(_) => Ok(AstType::U16),
            Expression::Unsigned32(_) => Ok(AstType::U32),
            Expression::Unsigned64(_) => Ok(AstType::U64),
            Expression::ArrayLiteral(elements) => {
                // Infer type from first element - array literals produce slices
                if let Some(first_elem) = elements.first() {
                    let elem_type = self.infer_expression_type(first_elem)?;
                    Ok(AstType::Slice(Box::new(elem_type)))
                } else {
                    Ok(AstType::Slice(Box::new(AstType::Void)))
                }
            }
            Expression::QuestionMatch { scrutinee, arms } => {
                // QuestionMatch expression type is determined by the arms
                // All arms should have the same type

                // Infer the type of the scrutinee to properly type pattern bindings
                let scrutinee_type = self.infer_expression_type(scrutinee)?;

                if arms.is_empty() {
                    Ok(AstType::Void)
                } else {
                    let mut result_type = AstType::Void;

                    // Process each arm with its own pattern bindings
                    for (i, arm) in arms.iter().enumerate() {
                        // Enter a new scope for the pattern bindings
                        self.enter_scope();

                        // Extract pattern bindings and add them to the scope
                        // Pass the scrutinee type for proper typing
                        self.add_pattern_bindings_to_scope_with_type(
                            &arm.pattern,
                            &scrutinee_type,
                        )?;

                        // Special handling for blocks with early returns
                        // If the arm body is a block, we need to check if it actually
                        // produces a value or just has side effects before returning
                        let arm_type = if let Expression::Block(stmts) = &arm.body {
                            // Check if the block has any non-return statements before the return
                            let mut block_type = AstType::Void;
                            let has_early_return = false;

                            for (j, stmt) in stmts.iter().enumerate() {
                                match stmt {
                                    Statement::Return { .. } => {
                                        // Don't use return statement to determine block type
                                        break;
                                    }
                                    Statement::Expression { expr, .. } => {
                                        // If this is the last statement and there's no early return after it
                                        if j == stmts.len() - 1 && !has_early_return {
                                            block_type = self.infer_expression_type(expr)?;
                                        } else {
                                            // Still type-check intermediate expressions
                                            let _ = self.infer_expression_type(expr)?;
                                        }
                                    }
                                    _ => {
                                        self.check_statement(stmt)?;
                                    }
                                }
                            }
                            block_type
                        } else {
                            self.infer_expression_type(&arm.body)?
                        };

                        // The first non-void arm determines the type, or use first arm if all void
                        if i == 0
                            || (matches!(result_type, AstType::Void)
                                && !matches!(arm_type, AstType::Void))
                        {
                            result_type = arm_type;
                        }

                        // Exit the scope to remove the bindings
                        self.exit_scope();
                    }

                    Ok(result_type)
                }
            }
            Expression::PatternMatch { arms, .. } => {
                // Pattern match expression type is determined by the first arm
                // All arms should have the same type
                if arms.is_empty() {
                    Ok(AstType::Void)
                } else {
                    let mut result_type = AstType::Void;

                    // Process each arm with its own pattern bindings
                    for (i, arm) in arms.iter().enumerate() {
                        // Enter a new scope for the pattern bindings
                        self.enter_scope();

                        // Extract pattern bindings and add them to the scope
                        self.add_pattern_bindings_to_scope(&arm.pattern)?;

                        // Infer the type with bindings in scope
                        let arm_type = self.infer_expression_type(&arm.body)?;

                        // The first arm determines the type
                        if i == 0 {
                            result_type = arm_type;
                        }

                        // Exit the scope to remove the bindings
                        self.exit_scope();
                    }

                    Ok(result_type)
                }
            }
            Expression::Block(statements) => {
                // Enter a new scope for the block
                self.enter_scope();

                let mut block_type = AstType::Void;

                // Process all statements in the block
                for (i, stmt) in statements.iter().enumerate() {
                    match stmt {
                        Statement::Expression { expr, .. } => {
                            // The last expression determines the block's type
                            if i == statements.len() - 1 {
                                block_type = self.infer_expression_type(expr)?;
                            } else {
                                // Still type-check intermediate expressions
                                self.infer_expression_type(expr)?;
                            }
                        }
                        _ => {
                            // Process other statements (declarations, assignments, etc.)
                            self.check_statement(stmt)?;
                        }
                    }
                }

                // Exit the block's scope
                self.exit_scope();

                Ok(block_type)
            }
            Expression::Return(expr) => self.infer_expression_type(expr),
            Expression::EnumVariant {
                enum_name,
                variant,
                payload,
            } => inference::infer_enum_variant_type(self, enum_name, variant, payload),
            Expression::StringLength(_) => Ok(AstType::I64),
            Expression::MethodCall {
                object,
                method,
                type_args,
                args: _,
            } => inference::infer_method_call_type(self, object, method, type_args),
            Expression::Loop { body: _ } => {
                // Loop expressions return void for now
                Ok(AstType::Void)
            }
            Expression::Raise(expr) => inference::infer_raise_type(self, expr),
            Expression::Break { .. } | Expression::Continue { .. } => {
                // Break and continue don't return a value, they transfer control
                // For type checking purposes, they can be considered to return void
                Ok(AstType::Void)
            }
            Expression::EnumLiteral { variant, payload } => {
                inference::infer_enum_literal_type(self, variant, payload)
            }
            Expression::Conditional { scrutinee, arms } => {
                // Infer the type of the scrutinee to properly type pattern bindings
                let scrutinee_type = self.infer_expression_type(scrutinee)?;

                if arms.is_empty() {
                    Ok(AstType::Void)
                } else {
                    let mut result_type = AstType::Void;

                    // Process each arm with its own pattern bindings
                    for (i, arm) in arms.iter().enumerate() {
                        self.enter_scope();

                        // Extract pattern bindings and add them to the scope
                        self.add_pattern_bindings_to_scope_with_type(
                            &arm.pattern,
                            &scrutinee_type,
                        )?;

                        let arm_type = self.infer_expression_type(&arm.body)?;

                        // The first arm determines the type
                        if i == 0 {
                            result_type = arm_type;
                        }

                        self.exit_scope();
                    }

                    Ok(result_type)
                }
            }
            // Zen spec pointer operations
            Expression::PointerDereference(expr) => {
                // ptr.val -> T (if ptr is Ptr<T>, MutPtr<T>, or RawPtr<T>)
                let ptr_type = self.infer_expression_type(expr)?;
                if let Some(inner) = ptr_type.ptr_inner() {
                    Ok(inner.clone())
                } else {
                    Err(CompileError::TypeError(
                        format!("Cannot dereference non-pointer type {}: .val requires a pointer type (Ptr<T>, MutPtr<T>, or RawPtr<T>)", ptr_type),
                        None,
                    ))
                }
            }
            Expression::PointerAddress(expr) => {
                // expr.addr -> RawPtr<T> (if expr is of type T)
                let expr_type = self.infer_expression_type(expr)?;
                Ok(AstType::raw_ptr(expr_type))
            }
            Expression::CreateReference(expr) => {
                // expr.ref() -> Ptr<T> (if expr is of type T)
                let expr_type = self.infer_expression_type(expr)?;
                Ok(AstType::ptr(expr_type))
            }
            Expression::CreateMutableReference(expr) => {
                // expr.mut_ref() -> MutPtr<T> (if expr is of type T)
                let expr_type = self.infer_expression_type(expr)?;
                Ok(AstType::mut_ptr(expr_type))
            }
            Expression::VecConstructor {
                element_type,
                size: _,
                initial_values: _,
            } => {
                // Vec<T>() -> Generic { name: "Vec", type_args: [T] }
                Ok(AstType::Generic {
                    name: "Vec".to_string(),
                    type_args: vec![element_type.clone()],
                })
            }
            Expression::DynVecConstructor {
                element_types,
                allocator: _,
                initial_capacity: _,
            } => {
                // DynVec<T>() -> Generic { name: "DynVec", type_args: [T, ...] }
                Ok(AstType::Generic {
                    name: "DynVec".to_string(),
                    type_args: element_types.clone(),
                })
            }
            Expression::ArrayConstructor { element_type } => {
                // Array<T>() -> Generic { name: "Array", type_args: [T] }
                // This matches the expected type format for generic types
                Ok(AstType::Generic {
                    name: "Array".to_string(),
                    type_args: vec![element_type.clone()],
                })
            }
            Expression::Some(inner) => {
                let inner_type = self.infer_expression_type(inner)?;
                Ok(AstType::Generic {
                    name: self
                        .well_known
                        .get_variant_parent_name(self.well_known.some_name())
                        .ok_or_else(|| {
                            CompileError::InternalError(
                                "Some variant missing Option parent type".to_string(),
                                self.get_current_span(),
                            )
                        })?
                        .to_string(),
                    type_args: vec![inner_type],
                })
            }
            Expression::None => Ok(AstType::Generic {
                name: self
                    .well_known
                    .get_variant_parent_name(self.well_known.none_name())
                    .ok_or_else(|| {
                        CompileError::InternalError(
                            "None variant missing Option parent type".to_string(),
                            self.get_current_span(),
                        )
                    })?
                    .to_string(),
                type_args: vec![AstType::Void],
            }),
            Expression::CollectionLoop { .. } => {
                // collection.loop() returns unit/void
                Ok(AstType::Void)
            }
            Expression::Defer(_) => {
                // @this.defer() returns unit/void
                Ok(AstType::Void)
            }
        }
    }
}
