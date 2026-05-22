use super::*;

pub(super) struct ResolverBehaviorImplMethodSignatureCollection<'a> {
    pub(super) symbols: &'a SymbolTable,
    pub(super) ast_type_name: &'a str,
    pub(super) type_name: &'a str,
    pub(super) target_type_args: &'a [AstType],
    pub(super) behavior: &'a str,
    pub(super) behavior_type_args: &'a [AstType],
    pub(super) methods: &'a [Declaration],
}

impl TypeChecker {
    pub(super) fn collect_impl_method_signature(
        &mut self,
        type_name: &str,
        type_args: &[AstType],
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        method: &Declaration,
    ) {
        let Declaration::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = method
        else {
            return;
        };

        self.validate_generic_bounds(type_params);
        let key = Self::behavior_impl_method_key_with_target_args(
            type_name,
            name,
            behavior,
            behavior_type_args,
            type_args,
        );
        self.methods.insert(
            key.clone(),
            func_info_from_ast_signature(key.clone(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, *span)
        {
            self.generic_methods.insert(key, template);
        }
    }

    pub(super) fn collect_resolver_backed_impl_method_template(
        &mut self,
        type_name: &str,
        method: &Declaration,
    ) {
        let Declaration::Function {
            name,
            type_params,
            params,
            body,
            span,
            ..
        } = method
        else {
            return;
        };
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, *span)
        {
            self.generic_methods
                .insert(Self::method_key(type_name, name), template);
        }
    }

    pub(super) fn collect_resolver_behavior_impl_method_signatures(
        &mut self,
        task: ResolverBehaviorImplMethodSignatureCollection<'_>,
    ) {
        let ResolverBehaviorImplMethodSignatureCollection {
            symbols,
            ast_type_name,
            type_name,
            target_type_args,
            behavior,
            behavior_type_args,
            methods,
        } = task;
        let (behavior, behavior_type_args) =
            self.resolver_behavior_impl_ref_parts(type_name, behavior, behavior_type_args);
        let behavior = behavior.to_string();
        let behavior_type_args = behavior_type_args.to_vec();
        let behavior_substitutions =
            self.behavior_type_param_substitutions(&behavior, &behavior_type_args);
        let mut required_methods: VecDeque<ast::BehaviorMethod> = self
            .behavior_methods_for_impl(&behavior, &behavior_substitutions, &mut HashSet::new())
            .into_iter()
            .collect();

        for method in methods {
            let Declaration::Function { name, span, .. } = method else {
                continue;
            };
            let ast_key = Self::method_key(ast_type_name, name);
            let resolver_owned_key =
                self.resolver_backed_impl_method_key(Some(symbols), &ast_key, type_name, *span);
            let restored_name = self.resolver_backed_behavior_impl_method_signature_name(
                &mut required_methods,
                name,
                resolver_owned_key.as_deref(),
                type_name,
            );
            let Some(restored_name) = restored_name else {
                continue;
            };
            let restored_key = resolver_owned_key.unwrap_or_else(|| {
                Self::behavior_impl_method_key_with_target_args(
                    type_name,
                    &restored_name,
                    Some(&behavior),
                    &behavior_type_args,
                    target_type_args,
                )
            });
            self.collect_resolver_callable_signature_for_key(
                symbols,
                &ast_key,
                &restored_key,
                *span,
            );
        }
    }

    pub(super) fn collect_behavior_default_method_signatures(
        &mut self,
        type_name: &str,
        type_args: &[AstType],
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
        if self.should_skip_behavior_default_synthesis(type_name) {
            return;
        }
        let (behavior, behavior_type_args) =
            self.resolver_behavior_impl_ref_parts(type_name, behavior, behavior_type_args);
        for default in self.behavior_default_methods_for_impl(
            type_name,
            type_args,
            behavior,
            behavior_type_args,
            methods,
        ) {
            self.seed_behavior_default_method_signature(type_name, &default);
        }
    }

    pub(super) fn should_skip_behavior_default_synthesis(&self, type_name: &str) -> bool {
        self.resolver_backed_collection
            && self.resolver_missing_behavior_impl_refs.contains(type_name)
    }
}
