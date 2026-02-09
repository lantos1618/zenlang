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
                module,
                name,
                type_args,
                args,
                span,
            } => inference::infer_function_call_type(
                self,
                module.as_deref(),
                name,
                type_args,
                args,
                span.clone(),
            ),
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
                let type_store = self.type_store.borrow();
                inference::infer_member_type(
                    &object_type,
                    member,
                    type_store.get_all_structs(),
                    type_store.get_all_enums(),
                    self.get_current_span(),
                    0,
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
                } else if let Some(struct_def) = self.type_store.borrow().get_struct(name) {
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
                let type_store = self.type_store.borrow();
                inference::infer_struct_field_type(
                    &struct_type,
                    field,
                    type_store.get_all_structs(),
                    type_store.get_all_enums(),
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
                // Infer type from first element
                // Array literals with known count produce FixedArray, empty ones produce Slice
                if let Some(first_elem) = elements.first() {
                    let elem_type = self.infer_expression_type(first_elem)?;
                    Ok(AstType::FixedArray {
                        element_type: Box::new(elem_type),
                        size: elements.len(),
                    })
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

                    for (i, arm) in arms.iter().enumerate() {
                        self.enter_scope();

                        let arm_result =
                            self.check_match_arm_body(&arm.pattern, &arm.body, &scrutinee_type);

                        self.exit_scope();

                        let arm_type = arm_result?;

                        if i == 0
                            || (matches!(result_type, AstType::Void)
                                && !matches!(arm_type, AstType::Void))
                        {
                            result_type = arm_type;
                        }
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
                self.enter_scope();

                let mut block_type = AstType::Void;
                let mut err = None;

                for (i, stmt) in statements.iter().enumerate() {
                    let result = match stmt {
                        Statement::Expression { expr, .. } => {
                            if i == statements.len() - 1 {
                                self.infer_expression_type(expr).map(|t| {
                                    block_type = t;
                                })
                            } else {
                                self.infer_expression_type(expr).map(|_| ())
                            }
                        }
                        _ => self.check_statement(stmt),
                    };
                    if let Err(e) = result {
                        err = Some(e);
                        break;
                    }
                }

                self.exit_scope();

                match err {
                    Some(e) => Err(e),
                    None => Ok(block_type),
                }
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
                span,
            } => inference::infer_method_call_type(self, object, method, type_args, span.clone()),
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

                    for (i, arm) in arms.iter().enumerate() {
                        self.enter_scope();

                        let arm_result =
                            self.check_match_arm_body(&arm.pattern, &arm.body, &scrutinee_type);

                        self.exit_scope();

                        let arm_type = arm_result?;

                        if i == 0 {
                            result_type = arm_type;
                        }
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
            Expression::Defer(_) => Ok(AstType::Void),
        }
    }

    fn check_match_arm_body(
        &mut self,
        pattern: &crate::ast::Pattern,
        body: &Expression,
        scrutinee_type: &AstType,
    ) -> Result<AstType> {
        self.add_pattern_bindings_to_scope_with_type(pattern, scrutinee_type)?;

        if let Expression::Block(stmts) = body {
            let mut block_type = AstType::Void;
            let has_early_return = false;

            for (j, stmt) in stmts.iter().enumerate() {
                match stmt {
                    Statement::Return { .. } => {
                        break;
                    }
                    Statement::Expression { expr, .. } => {
                        if j == stmts.len() - 1 && !has_early_return {
                            block_type = self.infer_expression_type(expr)?;
                        } else {
                            let _ = self.infer_expression_type(expr)?;
                        }
                    }
                    _ => {
                        self.check_statement(stmt)?;
                    }
                }
            }
            Ok(block_type)
        } else {
            self.infer_expression_type(body)
        }
    }
}
