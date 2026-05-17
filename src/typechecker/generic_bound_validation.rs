use super::*;

impl TypeChecker {
    pub(super) fn validate_generic_bounds(&mut self, type_params: &[ast::TypeParam]) {
        for param in type_params {
            if let Some(bound) = &param.constraint {
                if !self.behaviors.contains_key(bound) {
                    self.diagnostics.push(Diagnostic::error(
                        "E5002",
                        format!(
                            "generic bound `{}` on type parameter `{}` references undefined behavior",
                            bound, param.name
                        ),
                        param.span,
                    ));
                } else {
                    let expected = self
                        .behaviors
                        .get(bound)
                        .map(|info| info.type_params.len())
                        .unwrap_or(0);
                    let found = param.constraint_type_args.len();
                    if expected == 0 && found > 0 {
                        self.diagnostics.push(Diagnostic::error(
                            "E5002",
                            format!(
                                "non-generic behavior `{}` does not accept type arguments",
                                bound
                            ),
                            param.span,
                        ));
                        continue;
                    }
                    if expected != found {
                        self.diagnostics.push(Diagnostic::error(
                            "E6012",
                            format!(
                                "generic behavior `{}` expects {} type arguments, found {}",
                                bound, expected, found
                            ),
                            param.span,
                        ));
                    }
                }
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
            let behavior_key = self.behavior_bound_key(bound, substitutions);
            let behavior_display = behavior_bound_display(bound, substitutions);
            let Some(type_name) = Self::behavior_bound_type_name(concrete) else {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        concrete.display_name(),
                        behavior_display,
                        param
                    ),
                    span,
                ));
                continue;
            };
            if !self.type_implements_behavior(&type_name, &behavior_key) {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        type_name, behavior_display, param
                    ),
                    span,
                ));
            }
        }
    }

    fn behavior_bound_key(
        &self,
        bound: &BehaviorBound,
        substitutions: &HashMap<String, Type>,
    ) -> String {
        let type_args = substitute_behavior_bound_type_args(&bound.type_args, substitutions);
        self.behavior_reference_key(&bound.behavior, &type_args)
    }

    fn behavior_bound_type_name(ty: &Type) -> Option<String> {
        match ty {
            Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => {
                Some(name.clone())
            }
            _ => None,
        }
    }
}
