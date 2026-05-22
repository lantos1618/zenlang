use super::*;

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
            .find(|info| self.concrete_type_name_matches_generic(concrete_name, &info.name))
            .map(|info| info.name.clone())
            .or_else(|| {
                self.enums
                    .values()
                    .filter(|info| !info.type_params.is_empty())
                    .find(|info| self.concrete_type_name_matches_generic(concrete_name, &info.name))
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
        &self,
        type_name: &str,
        method: &str,
    ) -> Option<String> {
        let prefix = format!("{}__", Self::method_key(type_name, method));
        let candidates: Vec<_> = self
            .methods
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .collect();

        if candidates.len() == 1 {
            return Some(candidates[0].0.clone());
        }

        let expected = self.current_return_type.as_ref()?;
        let matching: Vec<_> = candidates
            .into_iter()
            .filter(|(_, info)| {
                self.types_compatible(expected, &self.resolve_type(&info.return_type))
            })
            .collect();

        (matching.len() == 1).then(|| matching[0].0.clone())
    }

    pub(super) fn is_root_std_runtime_call(&self, module: &str, function: &str) -> bool {
        self.is_root_std_import(module)
            && crate::typechecker::std_runtime_calls::parse_std_runtime_call(module, function)
                .is_some()
    }

    pub(super) fn block_satisfies_return(&self, block: &TypedBlock, ret_type: &Type) -> bool {
        if block.ty != Type::Void && self.types_compatible(ret_type, &block.ty) {
            return true;
        }

        self.block_definitely_returns(block)
    }

    pub(super) fn block_definitely_returns(&self, block: &TypedBlock) -> bool {
        block
            .expr
            .as_ref()
            .is_some_and(|expr| self.expr_definitely_returns(expr))
            || block.statements.iter().any(|stmt| match &stmt.kind {
                TypedStatementKind::Expression(expr) => self.expr_definitely_returns(expr),
                TypedStatementKind::VarDecl { .. } => false,
            })
    }

    pub(super) fn expr_definitely_returns(&self, expr: &TypedExpression) -> bool {
        match &expr.kind {
            TypedExprKind::Block(block) => self.block_definitely_returns(block),
            TypedExprKind::Match { arms, .. } => {
                !arms.is_empty()
                    && arms
                        .iter()
                        .all(|arm| self.block_definitely_returns(&arm.body))
            }
            _ => false,
        }
    }
}
