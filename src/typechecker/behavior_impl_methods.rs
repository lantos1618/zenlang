use super::*;

impl TypeChecker {
    pub(super) fn impl_effective_method_name(
        &self,
        unmatched_required: &mut VecDeque<String>,
        ast_name: &str,
        resolver_owned_key: Option<String>,
        type_name: &str,
    ) -> String {
        if let Some(resolver_owned_key) = resolver_owned_key {
            let resolver_owned_name =
                method_signature_method_name_for_receiver(&resolver_owned_key, type_name)
                    .unwrap_or(&resolver_owned_key)
                    .to_string();
            let required_name = Self::behavior_impl_required_method_name(&resolver_owned_name);
            return Self::remove_named_queue_entry(unmatched_required, required_name)
                .unwrap_or_else(|| required_name.to_string());
        }

        if let Some(name) = Self::remove_named_queue_entry(unmatched_required, ast_name) {
            return name;
        }

        if self.resolver_backed_collection {
            if let Some(index) = unmatched_required.iter().position(|required| {
                self.resolver_backed_method_signature(type_name, required)
                    .is_some()
            }) {
                return unmatched_required
                    .remove(index)
                    .unwrap_or_else(|| ast_name.to_string());
            }
        }

        ast_name.to_string()
    }

    pub(super) fn effective_behavior_impl_methods<'a>(
        &self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        methods: &'a [Declaration],
        unmatched_required: &mut VecDeque<String>,
    ) -> Vec<EffectiveBehaviorImplMethod<'a>> {
        methods
            .iter()
            .map(|method| {
                let ast_name = match method {
                    Declaration::Function { name, .. } => name.as_str(),
                    _ => "",
                };
                let ast_key = Self::method_key(type_name, ast_name);
                let resolver_owned_name = self.resolver_backed_impl_method_key(
                    symbols,
                    &ast_key,
                    type_name,
                    method.span(),
                );
                let method_name = self.impl_effective_method_name(
                    unmatched_required,
                    ast_name,
                    resolver_owned_name,
                    type_name,
                );
                EffectiveBehaviorImplMethod {
                    declaration: method,
                    method_name,
                }
            })
            .collect()
    }

    pub(super) fn resolver_backed_behavior_impl_method_signature_name(
        &self,
        required_methods: &mut VecDeque<ast::BehaviorMethod>,
        ast_name: &str,
        resolver_owned_key: Option<&str>,
        type_name: &str,
    ) -> Option<String> {
        if let Some(resolver_owned_key) = resolver_owned_key {
            let resolver_owned_name =
                method_signature_method_name_for_receiver(resolver_owned_key, type_name)
                    .unwrap_or(resolver_owned_key);
            let resolver_owned_name = Self::behavior_impl_required_method_name(resolver_owned_name);
            if let Some(index) =
                Self::named_queue_index(required_methods, resolver_owned_name, |required| {
                    required.name.as_str()
                })
            {
                return required_methods.remove(index).map(|required| required.name);
            }
        }

        Self::named_queue_index(required_methods, ast_name, |required| {
            required.name.as_str()
        })
        .and_then(|index| required_methods.remove(index).map(|required| required.name))
    }

    pub(super) fn resolver_backed_impl_method_key(
        &self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> Option<String> {
        self.resolver_backed_collection
            .then(|| Self::validation_method_key(symbols, ast_key, type_name, span))
    }

    pub(super) fn resolver_backed_method_signature(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<&FuncInfo> {
        self.resolver_backed_collection
            .then(|| self.methods.get(&Self::method_key(type_name, method_name)))
            .flatten()
    }

    pub(super) fn method_key(type_name: &str, method_name: &str) -> String {
        method_signature_key(type_name, method_name)
    }

    pub(super) fn behavior_impl_method_key(
        type_name: &str,
        method_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
    ) -> String {
        behavior_impl_method_signature_key(type_name, method_name, behavior, behavior_type_args)
    }

    pub(super) fn behavior_impl_required_method_name(method_name: &str) -> &str {
        method_name
            .split_once("__")
            .map_or(method_name, |(name, _)| name)
    }

    pub(super) fn remove_named_queue_entry(
        items: &mut VecDeque<String>,
        name: &str,
    ) -> Option<String> {
        items
            .iter()
            .position(|item| item == name)
            .and_then(|index| items.remove(index))
    }
}
