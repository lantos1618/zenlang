use super::*;

impl TypeChecker {
    pub(super) fn resolve_generic_function_call(
        &mut self,
        function_name: &str,
        info: &FuncInfo,
        type_args: &[AstType],
        typed_args: &[TypedExpression],
        span: Span,
    ) -> (String, Type) {
        let (subs, type_args_valid) = if type_args.is_empty() {
            let arg_types: Vec<Type> = typed_args.iter().map(|arg| arg.ty.clone()).collect();
            let (subs, conflicts) =
                self.infer_type_args_with_conflicts(&info.type_params, &info.params, &arg_types);
            let inferred_type_args_valid =
                self.report_inference_conflicts("function", function_name, conflicts, span);
            (subs, inferred_type_args_valid)
        } else {
            self.explicit_type_arg_substitutions(
                "function",
                function_name,
                &info.type_params,
                type_args,
                span,
            )
        };

        let fallback_mangled =
            self.generic_function_mangled_name(function_name, &info.type_params, &subs);
        if !type_args_valid {
            return (fallback_mangled, Type::Unknown);
        }

        self.check_call_signature_with_substitutions(
            "function",
            function_name,
            &info.params,
            typed_args,
            &subs,
            &span,
        );

        if self.check_generic_bounds_valid(&info.type_param_bounds, &subs, span) {
            if let Some(mangled) = self.specialize_generic_function(function_name, &subs, span) {
                let ret_type = self.substitute_type(&info.return_type, &subs);
                (mangled, ret_type)
            } else {
                (fallback_mangled, Type::Unknown)
            }
        } else {
            (fallback_mangled, Type::Unknown)
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

    pub(crate) fn generic_type_annotation_arities_valid(&self, ast_type: &AstType) -> bool {
        match ast_type {
            AstType::Named(name) => self
                .structs
                .get(name)
                .map(|info| info.type_params.is_empty())
                .or_else(|| self.enums.get(name).map(|info| info.type_params.is_empty()))
                .unwrap_or(true),
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
}
