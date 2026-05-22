mod generic_templates;

use super::*;

pub(super) struct BehaviorImplValidationInput<'a> {
    pub(super) type_name: &'a str,
    pub(super) type_args: &'a [AstType],
    pub(super) behavior: &'a str,
    pub(super) behavior_type_args: &'a [AstType],
    pub(super) methods: &'a [Declaration],
    pub(super) span: Span,
    pub(super) symbols: Option<&'a SymbolTable>,
}

impl TypeChecker {
    pub(super) fn check_behavior_impl(&mut self, input: BehaviorImplValidationInput<'_>) {
        let BehaviorImplValidationInput {
            type_name,
            type_args,
            behavior,
            behavior_type_args,
            methods,
            span,
            symbols,
        } = input;
        let resolver_impl_ref = self.resolver_impl_ref_for(type_name, behavior, behavior_type_args);
        if self.should_skip_missing_resolver_behavior_ref(
            resolver_impl_ref.as_ref(),
            type_name,
            &self.resolver_missing_behavior_impl_refs,
        ) {
            return;
        }
        let (behavior, behavior_type_args) =
            Self::behavior_ref_parts(resolver_impl_ref.as_ref(), behavior, behavior_type_args);

        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.diagnostics.push(Diagnostic::error(
                "E6005",
                format!("undefined type `{}`", type_name),
                span,
            ));
            return;
        }

        if !type_args.is_empty() {
            self.check_generic_behavior_impl_template(
                type_name,
                type_args,
                behavior,
                behavior_type_args,
                methods,
                span,
            );
            return;
        }

        if self.reject_unspecialized_generic_type(type_name, span) {
            return;
        }

        let Some(behavior_substitutions) = self.behavior_type_arg_substitutions(
            behavior,
            behavior_type_args,
            &HashSet::new(),
            span,
        ) else {
            return;
        };
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        if self
            .behavior_impls
            .contains(&(type_name.to_string(), behavior_key.clone()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E6003",
                format!(
                    "duplicate implementation of behavior `{}` for type `{}`",
                    behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        if let Some(existing) = self.find_overlapping_behavior_impl(type_name, &behavior_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6010",
                format!(
                    "overlapping implementations of behaviors `{}` and `{}` for type `{}`",
                    existing, behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        self.behavior_impls
            .insert((type_name.to_string(), behavior_key.clone()));
        self.behavior_refs_by_key.insert(
            behavior_key.clone(),
            self.behavior_parent_ref(behavior, behavior_type_args),
        );
        let required_methods =
            self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new());
        let mut unmatched_required: VecDeque<String> = required_methods
            .iter()
            .map(|required| required.name.clone())
            .collect();
        let effective_methods = self.effective_behavior_impl_methods(
            symbols,
            type_name,
            methods,
            &mut unmatched_required,
        );

        for method in &effective_methods {
            if let Declaration::Function { span, .. } = method.declaration {
                if !required_methods
                    .iter()
                    .any(|required| required.name == method.method_name)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E6005",
                        format!(
                            "method `{}` is not declared by behavior `{}`",
                            method.method_name, behavior_key
                        ),
                        *span,
                    ));
                }
            }
        }

        for required in &required_methods {
            let Some(actual) =
                effective_methods
                    .iter()
                    .find_map(|method| match method.declaration {
                        Declaration::Function {
                            params,
                            return_type,
                            span,
                            ..
                        } if method.method_name == required.name => {
                            Some((params, return_type, *span))
                        }
                        _ => None,
                    })
            else {
                if required.default_body.is_some() {
                    continue;
                }
                self.diagnostics.push(Diagnostic::error(
                    "E6001",
                    format!(
                        "type `{}` implementation of `{}` is missing required method `{}`",
                        type_name, behavior_key, required.name
                    ),
                    span,
                ));
                continue;
            };

            let (actual_params, actual_return_type, actual_span) = actual;
            let collected_signature =
                self.resolver_backed_method_signature(type_name, &required.name);
            let actual_param_types: Vec<AstType> = collected_signature
                .map(|info| {
                    info.params
                        .iter()
                        .map(|(_, ty)| ty.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| actual_params.iter().map(|param| param.ty.clone()).collect());
            let actual_return = collected_signature
                .map(|info| info.return_type.clone())
                .unwrap_or_else(|| actual_return_type.clone().unwrap_or(AstType::Void));

            if actual_param_types.len() != required.params.len() {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects {} parameters, found {}",
                        required.name,
                        behavior_key,
                        required.params.len(),
                        actual_param_types.len()
                    ),
                    actual_span,
                ));
                continue;
            }

            for (idx, (expected, actual_ty)) in required
                .params
                .iter()
                .zip(actual_param_types.iter())
                .enumerate()
            {
                if !self.impl_ast_types_compatible(&expected.ty, actual_ty, type_name) {
                    self.diagnostics.push(Diagnostic::error(
                        "E6002",
                        format!(
                            "parameter {} for method `{}` in behavior `{}` expects `{}`, found `{}`",
                            idx + 1,
                            required.name,
                            behavior_key,
                            self.impl_type_display(&expected.ty, type_name),
                            actual_ty.display_name()
                        ),
                        actual_params
                            .get(idx)
                            .map(|param| param.span)
                            .unwrap_or(actual_span),
                    ));
                }
            }

            let expected_return = required.return_type.as_ref().unwrap_or(&AstType::Void);
            if !self.impl_ast_types_compatible(expected_return, &actual_return, type_name) {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects return `{}`, found `{}`",
                        required.name,
                        behavior_key,
                        self.impl_type_display(expected_return, type_name),
                        actual_return.display_name()
                    ),
                    actual_span,
                ));
            }
        }
    }
}
