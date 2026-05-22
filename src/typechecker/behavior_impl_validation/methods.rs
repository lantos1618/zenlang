use crate::ast::{self, AstType, Declaration};
use crate::error::{Diagnostic, Span};

use super::super::{EffectiveBehaviorImplMethod, TypeChecker};

impl TypeChecker {
    pub(super) fn check_behavior_impl_extra_methods(
        &mut self,
        behavior_key: &str,
        required_methods: &[ast::BehaviorMethod],
        effective_methods: &[EffectiveBehaviorImplMethod<'_>],
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

    pub(super) fn check_behavior_impl_required_methods(
        &mut self,
        type_name: &str,
        behavior_key: &str,
        required_methods: &[ast::BehaviorMethod],
        effective_methods: &[EffectiveBehaviorImplMethod<'_>],
        span: Span,
    ) {
        for required in required_methods {
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
