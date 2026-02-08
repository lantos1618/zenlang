use crate::ast::{AstType, Function};
use crate::error::Result;
use crate::typechecker::TypeChecker;

pub fn check_function(checker: &mut TypeChecker, function: &Function) -> Result<()> {
    checker.enter_scope();

    let prev_function_name = checker.current_function_name.take();
    checker.current_function_name = Some(function.name.clone());
    checker.set_function_return_type(Some(function.return_type.clone()));

    let result = check_function_body(checker, function);

    checker.set_function_return_type(None);
    checker.current_function_name = prev_function_name;
    checker.exit_scope();

    result
}

fn check_function_body(checker: &mut TypeChecker, function: &Function) -> Result<()> {
    for (param_name, param_type) in &function.args {
        let actual_type = if param_name == "self" {
            match param_type {
                AstType::Generic { name, .. } if name == "Self" || name.starts_with("Self_") => {
                    if let Some(impl_type) = &checker.current_impl_type {
                        let struct_fields = checker
                            .type_store
                            .borrow()
                            .get_struct(impl_type)
                            .map(|s| s.fields.clone());
                        if let Some(fields) = struct_fields {
                            AstType::Struct {
                                name: impl_type.clone(),
                                fields,
                            }
                        } else {
                            AstType::Struct {
                                name: impl_type.clone(),
                                fields: vec![],
                            }
                        }
                    } else {
                        param_type.clone()
                    }
                }
                _ => param_type.clone(),
            }
        } else {
            param_type.clone()
        };

        checker.declare_variable(param_name, actual_type, false)?;
    }

    for statement in &function.body {
        super::statement_checking::check_statement(checker, statement)?;
    }

    Ok(())
}
