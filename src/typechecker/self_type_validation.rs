use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(super) fn collect_self_type_context_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<SelfTypeContextValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_self_type_context_validation_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_self_type_context_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<SelfTypeContextValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct { fields, .. } => {
                tasks.push(SelfTypeContextValidationTask::Struct { fields });
            }
            Declaration::Enum { variants, .. } => {
                tasks.push(SelfTypeContextValidationTask::Enum { variants });
            }
            Declaration::Function {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Function {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Method {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Method {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Behavior { methods, .. } => {
                tasks.push(SelfTypeContextValidationTask::Behavior { methods });
            }
            Declaration::ImplBlock {
                behavior_type_args,
                methods,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::ImplBlock {
                behavior_type_args,
                methods,
                span: *span,
            }),
            Declaration::Requires {
                behavior_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Requires {
                behavior_type_args,
                span: *span,
            }),
            Declaration::BehaviorExtends {
                parent_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::BehaviorExtends {
                parent_type_args,
                span: *span,
            }),
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(SelfTypeContextValidationTask::TopLevelExpr { expr });
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
        }
    }

    pub(super) fn validate_self_type_context_tasks(
        &mut self,
        tasks: &[SelfTypeContextValidationTask<'_>],
    ) {
        for task in tasks {
            match task {
                SelfTypeContextValidationTask::Struct { fields } => {
                    for field in *fields {
                        self.validate_self_type_ref(&field.ty, field.span, false);
                        if let Some(default) = &field.default {
                            self.validate_self_type_expr(default, false);
                        }
                    }
                }
                SelfTypeContextValidationTask::Enum { variants } => {
                    for variant in *variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_self_type_ref(payload, variant.span, false);
                        }
                    }
                }
                SelfTypeContextValidationTask::Function {
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    self.validate_self_type_callable(params, return_type, body, *span, false);
                }
                SelfTypeContextValidationTask::Method {
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    self.validate_self_type_callable(params, return_type, body, *span, true);
                }
                SelfTypeContextValidationTask::Behavior { methods } => {
                    for method in *methods {
                        let Some(default_body) = &method.default_body else {
                            self.validate_self_type_params(&method.params, true);
                            if let Some(return_type) = &method.return_type {
                                self.validate_self_type_ref(return_type, method.span, true);
                            }
                            continue;
                        };
                        self.validate_self_type_callable(
                            &method.params,
                            &method.return_type,
                            default_body,
                            method.span,
                            true,
                        );
                    }
                }
                SelfTypeContextValidationTask::ImplBlock {
                    behavior_type_args,
                    methods,
                    span,
                } => {
                    self.validate_self_type_refs(behavior_type_args, *span, false);
                    for method in *methods {
                        if let Declaration::Function {
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            self.validate_self_type_callable(
                                params,
                                return_type,
                                body,
                                *span,
                                true,
                            );
                        }
                    }
                }
                SelfTypeContextValidationTask::Requires {
                    behavior_type_args,
                    span,
                } => {
                    self.validate_self_type_refs(behavior_type_args, *span, false);
                }
                SelfTypeContextValidationTask::BehaviorExtends {
                    parent_type_args,
                    span,
                } => {
                    self.validate_self_type_refs(parent_type_args, *span, false);
                }
                SelfTypeContextValidationTask::TopLevelExpr { expr } => {
                    self.validate_self_type_expr(expr, false);
                }
            }
        }
    }

    fn validate_self_type_callable(
        &mut self,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
        allow_self_type: bool,
    ) {
        self.validate_self_type_params(params, allow_self_type);
        if let Some(return_type) = return_type {
            self.validate_self_type_ref(return_type, span, allow_self_type);
        }
        self.validate_self_type_expr(body, allow_self_type);
    }

    fn validate_self_type_params(&mut self, params: &[Param], allow_self_type: bool) {
        for param in params {
            self.validate_self_type_ref(&param.ty, param.span, allow_self_type);
        }
    }

    fn validate_self_type_refs(
        &mut self,
        ast_types: &[AstType],
        span: Span,
        allow_self_type: bool,
    ) {
        for ast_type in ast_types {
            self.validate_self_type_ref(ast_type, span, allow_self_type);
        }
    }

    fn validate_self_type_ref(&mut self, ast_type: &AstType, span: Span, allow_self_type: bool) {
        match ast_type {
            AstType::SelfType => {
                if !allow_self_type {
                    self.diagnostics.push(Diagnostic::error(
                        "E0204",
                        "Self type is only valid in method or behavior contexts",
                        span,
                    ));
                }
            }
            AstType::Generic { type_args, .. } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, span, allow_self_type);
                }
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.validate_self_type_ref(inner, span, allow_self_type);
            }
            AstType::Array { elem, .. } => {
                self.validate_self_type_ref(elem, span, allow_self_type);
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_self_type_ref(param, span, allow_self_type);
                }
                self.validate_self_type_ref(ret, span, allow_self_type);
            }
            AstType::I8
            | AstType::I16
            | AstType::I32
            | AstType::I64
            | AstType::U8
            | AstType::U16
            | AstType::U32
            | AstType::U64
            | AstType::Usize
            | AstType::F32
            | AstType::F64
            | AstType::Bool
            | AstType::Void
            | AstType::Str
            | AstType::String
            | AstType::Named(_)
            | AstType::Inferred => {}
        }
    }

    fn validate_self_type_expr(&mut self, expr: &Expression, allow_self_type: bool) {
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
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_self_type_expr(value, allow_self_type);
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
            | Expression::CharLiteral { .. }
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
