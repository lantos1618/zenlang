use super::*;

pub(super) enum BehaviorMethodResolution {
    None,
    Resolved(String),
    Ambiguous,
}

impl TypeChecker {
    pub(super) fn check_call_signature(
        &mut self,
        kind: &str,
        callee: &str,
        params: &[(String, AstType)],
        args: &[TypedExpression],
        span: &Span,
    ) {
        if params.len() != args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E3021",
                format!(
                    "{} `{}` expects {} arguments, found {}",
                    kind,
                    callee,
                    params.len(),
                    args.len()
                ),
                *span,
            ));
            return;
        }

        for (idx, ((_, expected), actual)) in params.iter().zip(args.iter()).enumerate() {
            let expected = self.resolve_type(expected);
            if expected == Type::Unknown || actual.ty == Type::Unknown {
                continue;
            }

            if !self.types_compatible(&expected, &actual.ty) {
                self.diagnostics.push(Diagnostic::error(
                    "E3022",
                    format!(
                        "argument {} for `{}` expects `{}`, found `{}`",
                        idx + 1,
                        callee,
                        expected.display_name(),
                        actual.ty.display_name()
                    ),
                    actual.span,
                ));
            }
        }
    }

    pub(super) fn field_access_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Struct { name, .. } => Some(name.clone()),
            Type::Named(name) if self.structs.contains_key(name) => Some(name.clone()),
            Type::Ptr(inner) | Type::MutPtr(inner) => self.field_access_type_name(inner),
            _ => None,
        }
    }

    pub(super) fn generic_base_type_name(&self, concrete_name: &str) -> Option<String> {
        self.structs
            .values()
            .filter(|info| !info.type_params.is_empty())
            .find(|info| concrete_name_matches_generic(concrete_name, &info.name))
            .map(|info| info.name.clone())
            .or_else(|| {
                self.enums
                    .values()
                    .filter(|info| !info.type_params.is_empty())
                    .find(|info| concrete_name_matches_generic(concrete_name, &info.name))
                    .map(|info| info.name.clone())
            })
    }

    pub(super) fn unknown_method_expr(
        &mut self,
        type_name: &str,
        method: &str,
        typed_args: Vec<TypedExpression>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        if !type_name.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E3043",
                format!("type `{}` has no method `{}`", type_name, method),
                span,
            ));
        }
        Ok(TypedExpression {
            kind: TypedExprKind::FunctionCall {
                function: format!("{}_{}", type_name, method),
                args: typed_args,
            },
            ty: Type::Unknown,
            span,
        })
    }

    pub(super) fn behavior_specialized_method_key(
        &mut self,
        type_name: &str,
        method: &str,
        span: Span,
    ) -> BehaviorMethodResolution {
        let prefix = format!("{}__", Self::method_key(type_name, method));
        let candidates: Vec<String> = self
            .methods
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect();

        if candidates.is_empty() {
            return BehaviorMethodResolution::None;
        }

        if candidates.len() == 1 {
            return BehaviorMethodResolution::Resolved(candidates[0].clone());
        }

        if let Some(expected) = self.current_return_type.as_ref() {
            let matching: Vec<String> = candidates
                .iter()
                .filter(|key| {
                    self.methods.get(*key).is_some_and(|info| {
                        self.types_compatible(expected, &self.resolve_type(&info.return_type))
                    })
                })
                .cloned()
                .collect();

            if matching.len() == 1 {
                return BehaviorMethodResolution::Resolved(matching[0].clone());
            }
        }

        self.diagnostics.push(Diagnostic::error(
            "E3044",
            format!(
                "ambiguous behavior method `{}` for type `{}`; candidates: {}",
                method,
                type_name,
                candidates
                    .iter()
                    .map(|candidate| format!("`{candidate}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            span,
        ));
        BehaviorMethodResolution::Ambiguous
    }

    pub(super) fn ambiguous_method_expr(
        &self,
        type_name: &str,
        method: &str,
        typed_args: Vec<TypedExpression>,
        span: Span,
    ) -> TypedExpression {
        TypedExpression {
            kind: TypedExprKind::FunctionCall {
                function: format!("{}_{}", type_name, method),
                args: typed_args,
            },
            ty: Type::Unknown,
            span,
        }
    }

    pub(super) fn is_root_std_runtime_call(&self, module: &str, function: &str) -> bool {
        self.is_root_std_import(module)
            && crate::typechecker::std_runtime_calls::parse_std_runtime_call(module, function)
                .is_some()
    }
}
