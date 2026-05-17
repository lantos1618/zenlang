use super::*;

impl TypeChecker {
    pub(super) fn resolver_impl_ref_for(
        &mut self,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        self.resolver_behavior_ref_for(BehaviorRefRole::Impl, type_name, behavior)
    }

    pub(super) fn resolver_behavior_ref_for(
        &mut self,
        role: BehaviorRefRole,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        match role {
            BehaviorRefRole::Impl => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_impl_refs,
                type_name,
                behavior,
            ),
            BehaviorRefRole::Required => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_required_refs,
                type_name,
                behavior,
            ),
            BehaviorRefRole::Parent => None,
        }
    }

    pub(super) fn behavior_ref_parts<'a>(
        resolver_ref: Option<&'a BehaviorRefMetadata>,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        resolver_ref
            .map(|reference| (reference.name.as_str(), reference.type_args.as_slice()))
            .unwrap_or((behavior, behavior_type_args))
    }

    pub(super) fn pop_resolver_behavior_ref(
        resolver_backed_collection: bool,
        refs_by_type: &mut HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get_mut(type_name)?;
        Self::pop_resolver_behavior_ref_from_queue(refs, behavior)
    }

    pub(super) fn should_skip_missing_resolver_behavior_ref(
        &self,
        resolver_ref: Option<&BehaviorRefMetadata>,
        type_name: &str,
        missing_refs: &HashSet<String>,
    ) -> bool {
        self.resolver_backed_collection
            && resolver_ref.is_none()
            && missing_refs.contains(type_name)
    }

    pub(super) fn pop_resolver_behavior_ref_from_queue(
        refs: &mut VecDeque<BehaviorRefMetadata>,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        let index = Self::resolver_behavior_ref_queue_index(refs, behavior)?;
        refs.remove(index)
    }

    pub(super) fn resolver_behavior_impl_ref_for_peek(
        &self,
        type_name: &str,
        behavior: &str,
    ) -> Option<&BehaviorRefMetadata> {
        Self::peek_resolver_behavior_ref(
            self.resolver_backed_collection,
            &self.resolver_behavior_impl_refs,
            type_name,
            behavior,
        )
    }

    pub(super) fn peek_resolver_behavior_ref<'a>(
        resolver_backed_collection: bool,
        refs_by_type: &'a HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
    ) -> Option<&'a BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get(type_name)?;
        Self::resolver_behavior_ref_queue_index(refs, behavior).and_then(|index| refs.get(index))
    }

    pub(super) fn resolver_behavior_ref_queue_index(
        refs: &VecDeque<BehaviorRefMetadata>,
        behavior: &str,
    ) -> Option<usize> {
        Self::named_queue_index(refs, behavior, |reference| reference.name.as_str())
    }

    pub(super) fn named_queue_index<T>(
        items: &VecDeque<T>,
        name: &str,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        items
            .iter()
            .position(|item| item_name(item) == name)
            .or_else(|| (!items.is_empty()).then_some(0))
    }

    pub(super) fn named_queue_index_preserving_future_front<'a, T>(
        items: &VecDeque<T>,
        name: &str,
        future_names: impl IntoIterator<Item = &'a str>,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        if let Some(index) = items.iter().position(|item| item_name(item) == name) {
            return Some(index);
        }

        let front_name = item_name(items.front()?);
        (!future_names
            .into_iter()
            .any(|future_name| future_name == front_name))
        .then_some(0)
    }

    pub(super) fn resolver_behavior_impl_ref_parts<'a>(
        &'a self,
        type_name: &str,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        match self.resolver_behavior_impl_ref_for_peek(type_name, behavior) {
            Some(implementation) => (
                implementation.name.as_str(),
                implementation.type_args.as_slice(),
            ),
            None => (behavior, behavior_type_args),
        }
    }

    pub(super) fn find_overlapping_behavior_impl(
        &self,
        type_name: &str,
        behavior: &str,
    ) -> Option<String> {
        self.behavior_impls
            .iter()
            .filter(|(implemented_type, _)| implemented_type == type_name)
            .map(|(_, implemented_behavior)| implemented_behavior)
            .find(|implemented_behavior| {
                self.behavior_inherits_from(implemented_behavior, behavior)
                    || self.behavior_inherits_from(behavior, implemented_behavior)
            })
            .cloned()
    }

    pub(super) fn reject_unspecialized_generic_type(
        &mut self,
        type_name: &str,
        span: Span,
    ) -> bool {
        let type_param_count = self
            .structs
            .get(type_name)
            .map(|info| info.type_params.len())
            .or_else(|| self.enums.get(type_name).map(|info| info.type_params.len()))
            .unwrap_or(0);
        if type_param_count == 0 {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            "E6013",
            format!(
                "generic type `{}` expects {} type arguments, found 0",
                type_name, type_param_count
            ),
            span,
        ));
        true
    }

    pub(super) fn behavior_default_methods_for_impl(
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

    pub(super) fn seed_behavior_default_method_signature(
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

    pub(super) fn impl_methods_include_behavior_method(
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

    pub(super) fn behavior_methods_with_inherited_substituted(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        if !self.mark_behavior_seen(behavior, substitutions, seen) {
            return Vec::new();
        }

        let mut methods = Vec::new();
        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_substitutions =
                    self.behavior_parent_type_param_substitutions(parent, substitutions);
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

    pub(super) fn behavior_seen_key(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
    ) -> String {
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
        behavior_ref_display(behavior, &type_args)
    }

    pub(super) fn mark_behavior_seen(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> bool {
        let behavior_seen_key = self.behavior_seen_key(behavior, substitutions);
        seen.insert(behavior_seen_key)
    }

    pub(super) fn behavior_methods_for_impl(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        self.behavior_methods_with_inherited_substituted(behavior, substitutions, seen)
    }

    pub(super) fn behavior_type_param_substitutions(
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

    pub(super) fn behavior_parent_type_param_substitutions(
        &self,
        parent: &BehaviorParentRef,
        substitutions: &HashMap<String, AstType>,
    ) -> HashMap<String, AstType> {
        let parent_type_args: Vec<AstType> = parent
            .type_args
            .iter()
            .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
            .collect();
        self.behavior_type_param_substitutions(&parent.behavior, &parent_type_args)
    }

    pub(super) fn impl_ast_types_compatible(
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

    pub(super) fn impl_type_display(&self, ty: &AstType, self_type_name: &str) -> String {
        match ty {
            AstType::SelfType => self_type_name.to_string(),
            _ => ty.display_name(),
        }
    }
}
