mod coherence;
mod method_lookup;
mod type_matching;

use super::*;
use method_lookup::{generic_behavior_actual_method, GenericBehaviorActualMethod};
use type_matching::{generic_impl_ast_types_compatible, generic_impl_type_display};

#[derive(Clone, Copy)]
struct GenericBehaviorImplTemplateContext<'a> {
    type_name: &'a str,
    type_args: &'a [AstType],
    behavior: &'a str,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
    span: Span,
}

impl TypeChecker {
    pub(super) fn check_generic_behavior_impl_template(
        &mut self,
        type_name: &str,
        type_args: &[AstType],
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
    ) {
        let type_params = named_type_arg_names(type_args);
        if type_params.len() != type_args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic behavior implementation target `{type_name}` must use named type parameters"
                ),
                span,
            ));
            return;
        }

        let Some(expected_type_arg_count) = self
            .structs
            .get(type_name)
            .map(|info| info.type_params.len())
            .or_else(|| self.enums.get(type_name).map(|info| info.type_params.len()))
        else {
            return;
        };
        if expected_type_arg_count != type_args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic type `{}` expects {} type arguments, found {}",
                    type_name,
                    expected_type_arg_count,
                    type_args.len()
                ),
                span,
            ));
            return;
        }

        let scoped: HashSet<String> = type_params.iter().cloned().collect();
        let Some(behavior_substitutions) =
            self.behavior_type_arg_substitutions(behavior, behavior_type_args, &scoped, span)
        else {
            return;
        };
        if self.reject_generic_behavior_impl_coherence_conflict(
            type_name,
            type_args,
            behavior,
            behavior_type_args,
            span,
        ) {
            return;
        }
        let required_methods =
            self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new());

        let context = GenericBehaviorImplTemplateContext {
            type_name,
            type_args,
            behavior,
            behavior_type_args,
            methods,
            span,
        };
        self.check_generic_behavior_extra_methods(&context, &required_methods);
        self.check_generic_behavior_required_methods(context, &required_methods);
        self.record_generic_behavior_impl_ref(behavior, behavior_type_args);
        self.generic_behavior_impls
            .push(GenericBehaviorImplTemplate {
                type_name: type_name.to_string(),
                type_params,
                behavior: behavior.to_string(),
                behavior_type_args: behavior_type_args.to_vec(),
            });
    }

    fn check_generic_behavior_extra_methods(
        &mut self,
        context: &GenericBehaviorImplTemplateContext<'_>,
        required_methods: &[BehaviorMethod],
    ) {
        for method in context.methods {
            if let Declaration::Function { name, span, .. } = method {
                if !required_methods
                    .iter()
                    .any(|required| required.name == *name)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E6005",
                        format!(
                            "method `{}` is not declared by behavior `{}`",
                            name,
                            behavior_ref_display(context.behavior, context.behavior_type_args)
                        ),
                        *span,
                    ));
                }
            }
        }
    }

    fn check_generic_behavior_required_methods(
        &mut self,
        context: GenericBehaviorImplTemplateContext<'_>,
        required_methods: &[BehaviorMethod],
    ) {
        for required in required_methods {
            let Some(actual) = generic_behavior_actual_method(context.methods, &required.name)
            else {
                if required.default_body.is_some() {
                    continue;
                }
                self.diagnostics.push(Diagnostic::error(
                    "E6001",
                    format!(
                        "generic type `{}` implementation of `{}` is missing required method `{}`",
                        context.type_name,
                        behavior_ref_display(context.behavior, context.behavior_type_args),
                        required.name
                    ),
                    context.span,
                ));
                continue;
            };

            if actual.params.len() != required.params.len() {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects {} parameters, found {}",
                        required.name,
                        behavior_ref_display(context.behavior, context.behavior_type_args),
                        required.params.len(),
                        actual.params.len()
                    ),
                    actual.span,
                ));
                continue;
            }
            self.check_generic_behavior_method_signature(context, required, actual);
        }
    }

    fn check_generic_behavior_method_signature(
        &mut self,
        context: GenericBehaviorImplTemplateContext<'_>,
        required: &BehaviorMethod,
        actual: GenericBehaviorActualMethod<'_>,
    ) {
        for (idx, (expected, actual_param)) in required.params.iter().zip(actual.params).enumerate()
        {
            if !generic_impl_ast_types_compatible(
                &expected.ty,
                &actual_param.ty,
                context.type_name,
                context.type_args,
            ) {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "parameter {} for method `{}` in behavior `{}` expects `{}`, found `{}`",
                        idx + 1,
                        required.name,
                        behavior_ref_display(context.behavior, context.behavior_type_args),
                        generic_impl_type_display(
                            &expected.ty,
                            context.type_name,
                            context.type_args
                        ),
                        actual_param.ty.display_name()
                    ),
                    actual_param.span,
                ));
            }
        }

        let expected_return = required.return_type.as_ref().unwrap_or(&AstType::Void);
        let actual_return = actual.return_type.clone().unwrap_or(AstType::Void);
        if !generic_impl_ast_types_compatible(
            expected_return,
            &actual_return,
            context.type_name,
            context.type_args,
        ) {
            self.diagnostics.push(Diagnostic::error(
                "E6002",
                format!(
                    "method `{}` for behavior `{}` expects return `{}`, found `{}`",
                    required.name,
                    behavior_ref_display(context.behavior, context.behavior_type_args),
                    generic_impl_type_display(
                        expected_return,
                        context.type_name,
                        context.type_args
                    ),
                    actual_return.display_name()
                ),
                actual.span,
            ));
        }
    }
}
