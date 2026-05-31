use super::*;

impl TypeChecker {
    pub(super) fn validate_type_ref_bounds(
        &mut self,
        name: &str,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        if scoped_type_params.contains(name) {
            return;
        }

        let Some((kind, type_params, type_param_bounds)) = self.generic_type_decl(name) else {
            // Unknown bare type names are caught by the resolver before the
            // typechecker runs; nothing to do for non-generic names here.
            return;
        };

        let type_params = type_params.to_vec();
        let type_param_bounds = type_param_bounds.clone();
        let type_args = self.fill_type_arg_defaults(name, type_args);
        if !self.validate_type_arg_arity(kind, name, type_params.len(), &type_args, span) {
            return;
        }

        let substitutions =
            self.concrete_type_arg_substitutions(&type_params, &type_args, scoped_type_params);
        self.check_generic_bounds(&type_param_bounds, &substitutions, span);
    }
}
