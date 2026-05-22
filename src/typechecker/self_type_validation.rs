use super::*;

mod expressions;
mod statements;
mod tasks;

impl TypeChecker {
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
}
