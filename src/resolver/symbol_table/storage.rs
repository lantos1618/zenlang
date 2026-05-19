impl SymbolTable {
    fn define_in_scope(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
        metadata: SymbolMetadata,
        scope_id: u32,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let scoped_key = (namespace, name.to_string(), scope_id);
        let duplicate = if namespace == Namespace::Variant {
            self.symbols.iter().any(|symbol| {
                symbol.namespace == Namespace::Variant
                    && symbol.name == name
                    && symbol.variant_owner_name == metadata.variant_owner_name
            })
        } else {
            self.by_scoped_name.contains_key(&scoped_key)
        };
        if duplicate {
            return Err(Box::new(Diagnostic::error(
                "E0200",
                format!(
                    "duplicate {} symbol '{}'",
                    namespace.diagnostic_name(),
                    name
                ),
                definition_span,
            )));
        }

        let id = SymbolId(self.symbols.len() as u32);
        if namespace != Namespace::Local {
            self.by_name.insert((namespace, name.to_string()), id);
        }
        self.symbols.push(Symbol {
            id,
            namespace,
            name: name.to_string(),
            is_public,
            import_source: metadata.import_source,
            parameter_count: metadata.parameter_count,
            parameter_names: metadata.parameter_names,
            parameter_types: metadata.parameter_types,
            parameter_type_names: metadata.parameter_type_names,
            return_type: metadata.return_type,
            return_type_name: metadata.return_type_name,
            type_parameter_count: metadata.type_parameter_count,
            type_parameter_names: metadata.type_parameter_names,
            type_parameter_bounds: metadata.type_parameter_bounds,
            type_parameter_bound_refs: metadata.type_parameter_bound_refs,
            field_count: metadata.field_count,
            field_types: metadata.field_types,
            field_type_names: metadata.field_type_names,
            variant_names: metadata.variant_names,
            variant_owner_name: metadata.variant_owner_name,
            variant_payload_count: metadata.variant_payload_count,
            variant_payload_type: metadata.variant_payload_type,
            variant_payload_type_name: metadata.variant_payload_type_name,
            behavior_method_signatures: metadata.behavior_method_signatures,
            behavior_method_types: metadata.behavior_method_types,
            behavior_parent_names: metadata.behavior_parent_names,
            behavior_parent_refs: metadata.behavior_parent_refs,
            behavior_impl_names: metadata.behavior_impl_names,
            behavior_impl_refs: metadata.behavior_impl_refs,
            behavior_required_names: metadata.behavior_required_names,
            behavior_required_refs: metadata.behavior_required_refs,
            is_mutable: metadata.is_mutable,
            scope_id,
            definition_span,
        });
        self.by_scoped_name.insert(scoped_key, id);
        Ok(id)
    }

    pub(super) fn define_local(
        &mut self,
        name: &str,
        mutable: bool,
        scope_id: u32,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        self.define_in_scope(
            Namespace::Local,
            name,
            false,
            SymbolMetadata {
                is_mutable: Some(mutable),
                ..SymbolMetadata::default()
            },
            scope_id,
            definition_span,
        )
    }

    pub(super) fn new_scope(&mut self) -> u32 {
        self.next_scope_id += 1;
        self.next_scope_id
    }
}
