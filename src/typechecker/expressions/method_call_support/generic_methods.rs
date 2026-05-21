use super::*;
use crate::typechecker::FuncInfo;

impl TypeChecker {
    pub(super) fn resolve_generic_method_call(
        &mut self,
        method_key: &str,
        info: &FuncInfo,
        type_args: &[AstType],
        typed_args: &[TypedExpression],
        span: Span,
    ) -> (String, Type) {
        let (subs, type_args_valid) = if type_args.is_empty() {
            let arg_types: Vec<Type> = typed_args.iter().map(|arg| arg.ty.clone()).collect();
            let (subs, conflicts) = self.infer_method_type_args(
                method_key,
                &info.type_params,
                &info.params,
                &arg_types,
            );
            let inferred_type_args_valid =
                self.report_inference_conflicts("method", method_key, conflicts, span);
            (subs, inferred_type_args_valid)
        } else {
            self.explicit_type_arg_substitutions(
                "method",
                method_key,
                &info.type_params,
                type_args,
                span,
            )
        };

        let fallback_mangled =
            self.generic_function_mangled_name(method_key, &info.type_params, &subs);
        if !type_args_valid {
            return (fallback_mangled, Type::Unknown);
        }

        let saved_self_type = self.current_self_type.clone();
        self.current_self_type = self.generic_method_self_type(method_key, &subs);
        self.check_call_signature_with_substitutions(
            "method",
            method_key,
            &info.params,
            typed_args,
            &subs,
            &span,
        );

        let result = if self.check_generic_bounds_valid(&info.type_param_bounds, &subs, span) {
            let ret_type = self.substitute_type(&info.return_type, &subs);
            let mangled = self
                .specialize_generic_method(method_key, &subs, span)
                .unwrap_or(fallback_mangled);
            (mangled, ret_type)
        } else {
            (fallback_mangled, Type::Unknown)
        };

        self.current_self_type = saved_self_type;
        result
    }
}
