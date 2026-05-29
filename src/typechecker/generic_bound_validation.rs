use super::generics::monomorphize_types::substitute_ast_type;
use super::*;

impl TypeChecker {
    pub(super) fn validate_generic_bounds(&mut self, type_params: &[ast::TypeParam]) {
        for param in type_params {
            if let Some(bound) = &param.constraint {
                let Some(expected) = self.behaviors.get(bound).map(|info| info.type_params.len())
                else {
                    self.push_error(
                        E5002,
                        format!("generic bound `{bound}` on type parameter `{}` references undefined behavior", param.name),
                        param.span,
                    );
                    continue;
                };
                self.validate_type_arg_arity(
                    "behavior",
                    bound,
                    expected,
                    &param.constraint_type_args,
                    param.span,
                );
            }
        }
    }

    pub(crate) fn check_generic_bounds(
        &mut self,
        bounds: &HashMap<String, BehaviorBound>,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) {
        for (param, bound) in bounds {
            let Some(concrete) = substitutions.get(param) else {
                continue;
            };
            let type_args = bound
                .type_args
                .iter()
                .map(|arg| substitute_ast_type(arg, substitutions))
                .collect::<Vec<_>>();
            let behavior_key = self.behavior_reference_key(&bound.behavior, &type_args);
            let behavior_display = behavior_ref_display(&bound.behavior, &type_args);
            let Some(type_name) = concrete.nominal_name() else {
                let concrete_name = concrete.display_name();
                self.push_error(
                    E6004,
                    format!("type `{concrete_name}` does not implement behavior `{behavior_display}` required by `{param}`"),
                    span,
                );
                continue;
            };
            if !self.type_implements_behavior(type_name, &behavior_key)
                && !self.generic_type_implements_behavior(concrete, &behavior_key)
            {
                self.push_error(
                    E6004,
                    format!("type `{type_name}` does not implement behavior `{behavior_display}` required by `{param}`"),
                    span,
                );
            }
        }
    }

    fn generic_type_implements_behavior(&self, concrete: &Type, behavior_key: &str) -> bool {
        let Some(type_name) = concrete.nominal_name() else {
            return false;
        };
        let Some((generic_name, type_args)) = self.generic_type_args_from_type(type_name, concrete)
        else {
            return false;
        };

        self.generic_behavior_impls.iter().any(|template| {
            if template.type_name != generic_name || template.type_params.len() != type_args.len() {
                return false;
            }
            let substitutions = self.type_arg_substitutions(&template.type_params, &type_args);
            let behavior_type_args: Vec<AstType> = template
                .behavior_type_args
                .iter()
                .map(|arg| substitute_ast_type(arg, &substitutions))
                .collect();
            let implemented_key =
                self.behavior_reference_key(&template.behavior, &behavior_type_args);
            if implemented_key == behavior_key {
                return true;
            }
            let behavior_ref = self.behavior_parent_ref(&template.behavior, &behavior_type_args);
            self.behavior_ref_inherits_from_inner(&behavior_ref, behavior_key, &mut HashSet::new())
        })
    }
}
