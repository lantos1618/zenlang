use super::*;

impl SymbolTable {
    #[cfg(test)]
    pub(crate) fn remove_for_test(&mut self, namespace: Namespace, name: &str) {
        self.symbols
            .retain(|symbol| symbol.namespace != namespace || symbol.name != name);
        self.by_name.clear();
        self.by_scoped_name.clear();
        for (idx, symbol) in self.symbols.iter_mut().enumerate() {
            symbol.id = SymbolId(idx as u32);
            if symbol.namespace != Namespace::Local {
                self.by_name
                    .insert((symbol.namespace, symbol.name.clone()), symbol.id);
            }
            self.by_scoped_name.insert(
                (symbol.namespace, symbol.name.clone(), symbol.scope_id),
                symbol.id,
            );
        }
    }

    #[cfg(test)]
    pub(crate) fn set_public_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.is_public = is_public;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_import_source_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        import_source: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.import_source = import_source;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_local_mutability_for_test(&mut self, name: &str, is_mutable: Option<bool>) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Local && symbol.name == name)
        {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_local_mutability_in_scope_for_test(
        &mut self,
        name: &str,
        scope_id: u32,
        is_mutable: Option<bool>,
    ) {
        if let Some(symbol) = self.symbols.iter_mut().find(|symbol| {
            symbol.namespace == Namespace::Local
                && symbol.name == name
                && symbol.scope_id == scope_id
        }) {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_mutability_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_mutable: Option<bool>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.parameter_count = parameter_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_type_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_type_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.parameter_type_names = parameter_type_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_types_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_types: Option<Vec<AstType>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.parameter_types = parameter_types;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.parameter_names = parameter_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_return_type_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        return_type_name: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.return_type_name = return_type_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_return_type_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        return_type: Option<AstType>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.return_type = return_type;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.type_parameter_count = type_parameter_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.type_parameter_names = type_parameter_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_bounds_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_bounds: Option<Vec<TypeParameterBoundMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.type_parameter_bounds = type_parameter_bounds;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_bound_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_bound_refs: Option<Vec<TypeParameterBoundRefMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.type_parameter_bound_refs = type_parameter_bound_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_method_signatures_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_method_signatures: Option<Vec<MethodSignatureMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_method_signatures = behavior_method_signatures;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_method_types_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_method_types: Option<Vec<BehaviorMethodTypeMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_method_types = behavior_method_types;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_parent_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_parent_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_parent_names = behavior_parent_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_parent_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_parent_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_parent_refs = behavior_parent_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_impl_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_impl_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_impl_names = behavior_impl_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_impl_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_impl_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_impl_refs = behavior_impl_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_required_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_required_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_required_names = behavior_required_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_required_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_required_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_required_refs = behavior_required_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_field_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.field_count = field_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_field_type_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_type_names: Option<Vec<(String, String)>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.field_type_names = field_type_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_field_types_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_types: Option<Vec<(String, AstType)>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.field_types = field_types;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_names = variant_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_owner_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_owner_name: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_owner_name = variant_owner_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_payload_count = variant_payload_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_type_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_type_name: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_payload_type_name = variant_payload_type_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_type_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_type: Option<AstType>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_payload_type = variant_payload_type;
        }
    }
}
