use super::*;

impl TypeChecker {
    pub(in crate::typechecker) fn behavior_default_methods_for_impl(
        &self,
        type_name: &str,
        type_args: &[AstType],
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) -> Vec<Declaration> {
        let behavior_substitutions =
            self.behavior_type_param_substitutions(behavior, behavior_type_args);
        let type_params = named_type_arg_params(type_args);
        self.behavior_methods_with_inherited_substituted(
            behavior,
            &behavior_substitutions,
            &mut HashSet::new(),
        )
        .iter()
        .filter(|required| {
            !methods
                .iter()
                .any(|decl| decl.name() == Some(required.name.as_str()))
        })
        .filter_map(|required| {
            let body = required.default_body.clone()?;
            Some(Declaration::Function {
                name: required.name.clone(),
                type_params: type_params.clone(),
                params: required
                    .params
                    .iter()
                    .map(|param| Param {
                        name: param.name.clone(),
                        ty: concrete_self_ast_type_for_target(&param.ty, type_name, type_args),
                        mutable: param.mutable,
                        span: param.span,
                    })
                    .collect(),
                return_type: required
                    .return_type
                    .as_ref()
                    .map(|ty| concrete_self_ast_type_for_target(ty, type_name, type_args)),
                body,
                public: true,
                external: false,
                span: required.span,
            })
        })
        .collect()
    }

    pub(in crate::typechecker) fn seed_behavior_default_method_signature(
        &mut self,
        type_name: &str,
        default: &Declaration,
    ) {
        let Some(name) = default.name() else {
            return;
        };
        insert_callable_signature(
            method_signature_key(type_name, name),
            default,
            &mut self.methods,
            &mut self.generic_methods,
        );
    }

    pub(in crate::typechecker) fn behavior_methods_with_inherited_substituted(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        let type_args = self
            .behaviors
            .get(behavior)
            .map(|info| {
                info.type_params
                    .iter()
                    .filter_map(|param| substitutions.get(param).cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !seen.insert(behavior_ref_display(behavior, &type_args)) {
            return Vec::new();
        }

        let mut methods = Vec::new();
        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_type_args: Vec<AstType> = parent
                    .type_args
                    .iter()
                    .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
                    .collect();
                let parent_substitutions =
                    self.behavior_type_param_substitutions(&parent.behavior, &parent_type_args);
                methods.extend(self.behavior_methods_with_inherited_substituted(
                    &parent.behavior,
                    &parent_substitutions,
                    seen,
                ));
            }
        }
        if let Some(info) = self.behaviors.get(behavior) {
            methods.extend(
                info.methods
                    .iter()
                    .map(|method| substituted_behavior_method_signature(method, substitutions)),
            );
        }
        methods
    }

    pub(in crate::typechecker) fn behavior_type_param_substitutions(
        &self,
        behavior: &str,
        type_args: &[AstType],
    ) -> HashMap<String, AstType> {
        self.behaviors
            .get(behavior)
            .map(|info| {
                info.type_params
                    .iter()
                    .cloned()
                    .zip(type_args.iter().cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
}
