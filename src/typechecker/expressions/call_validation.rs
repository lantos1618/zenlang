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

    pub(super) fn check_call_signature_with_substitutions(
        &mut self,
        kind: &str,
        callee: &str,
        params: &[(String, AstType)],
        args: &[TypedExpression],
        substitutions: &std::collections::HashMap<String, Type>,
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
            let expected = self.substitute_type(expected, substitutions);
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

    pub(super) fn report_inference_conflicts(
        &mut self,
        kind: &str,
        callee: &str,
        conflicts: Vec<InferenceConflict>,
        span: Span,
    ) -> bool {
        let valid = conflicts.is_empty();
        for conflict in conflicts {
            self.diagnostics.push(Diagnostic::error(
                "E5000",
                format!(
                    "conflicting inferred type argument `{}` for generic {} `{}`: inferred `{}` and `{}`",
                    conflict.param,
                    kind,
                    callee,
                    conflict.inferred.display_name(),
                    conflict.actual.display_name()
                ),
                span,
            ));
        }
        valid
    }

    pub(super) fn explicit_type_arg_substitutions(
        &mut self,
        kind: &str,
        callee: &str,
        type_params: &[String],
        type_args: &[AstType],
        span: Span,
    ) -> (std::collections::HashMap<String, Type>, bool) {
        let arity_valid = Self::explicit_type_args_valid(type_args, type_params);
        let diagnostic_count = self.diagnostics.len();
        let substitutions =
            self.type_param_substitutions(type_params, type_args, kind, callee, span);
        let resolved_without_errors = self.diagnostics.len() == diagnostic_count;
        let annotations_valid = type_args
            .iter()
            .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg));
        (
            substitutions,
            arity_valid && annotations_valid && resolved_without_errors,
        )
    }

    pub(super) fn check_generic_bounds_valid(
        &mut self,
        bounds: &std::collections::HashMap<String, BehaviorBound>,
        substitutions: &std::collections::HashMap<String, Type>,
        span: Span,
    ) -> bool {
        let diagnostic_count = self.diagnostics.len();
        self.check_generic_bounds(bounds, substitutions, span);
        self.diagnostics.len() == diagnostic_count
    }

    pub(super) fn explicit_type_args_valid(type_args: &[AstType], type_params: &[String]) -> bool {
        type_args.is_empty() || type_args.len() == type_params.len()
    }

    pub(super) fn generic_type_annotation_arities_valid(&self, ast_type: &AstType) -> bool {
        match ast_type {
            AstType::Generic { name, type_args } => {
                let own_arity_valid = self
                    .structs
                    .get(name)
                    .map(|info| info.type_params.len())
                    .or_else(|| self.enums.get(name).map(|info| info.type_params.len()))
                    .is_none_or(|expected| expected == type_args.len());
                own_arity_valid
                    && type_args
                        .iter()
                        .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg))
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => self.generic_type_annotation_arities_valid(inner),
            AstType::Array { elem, .. } => self.generic_type_annotation_arities_valid(elem),
            AstType::Function { params, ret } => {
                params
                    .iter()
                    .all(|param| self.generic_type_annotation_arities_valid(param))
                    && self.generic_type_annotation_arities_valid(ret)
            }
            _ => true,
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
            .find(|info| concrete_name.starts_with(&format!("{}_", info.name)))
            .map(|info| info.name.clone())
            .or_else(|| {
                self.enums
                    .values()
                    .filter(|info| !info.type_params.is_empty())
                    .find(|info| concrete_name.starts_with(&format!("{}_", info.name)))
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
            && matches!((module, function), ("io", "print") | ("io", "println"))
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
            TypedExprKind::Return(_) => true,
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
