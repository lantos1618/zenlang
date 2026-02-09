//! Closure type inference

use crate::ast::AstType;
use crate::ast::Expression;
use crate::error::Result;
use crate::typechecker::TypeChecker;

/// Infer the type of a closure/lambda expression
///
/// Untyped closure params default to i32 — this is intentional for the `.loop()` callback
/// pattern where the counter parameter is conventionally i32. A proper type inference system
/// for closures (inferring param types from the call-site expected type) is a future improvement.
pub fn infer_closure_type(
    checker: &mut TypeChecker,
    params: &[(String, Option<AstType>)],
    return_type: &Option<AstType>,
    body: &Expression,
) -> Result<AstType> {
    // DESIGN: Default untyped closure params to i32 (see fn doc comment above for rationale)
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

            Ok(ret_type)
        }
        _ => checker.infer_expression_type(body).map(Box::new),
    };

    checker.exit_scope();

    let inferred_return = inferred_return?;

    Ok(AstType::FunctionPointer {
        param_types,
        return_type: inferred_return,
    })
}
