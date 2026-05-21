impl TypeChecker {
    pub(in crate::typechecker) fn collect_resolver_struct_fields(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        ast_fields: &[StructField],
    ) {
        let Some((symbol, field_types)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Type, name, |symbol| {
                Self::resolver_struct_field_metadata(symbol)
            })
        else {
            self.structs.remove(name);
            return;
        };

        let (fields, field_defaults) =
            Self::resolver_struct_fields_from_metadata(field_types, ast_fields);
        self.structs.insert(
            name.to_string(),
            struct_info_from_resolver_fields(name.to_string(), symbol, fields, field_defaults),
        );
    }

    pub(in crate::typechecker) fn resolver_struct_field_metadata(
        symbol: &Symbol,
    ) -> Option<&[(String, AstType)]> {
        symbol.field_types.as_deref()
    }

    pub(in crate::typechecker) fn resolver_struct_fields_from_metadata(
        fields: &[(String, AstType)],
        ast_fields: &[StructField],
    ) -> (Vec<(String, AstType)>, HashMap<String, Expression>) {
        let field_defaults = ast_fields
            .iter()
            .zip(fields.iter())
            .filter_map(|(field, (restored_name, _))| {
                field
                    .default
                    .as_ref()
                    .map(|default| (restored_name.clone(), default.clone()))
            })
            .collect();
        (fields.to_vec(), field_defaults)
    }

    pub(in crate::typechecker) fn collect_resolver_enum_variants(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
    ) {
        let Some((symbol, variant_names)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Type, name, |symbol| {
                Self::resolver_enum_variant_name_metadata(symbol)
            })
        else {
            self.enums.remove(name);
            return;
        };

        let variants = Self::resolver_enum_variants_from_metadata(symbols, name, variant_names);
        self.enums.insert(
            name.to_string(),
            enum_info_from_resolver_variants(name.to_string(), symbol, variants),
        );
    }

    pub(in crate::typechecker) fn resolver_enum_variant_name_metadata(
        symbol: &Symbol,
    ) -> Option<&[String]> {
        symbol.variant_names.as_deref()
    }

    pub(in crate::typechecker) fn resolver_enum_variants_from_metadata(
        symbols: &SymbolTable,
        enum_name: &str,
        variant_names: &[String],
    ) -> Vec<(String, Option<AstType>)> {
        variant_names
            .iter()
            .map(|variant_name| {
                (
                    variant_name.clone(),
                    symbols
                        .lookup_variant(enum_name, variant_name)
                        .and_then(|variant| variant.variant_payload_type.clone()),
                )
            })
            .collect()
    }
}
