use super::*;

impl TypeChecker {
    pub(super) fn validate_self_type_expr(&mut self, expr: &Expression, allow_self_type: bool) {
        match expr {
            Expression::FunctionCall {
                type_args,
                args,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for arg in args {
                    self.validate_self_type_expr(arg, allow_self_type);
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_self_type_expr(receiver, allow_self_type);
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for arg in args {
                    self.validate_self_type_expr(arg, allow_self_type);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_self_type_expr(left, allow_self_type);
                self.validate_self_type_expr(right, allow_self_type);
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_self_type_expr(operand, allow_self_type);
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_self_type_expr(object, allow_self_type);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_self_type_expr(object, allow_self_type);
                self.validate_self_type_expr(index, allow_self_type);
            }
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for (_, value) in fields {
                    self.validate_self_type_expr(value, allow_self_type);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload: None,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload: Some(payload),
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                self.validate_self_type_expr(payload, allow_self_type);
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_self_type_expr(element, allow_self_type);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_self_type_expr(scrutinee, allow_self_type);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.validate_self_type_expr(guard, allow_self_type);
                    }
                    self.validate_self_type_expr(&arm.body, allow_self_type);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.validate_self_type_expr(condition, allow_self_type);
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::Loop { body, .. } => {
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_self_type_expr(condition, allow_self_type);
                self.validate_self_type_expr(then_body, allow_self_type);
                if let Some(else_body) = else_body {
                    self.validate_self_type_expr(else_body, allow_self_type);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.validate_self_type_statement(statement, allow_self_type);
                }
                if let Some(expr) = expr {
                    self.validate_self_type_expr(expr, allow_self_type);
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                self.validate_self_type_params(params, allow_self_type);
                if let Some(return_type) = return_type {
                    self.validate_self_type_ref(return_type, *span, allow_self_type);
                }
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_self_type_expr(expr, allow_self_type);
                self.validate_self_type_ref(target_type, *span, allow_self_type);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.validate_self_type_expr(expr, allow_self_type);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_self_type_expr(start, allow_self_type);
                self.validate_self_type_expr(end, allow_self_type);
            }
            Expression::Defer { expr, .. } => {
                self.validate_self_type_expr(expr, allow_self_type);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::LoopControl { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_self_type_statement(&mut self, statement: &ast::Statement, allow_self_type: bool) {
        match statement {
            ast::Statement::VarDecl {
                ty, value, span, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_self_type_ref(ty, *span, allow_self_type);
                }
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.validate_self_type_expr(target, allow_self_type);
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Expression { expr, .. } => {
                self.validate_self_type_expr(expr, allow_self_type);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.validate_self_type_statement(statement, allow_self_type);
                }
            }
        }
    }
}
