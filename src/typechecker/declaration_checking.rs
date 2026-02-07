//! Declaration type checking - collecting and checking declarations

use crate::ast::{AstType, Declaration, Expression};
use crate::error::{CompileError, Result};
use crate::typechecker::behaviors::MethodInfo;
use crate::typechecker::validation;
use crate::typechecker::{FunctionSignature, StructInfo, TypeChecker};

/// Collect all type definitions and function signatures (first pass)
pub fn collect_declaration_types(
    checker: &mut TypeChecker,
    declaration: &Declaration,
) -> Result<()> {
    match declaration {
        Declaration::Function(func) => {
            // Store the function signature with the declared return type for now
            // We'll infer the actual return type in a later pass if needed
            let signature = FunctionSignature {
                params: func.args.clone(),
                return_type: func.return_type.clone(),
                is_external: false,
            };
            checker
                .functions
                .insert(func.name.clone(), signature.clone());
            checker
                .type_store
                .borrow_mut()
                .register_function(&func.name, signature);
        }
        Declaration::ExternalFunction(ext_func) => {
            // External functions have args as Vec<AstType>, convert to params format
            let params = ext_func
                .args
                .iter()
                .enumerate()
                .map(|(i, t)| (format!("arg{}", i), t.clone()))
                .collect();
            let signature = FunctionSignature {
                params,
                return_type: ext_func.return_type.clone(),
                is_external: true,
            };
            checker
                .functions
                .insert(ext_func.name.clone(), signature.clone());
            checker
                .type_store
                .borrow_mut()
                .register_function(&ext_func.name, signature);
        }
        Declaration::Struct(struct_def) => {
            // Convert StructField to (String, AstType)
            // Store field types as-is for now (may contain Generic types for forward references)
            // We'll resolve Generic to Struct in a second pass after all structs are registered
            let fields: Vec<(String, AstType)> = struct_def
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.type_.clone()))
                .collect();
            let info = StructInfo {
                fields: fields.clone(),
            };
            checker
                .structs
                .insert(struct_def.name.clone(), info.clone());
            checker
                .type_store
                .borrow_mut()
                .register_struct(&struct_def.name, info);
        }
        Declaration::Enum(enum_def) => {
            // Convert EnumVariant to (String, Option<AstType>)
            let variants = enum_def
                .variants
                .iter()
                .map(|v| (v.name.clone(), v.payload.clone()))
                .collect();
            use crate::typechecker::EnumInfo;
            let info = EnumInfo { variants };
            checker.enums.insert(enum_def.name.clone(), info.clone());
            checker
                .type_store
                .borrow_mut()
                .register_enum(&enum_def.name, info);
        }
        Declaration::Behavior(behavior_def) => {
            checker.behavior_resolver.register_behavior(behavior_def)?;
        }
        Declaration::Trait(trait_def) => {
            checker.behavior_resolver.register_trait(trait_def)?;
        }
        Declaration::TraitImplementation(trait_impl) => {
            checker
                .behavior_resolver
                .register_trait_implementation(trait_impl)?;
        }
        Declaration::ImplBlock(impl_block) => {
            // Register inherent methods for the type
            // Store methods in the behavior resolver's inherent_methods
            let mut methods = Vec::new();
            for method in &impl_block.methods {
                let param_types: Vec<AstType> = method
                    .args
                    .iter()
                    .map(|(_, t)| {
                        if let AstType::Generic { name, .. } = t {
                            if name == "Self" {
                                // Replace Self with the actual type
                                AstType::Generic {
                                    name: impl_block.type_name.clone(),
                                    type_args: vec![],
                                }
                            } else {
                                t.clone()
                            }
                        } else {
                            t.clone()
                        }
                    })
                    .collect();
                let return_type = if let AstType::Generic { name, .. } = &method.return_type {
                    if name == "Self" {
                        AstType::Generic {
                            name: impl_block.type_name.clone(),
                            type_args: vec![],
                        }
                    } else {
                        method.return_type.clone()
                    }
                } else {
                    method.return_type.clone()
                };

                methods.push(MethodInfo {
                    name: method.name.clone(),
                    param_types,
                    return_type,
                });
            }
            checker
                .behavior_resolver
                .inherent_methods
                .insert(impl_block.type_name.clone(), methods);
        }
        Declaration::TraitRequirement(trait_req) => {
            checker
                .behavior_resolver
                .register_trait_requirement(trait_req)?;
        }
        Declaration::Constant {
            name, value, type_, ..
        } => {
            // Check if this is a struct definition pattern: Name = { field: Type, ... }
            if let Expression::StructLiteral { name: _, fields } = value {
                // This is a struct definition in the form: Point = { x: f64, y: f64 }
                // Convert the struct literal fields to struct type fields
                let mut struct_fields = Vec::new();
                for (field_name, field_value) in fields {
                    // Try to infer the type from the field value expression
                    // In struct definitions, field values should be type annotations
                    // For now, treat them as type identifiers or infer from expression
                    let field_type = checker.infer_expression_type(field_value)?;
                    struct_fields.push((field_name.clone(), field_type));
                }

                // Register this as a struct type
                let info = StructInfo {
                    fields: struct_fields.clone(),
                };
                checker.structs.insert(name.clone(), info.clone());
                checker.type_store.borrow_mut().register_struct(name, info);

                // Also store as a constant of struct type
                let struct_type = AstType::Struct {
                    name: name.clone(),
                    fields: Vec::new(), // Empty for the type reference
                };
                checker.declare_variable(name, struct_type, false)?;
            } else {
                // Regular constant declaration
                // Type check the constant value
                let inferred_type = checker.infer_expression_type(value)?;

                // If a type was specified, verify it matches
                if let Some(declared_type) = type_ {
                    if !checker.types_compatible(declared_type, &inferred_type) {
                        return Err(CompileError::TypeError(
                                format!(
                                    "Type mismatch: constant '{}' declared as {:?} but has value of type {:?}",
                                    name, declared_type, inferred_type
                                ),
                                None
                            ));
                    }
                }

                // Store the constant as a global variable (constants are immutable)
                checker.declare_variable(name, inferred_type, false)?;
            }
        }
        Declaration::ModuleImport {
            alias, module_path, ..
        } => {
            // Track module imports
            checker
                .module_imports
                .insert(alias.clone(), module_path.clone());
            // Register stdlib functions if this is a known stdlib module
            // Handle "@std.math", "std.math", and "math" formats
            let module_name = module_path
                .strip_prefix("@std.")
                .or_else(|| module_path.strip_prefix("std."))
                .unwrap_or(module_path.as_str());
            checker.register_stdlib_module(alias, module_name)?;
        }
        Declaration::TypeAlias(type_alias) => {
            // Check if the target type is a struct literal
            if let AstType::Struct { name: _, fields } = &type_alias.target_type {
                let info = StructInfo {
                    fields: fields.clone(),
                };
                checker.structs.insert(type_alias.name.clone(), info);
            } else if matches!(
                &type_alias.target_type,
                AstType::Function { .. } | AstType::FunctionPointer { .. }
            ) {
                // Store function type aliases for callable resolution
                // This includes both Function and FunctionPointer types
                checker
                    .type_aliases
                    .insert(type_alias.name.clone(), type_alias.target_type.clone());
                checker
                    .type_store
                    .borrow_mut()
                    .register_type_alias(&type_alias.name, type_alias.target_type.clone());
            }
        }
        _ => {}
    }
    Ok(())
}

/// Check declarations (second pass - after type collection)
pub fn check_declaration(checker: &mut TypeChecker, declaration: &Declaration) -> Result<()> {
    match declaration {
        Declaration::Function(func) => {
            super::function_checking::check_function(checker, func)?;
        }
        Declaration::ComptimeBlock(statements) => {
            // Check for imports in comptime blocks
            for stmt in statements {
                if let Err(msg) = validation::validate_import_not_in_comptime(stmt) {
                    return Err(CompileError::SyntaxError(msg, checker.get_current_span()));
                }
            }

            checker.enter_scope();
            for statement in statements {
                super::statement_checking::check_statement(checker, statement)?;
            }
            checker.exit_scope();
        }
        Declaration::Trait(_) => {
            // Trait definitions don't need additional checking beyond registration
        }
        Declaration::TraitImplementation(trait_impl) => {
            // Verify that the implementation satisfies the trait
            checker
                .behavior_resolver
                .verify_trait_implementation(trait_impl)?;
            // Type check each method in the implementation
            // We need to set context for resolving Self types
            checker.current_impl_type = Some(trait_impl.type_name.clone());
            for method in &trait_impl.methods {
                super::function_checking::check_function(checker, method)?;
            }
            checker.current_impl_type = None;
        }
        Declaration::TraitRequirement(trait_req) => {
            // Verify that the requirement is valid
            checker
                .behavior_resolver
                .verify_trait_requirement(trait_req)?;
        }
        Declaration::Constant { .. } => {
            // Constants are already type-checked in collect_declaration_types
        }
        Declaration::ModuleImport {
            alias,
            module_path: _,
            ..
        } => {
            // Handle module imports like { io, math } = @std which become
            // ModuleImport declarations at the top level
            // Register the imported module as a variable with StdModule type
            checker.declare_variable_with_init(alias, AstType::StdModule, false, true)?;
        }
        Declaration::TypeAlias(_) => {
            // TypeAlias already handled in collect_declaration_types
        }
        _ => {}
    }
    Ok(())
}
