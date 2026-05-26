use super::*;
use std::collections::HashMap;

fn literal_defer_stmt() -> ast::Statement {
    ast::Statement::Expression {
        expr: ast::Expression::Defer {
            expr: Box::new(ast::Expression::IntLiteral {
                value: 1,
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        },
        span: Span::dummy(),
    }
}

fn gated_range_expr() -> ast::Expression {
    ast::Expression::Range {
        start: Box::new(ast::Expression::IntLiteral {
            value: 1,
            span: Span::dummy(),
        }),
        end: Box::new(ast::Expression::IntLiteral {
            value: 2,
            span: Span::dummy(),
        }),
        inclusive: false,
        span: Span::dummy(),
    }
}

fn saved_defer_expr() -> TypedExpression {
    TypedExpression {
        kind: TypedExprKind::IntLiteral(99),
        ty: Type::I32,
        span: Span::dummy(),
    }
}

#[test]
fn function_context_is_restored_after_body_error() {
    let mut tc = TypeChecker::new();
    tc.current_return_type = Some(Type::Bool);
    tc.pending_defers.push(saved_defer_expr());
    let scope_depth = tc.scopes.len();

    let body = ast::Expression::Block {
        statements: vec![literal_defer_stmt()],
        expr: Some(Box::new(gated_range_expr())),
        span: Span::dummy(),
    };

    tc.check_function("broken", &[], &Some(AstType::Void), &body, &Span::dummy())
        .expect_err("gated range should fail function checking");

    assert_eq!(tc.scopes.len(), scope_depth);
    assert_eq!(tc.current_return_type, Some(Type::Bool));
    assert_eq!(tc.pending_defers, vec![saved_defer_expr()]);
}

#[test]
fn function_defers_do_not_leak_after_return_validation_error() {
    let mut tc = TypeChecker::new();
    tc.pending_defers.push(saved_defer_expr());

    let body = ast::Expression::Block {
        statements: vec![literal_defer_stmt()],
        expr: Some(Box::new(ast::Expression::BoolLiteral {
            value: true,
            span: Span::dummy(),
        })),
        span: Span::dummy(),
    };

    tc.check_function("broken", &[], &Some(AstType::I32), &body, &Span::dummy())
        .expect_err("return mismatch should fail function checking");

    assert_eq!(tc.current_return_type, None);
    assert_eq!(tc.pending_defers, vec![saved_defer_expr()]);
}

#[test]
fn specialized_type_recovery_prefers_remembered_source() {
    let mut tc = TypeChecker::new();
    tc.structs.insert(
        "Maybe".into(),
        StructInfo {
            specialization_scope: None,
            name: "Maybe".into(),
            fields: Vec::new(),
            field_defaults: HashMap::new(),
            type_params: vec!["T".into()],
            type_param_bounds: HashMap::new(),
        },
    );
    tc.remember_specialized_type_source("Maybe_i32", "CanonicalMaybe", &[AstType::I32]);

    let ty = Type::Struct {
        name: "Maybe_i32".into(),
        fields: Vec::new(),
    };

    assert_eq!(
        tc.generic_type_args_from_type("Maybe_i32", &ty),
        Some(("CanonicalMaybe".into(), vec![AstType::I32]))
    );
}
