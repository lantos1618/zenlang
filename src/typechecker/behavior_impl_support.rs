use super::*;

mod default_methods;

impl TypeChecker {
    pub(super) fn reject_unspecialized_generic_type(
        &mut self,
        type_name: &str,
        span: Span,
    ) -> bool {
        let type_param_count = self.type_params_for_type(type_name).map_or(0, <[_]>::len);
        if type_param_count == 0 {
            return false;
        }

        self.push_error(
            E6013,
            format!(
                "generic type `{}` expects {} type arguments, found 0",
                type_name, type_param_count
            ),
            span,
        );
        true
    }
}
