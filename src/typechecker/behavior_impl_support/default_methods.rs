use super::*;

impl TypeChecker {
    pub(in crate::typechecker) fn behavior_default_methods_for_impl(
        &self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) -> Vec<DefaultBehaviorMethod> {
        let behavior_substitutions =
            self.behavior_type_param_substitutions(behavior, behavior_type_args);
        self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new())
            .iter()
            .filter(|required| {
                required.default_body.is_some()
                    && !self.impl_methods_include_behavior_method(
                        type_name,
                        methods,
                        &required.name,
                    )
            })
            .filter_map(|required| {
                let body = required.default_body.clone()?;
                Some(DefaultBehaviorMethod {
                    name: required.name.clone(),
                    params: required
                        .params
                        .iter()
                        .map(|param| Param {
                            name: param.name.clone(),
                            ty: concrete_self_ast_type(&param.ty, type_name),
                            mutable: param.mutable,
                            span: param.span,
                        })
                        .collect(),
                    return_type: required
                        .return_type
                        .as_ref()
                        .map(|ty| concrete_self_ast_type(ty, type_name)),
                    body,
                    span: required.span,
                })
            })
            .collect()
    }

    pub(in crate::typechecker) fn seed_behavior_default_method_signature(
        &mut self,
        type_name: &str,
        default: &DefaultBehaviorMethod,
    ) {
        let key = Self::method_key(type_name, &default.name);
        self.methods.insert(
            key.clone(),
            func_info_from_behavior_method(key, &default.params, &default.return_type),
        );
    }

    pub(in crate::typechecker) fn impl_methods_include_behavior_method(
        &self,
        type_name: &str,
        methods: &[Declaration],
        required_name: &str,
    ) -> bool {
        methods
            .iter()
            .any(|decl| matches!(decl, Declaration::Function { name, .. } if name == required_name))
            || (self.resolver_backed_collection
                && self
                    .resolver_backed_method_signature(type_name, required_name)
                    .is_some())
    }

    pub(in crate::typechecker) fn impl_ast_types_compatible(
        &self,
        expected: &AstType,
        actual: &AstType,
        self_type_name: &str,
    ) -> bool {
        match expected {
            AstType::SelfType => matches!(actual, AstType::Named(name) if name == self_type_name),
            _ => expected == actual,
        }
    }

    pub(in crate::typechecker) fn impl_type_display(
        &self,
        ty: &AstType,
        self_type_name: &str,
    ) -> String {
        match ty {
            AstType::SelfType => self_type_name.to_string(),
            _ => ty.display_name(),
        }
    }
}
