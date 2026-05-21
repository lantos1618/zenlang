use super::*;

struct BehaviorImplActualMethodSignature<'a> {
    params: &'a [Param],
    param_types: Vec<AstType>,
    return_type: AstType,
    span: Span,
}

impl TypeChecker {
    pub(super) fn validate_behavior_impl_methods<'a>(
        &mut self,
        type_name: &str,
        behavior_key: &str,
        required_methods: &[ast::BehaviorMethod],
        effective_methods: &[EffectiveBehaviorImplMethod<'a>],
        span: Span,
    ) {
        self.validate_behavior_impl_declared_methods(
            behavior_key,
            required_methods,
            effective_methods,
        );

        for required in required_methods {
            self.validate_behavior_impl_required_method(
                type_name,
                behavior_key,
                required,
                effective_methods,
                span,
            );
        }
    }

    fn validate_behavior_impl_declared_methods<'a>(
        &mut self,
        behavior_key: &str,
        required_methods: &[ast::BehaviorMethod],
        effective_methods: &[EffectiveBehaviorImplMethod<'a>],
    ) {
        for method in effective_methods {
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
    }

    fn validate_behavior_impl_required_method<'a>(
        &mut self,
        type_name: &str,
        behavior_key: &str,
        required: &ast::BehaviorMethod,
        effective_methods: &[EffectiveBehaviorImplMethod<'a>],
        span: Span,
    ) {
        let Some(actual) = self.behavior_impl_actual_method_signature(
            type_name,
            &required.name,
            effective_methods,
        ) else {
            if required.default_body.is_some() {
                return;
            }
            self.diagnostics.push(Diagnostic::error(
                "E6001",
                format!(
                    "type `{}` implementation of `{}` is missing required method `{}`",
                    type_name, behavior_key, required.name
                ),
                span,
            ));
            return;
        };

        if actual.param_types.len() != required.params.len() {
            self.diagnostics.push(Diagnostic::error(
                "E6002",
                format!(
                    "method `{}` for behavior `{}` expects {} parameters, found {}",
                    required.name,
                    behavior_key,
                    required.params.len(),
                    actual.param_types.len()
                ),
                actual.span,
            ));
            return;
        }

        for (idx, (expected, actual_ty)) in required
            .params
            .iter()
            .zip(actual.param_types.iter())
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
                    actual
                        .params
                        .get(idx)
                        .map(|param| param.span)
                        .unwrap_or(actual.span),
                ));
            }
        }

        let expected_return = required.return_type.as_ref().unwrap_or(&AstType::Void);
        if !self.impl_ast_types_compatible(expected_return, &actual.return_type, type_name) {
            self.diagnostics.push(Diagnostic::error(
                "E6002",
                format!(
                    "method `{}` for behavior `{}` expects return `{}`, found `{}`",
                    required.name,
                    behavior_key,
                    self.impl_type_display(expected_return, type_name),
                    actual.return_type.display_name()
                ),
                actual.span,
            ));
        }
    }

    fn behavior_impl_actual_method_signature<'a>(
        &self,
        type_name: &str,
        required_name: &str,
        effective_methods: &[EffectiveBehaviorImplMethod<'a>],
    ) -> Option<BehaviorImplActualMethodSignature<'a>> {
        let (params, return_type, span) =
            effective_methods
                .iter()
                .find_map(|method| match method.declaration {
                    Declaration::Function {
                        params,
                        return_type,
                        span,
                        ..
                    } if method.method_name == required_name => Some((params, return_type, *span)),
                    _ => None,
                })?;

        let collected_signature = self.resolver_backed_method_signature(type_name, required_name);
        let param_types = collected_signature
            .map(|info| {
                info.params
                    .iter()
                    .map(|(_, ty)| ty.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| params.iter().map(|param| param.ty.clone()).collect());
        let return_type = collected_signature
            .map(|info| info.return_type.clone())
            .unwrap_or_else(|| return_type.clone().unwrap_or(AstType::Void));

        Some(BehaviorImplActualMethodSignature {
            params,
            param_types,
            return_type,
            span,
        })
    }
}
