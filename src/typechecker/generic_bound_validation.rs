use super::*;

impl TypeChecker {
    pub(super) fn validate_generic_bounds(&mut self, type_params: &[ast::TypeParam]) {
        for param in type_params {
            if let Some(bound) = &param.constraint {
                self.validate_generic_behavior_bound(
                    &param.name,
                    bound,
                    &param.constraint_type_args,
                    param.span,
                );
            }
        }
    }

    pub(super) fn validate_restored_generic_bounds(
        &mut self,
        bounds: &HashMap<String, BehaviorBound>,
        span: Span,
        symbols: &SymbolTable,
    ) {
        for (param, bound) in bounds {
            self.validate_generic_behavior_bound_with_symbols(
                param,
                &bound.behavior,
                &bound.type_args,
                span,
                Some(symbols),
            );
        }
    }

    fn validate_generic_behavior_bound(
        &mut self,
        param: &str,
        behavior: &str,
        type_args: &[AstType],
        span: Span,
    ) {
        self.validate_generic_behavior_bound_with_symbols(param, behavior, type_args, span, None);
    }

    fn validate_generic_behavior_bound_with_symbols(
        &mut self,
        param: &str,
        behavior: &str,
        type_args: &[AstType],
        span: Span,
        symbols: Option<&SymbolTable>,
    ) {
        let Some(expected) = self.generic_behavior_type_param_count(behavior, symbols) else {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "generic bound `{}` on type parameter `{}` references undefined behavior",
                    behavior, param
                ),
                span,
            ));
            return;
        };

        let found = type_args.len();
        if expected == 0 && found > 0 {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic behavior `{}` does not accept type arguments",
                    behavior
                ),
                span,
            ));
            return;
        }
        if expected != found {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic behavior `{}` expects {} type arguments, found {}",
                    behavior, expected, found
                ),
                span,
            ));
        }
    }

    fn generic_behavior_type_param_count(
        &self,
        behavior: &str,
        symbols: Option<&SymbolTable>,
    ) -> Option<usize> {
        symbols
            .and_then(|symbols| symbols.lookup(Namespace::Behavior, behavior))
            .and_then(|symbol| {
                symbol
                    .type_parameter_names
                    .as_ref()
                    .map(Vec::len)
                    .or(symbol.type_parameter_count)
            })
            .or_else(|| {
                self.behaviors
                    .get(behavior)
                    .map(|info| info.type_params.len())
            })
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
