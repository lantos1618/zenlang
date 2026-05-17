use super::*;

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

    pub(in crate::typechecker) fn collect_resolver_behavior_methods(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
    ) {
        let Some((symbol, method_types)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Behavior, name, |symbol| {
                Self::resolver_behavior_method_metadata(symbol)
            })
        else {
            self.behaviors.remove(name);
            return;
        };

        let Some(existing) = self.behaviors.get(name).cloned() else {
            return;
        };
        let methods = Self::resolver_behavior_methods_from_metadata(
            existing.methods,
            method_types,
            symbol.definition_span,
        );
        self.behaviors.insert(
            name.to_string(),
            behavior_info_from_resolver_methods(name.to_string(), symbol, methods),
        );
    }

    pub(in crate::typechecker) fn resolver_behavior_method_metadata(
        symbol: &Symbol,
    ) -> Option<&[BehaviorMethodTypeMetadata]> {
        symbol.behavior_method_types.as_deref()
    }

    pub(in crate::typechecker) fn resolver_behavior_methods_from_metadata(
        existing_methods: Vec<ast::BehaviorMethod>,
        method_types: &[BehaviorMethodTypeMetadata],
        span: Span,
    ) -> Vec<ast::BehaviorMethod> {
        let mut existing_methods: VecDeque<ast::BehaviorMethod> =
            existing_methods.into_iter().collect();
        let mut methods = Vec::new();
        for (metadata_index, metadata) in method_types.iter().cloned().enumerate() {
            let future_method_names = method_types[metadata_index + 1..]
                .iter()
                .map(|metadata| metadata.name.as_str());
            let method = Self::named_queue_index_preserving_future_front(
                &existing_methods,
                &metadata.name,
                future_method_names,
                |method| method.name.as_str(),
            )
            .and_then(|index| existing_methods.remove(index));
            methods.push(Self::resolver_behavior_method_from_metadata(
                method.as_ref(),
                metadata,
                span,
            ));
        }
        methods
    }

    fn resolver_behavior_method_from_metadata(
        existing_method: Option<&ast::BehaviorMethod>,
        metadata: BehaviorMethodTypeMetadata,
        span: Span,
    ) -> ast::BehaviorMethod {
        let params = Self::resolver_params_from_metadata(
            existing_method
                .map(|method| method.params.as_slice())
                .unwrap_or(&[]),
            &metadata.parameter_names,
            &metadata.parameter_types,
            Span::dummy(),
        );
        let return_type = Self::resolver_optional_return_type(&metadata.return_type);
        ast::BehaviorMethod {
            name: metadata.name,
            params,
            return_type,
            default_body: existing_method.and_then(|method| method.default_body.clone()),
            span: existing_method.map(|method| method.span).unwrap_or(span),
        }
    }

    pub(in crate::typechecker) fn resolver_params_from_metadata(
        existing_params: &[Param],
        parameter_names: &[String],
        parameter_types: &[AstType],
        default_span: Span,
    ) -> Vec<Param> {
        parameter_types
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, ty)| match existing_params.get(index).cloned() {
                Some(mut param) => {
                    if let Some(name) = parameter_names.get(index) {
                        param.name = name.clone();
                    }
                    param.ty = ty;
                    param
                }
                None => Param {
                    name: parameter_names.get(index).cloned().unwrap_or_default(),
                    ty,
                    mutable: false,
                    span: default_span,
                },
            })
            .collect()
    }

    pub(in crate::typechecker) fn resolver_optional_return_type(
        return_type: &AstType,
    ) -> Option<AstType> {
        match return_type {
            AstType::Void => None,
            ty => Some(ty.clone()),
        }
    }
}
