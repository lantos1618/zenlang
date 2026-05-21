struct ResolverStructEntry<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    fields: &'a [StructField],
    public: bool,
    span: Span,
}

struct ResolverEnumEntry<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    variants: &'a [EnumVariant],
    public: bool,
    span: Span,
}

struct ResolverBehaviorEntry<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    methods: &'a [ast::BehaviorMethod],
    public: bool,
    span: Span,
}

impl TypeChecker {
    fn validate_resolver_type_declaration_entry(
        &mut self,
        symbols: &SymbolTable,
        decl: &Declaration,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        match decl {
            Declaration::Struct {
                name,
                type_params,
                fields,
                public,
                span,
                ..
            } => self.validate_resolver_struct_entry(
                symbols,
                ResolverStructEntry {
                    name,
                    type_params,
                    fields,
                    public: *public,
                    span: *span,
                },
                scope_cursor,
            ),
            Declaration::Enum {
                name,
                type_params,
                variants,
                public,
                span,
                ..
            } => self.validate_resolver_enum_entry(
                symbols,
                ResolverEnumEntry {
                    name,
                    type_params,
                    variants,
                    public: *public,
                    span: *span,
                },
            ),
            Declaration::Behavior {
                name,
                type_params,
                methods,
                public,
                span,
                ..
            } => self.validate_resolver_behavior_entry(
                symbols,
                ResolverBehaviorEntry {
                    name,
                    type_params,
                    methods,
                    public: *public,
                    span: *span,
                },
                scope_cursor,
            ),
            _ => {}
        }
    }

    fn validate_resolver_struct_entry(
        &mut self,
        symbols: &SymbolTable,
        entry: ResolverStructEntry<'_>,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        if self
            .require_resolver_struct_symbol(
                symbols,
                entry.name,
                expected_struct_symbol(entry.type_params, entry.fields, entry.public),
                entry.span,
            )
            .is_none()
        {
            return;
        };

        for field in entry.fields {
            if let Some(default) = &field.default {
                self.require_resolver_scoped_expr_locals(symbols, default, scope_cursor);
            }
        }
    }

    fn validate_resolver_enum_entry(&mut self, symbols: &SymbolTable, entry: ResolverEnumEntry<'_>) {
        self.require_resolver_enum_symbol(
            symbols,
            entry.name,
            expected_enum_symbol(entry.type_params, entry.variants, entry.public),
            entry.span,
        );
        for variant in entry.variants {
            self.require_resolver_variant_symbol(
                symbols,
                &variant.name,
                expected_variant_symbol(entry.name, entry.public, &variant.payload),
                variant.span,
            );
        }
    }

    fn validate_resolver_behavior_entry(
        &mut self,
        symbols: &SymbolTable,
        entry: ResolverBehaviorEntry<'_>,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        if self
            .require_resolver_behavior_symbol(
                symbols,
                entry.name,
                expected_behavior_symbol(entry.type_params, entry.methods, entry.public),
                entry.span,
            )
            .is_none()
        {
            return;
        };

        for method in entry.methods {
            if let Some(default_body) = &method.default_body {
                self.require_resolver_callable_locals(
                    symbols,
                    &method.params,
                    default_body,
                    scope_cursor,
                );
            }
        }
    }
}
