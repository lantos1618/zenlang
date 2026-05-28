use super::*;

impl TypeChecker {
    pub(super) fn resolve_callable_call(
        &mut self,
        kind: &str,
        callee: &str,
        info: &FuncInfo,
        type_args: &[AstType],
        typed_args: &[TypedExpression],
        span: Span,
    ) -> (String, Type) {
        if !info.type_params.is_empty() {
            return self
                .resolve_generic_callable_call(kind, callee, info, type_args, typed_args, span);
        }

        self.reject_nongeneric_type_args(kind, callee, type_args, span);
        self.check_call_signature(kind, callee, &info.params, typed_args, &span);
        (callee.to_string(), self.resolve_type(&info.return_type))
    }

    fn resolve_generic_callable_call(
        &mut self,
        kind: &str,
        callee: &str,
        info: &FuncInfo,
        type_args: &[AstType],
        typed_args: &[TypedExpression],
        span: Span,
    ) -> (String, Type) {
        let (subs, type_args_valid) = if type_args.is_empty() {
            let arg_types: Vec<Type> = typed_args.iter().map(|arg| arg.ty.clone()).collect();
            let (subs, conflicts) = if kind == "method" {
                self.infer_method_type_args(callee, &info.type_params, &info.params, &arg_types)
            } else {
                self.infer_type_args_with_conflicts(&info.type_params, &info.params, &arg_types)
            };
            (
                subs,
                self.report_inference_conflicts(kind, callee, conflicts, span),
            )
        } else {
            self.explicit_type_arg_substitutions(kind, callee, &info.type_params, type_args, span)
        };

        let fallback_mangled = self.generic_function_mangled_name(callee, &info.type_params, &subs);
        if !type_args_valid {
            return (fallback_mangled, Type::Unknown);
        }

        let saved_self_type = if kind == "method" {
            let self_type = self.generic_method_self_type(callee, &subs);
            Some(std::mem::replace(&mut self.current_self_type, self_type))
        } else {
            None
        };

        self.check_call_signature_with_substitutions(
            kind,
            callee,
            &info.params,
            typed_args,
            &subs,
            &span,
        );

        let diagnostic_count = self.diagnostics.len();
        self.check_generic_bounds(&info.type_param_bounds, &subs, span);
        let result = if self.diagnostics.len() == diagnostic_count {
            let specialized = if kind == "method" {
                self.specialize_generic_method(callee, &subs, span)
            } else {
                self.specialize_generic_function(callee, &subs, span)
            };
            if let Some(mangled) = specialized {
                let ret_type = self.substitute_type(&info.return_type, &subs);
                (mangled, ret_type)
            } else {
                (fallback_mangled, Type::Unknown)
            }
        } else {
            (fallback_mangled, Type::Unknown)
        };
        if let Some(saved_self_type) = saved_self_type {
            self.current_self_type = saved_self_type;
        }
        result
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
        let expected_types = params
            .iter()
            .map(|(_, expected)| self.substitute_type(expected, substitutions))
            .collect::<Vec<_>>();
        self.check_call_signature_types(kind, callee, &expected_types, args, span);
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
            let param = &conflict.param;
            let (inferred, actual) = type_display_pair(&conflict.inferred, &conflict.actual);
            self.push_error(
                E5000,
                format!("conflicting inferred type argument `{param}` for generic {kind} `{callee}`: inferred `{inferred}` and `{actual}`"),
                span,
            );
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
        let diagnostic_count = self.diagnostics.len();
        let substitutions =
            self.type_param_substitutions(type_params, type_args, kind, callee, span);
        let resolved_without_errors = self.diagnostics.len() == diagnostic_count;
        let annotations_valid = type_args
            .iter()
            .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg));
        (substitutions, annotations_valid && resolved_without_errors)
    }

    pub(crate) fn generic_type_annotation_arities_valid(&self, ast_type: &AstType) -> bool {
        !ast_type.any(&mut |ty| match ty {
            AstType::Named(name) => self
                .type_params_for_type(name)
                .is_some_and(|params| !params.is_empty()),
            AstType::Generic { name, type_args } => self
                .type_params_for_type(name)
                .is_some_and(|params| params.len() != type_args.len()),
            _ => false,
        })
    }
}
