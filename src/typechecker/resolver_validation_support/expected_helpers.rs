fn expected_return_metadata(return_type: &Option<AstType>) -> ExpectedReturnMetadata {
    ExpectedReturnMetadata::new(return_type)
}

fn expected_parameter_metadata(params: &[Param]) -> Vec<ExpectedParameter> {
    let mut expected = Vec::new();
    for param in params {
        expected.push(ExpectedParameter::new(&param.name, &param.ty));
    }
    expected
}

fn expected_value_signature_metadata(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[ast::TypeParam],
) -> ExpectedValueSignature {
    ExpectedValueSignature::new(params, return_type, type_params)
}

fn expected_value_symbol(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[ast::TypeParam],
    is_public: bool,
) -> ExpectedValueSymbol {
    ExpectedValueSymbol::new(params, return_type, type_params, is_public)
}

fn expected_type_parameter_metadata(type_params: &[ast::TypeParam]) -> Vec<ExpectedTypeParameter> {
    let mut expected = Vec::new();
    for type_param in type_params {
        expected.push(ExpectedTypeParameter::new(type_param));
    }
    expected
}

fn expected_behavior_symbol(
    type_params: &[ast::TypeParam],
    methods: &[ast::BehaviorMethod],
    is_public: bool,
) -> ExpectedBehaviorSymbol {
    ExpectedBehaviorSymbol::new(type_params, methods, is_public)
}

fn expected_struct_symbol(
    type_params: &[ast::TypeParam],
    fields: &[StructField],
    is_public: bool,
) -> ExpectedStructSymbol {
    ExpectedStructSymbol::new(type_params, fields, is_public)
}

fn expected_enum_symbol(
    type_params: &[ast::TypeParam],
    variants: &[EnumVariant],
    is_public: bool,
) -> ExpectedEnumSymbol {
    ExpectedEnumSymbol::new(type_params, variants, is_public)
}

fn expected_variant_symbol(
    owner_name: &str,
    is_public: bool,
    payload: &Option<AstType>,
) -> ExpectedVariantSymbol {
    ExpectedVariantSymbol::new(owner_name, is_public, payload)
}

fn expected_import_symbol(source: &str) -> ExpectedImportSymbol {
    ExpectedImportSymbol::new(source)
}

fn expected_module_symbol(name: &str) -> ExpectedModuleSymbol {
    ExpectedModuleSymbol::new(name)
}

fn expected_local_symbol(is_mutable: bool, scope_id: u32) -> ExpectedLocalSymbol {
    ExpectedLocalSymbol::new(is_mutable, scope_id)
}

fn expected_field_metadata(fields: &[StructField]) -> Vec<ExpectedField> {
    let mut expected = Vec::new();
    for field in fields {
        expected.push(ExpectedField::new(&field.name, &field.ty));
    }
    expected
}

fn expected_variant_name_metadata(variants: &[EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

fn expected_behavior_edge(behavior: &str, type_args: &[AstType]) -> ExpectedBehaviorEdge {
    ExpectedBehaviorEdge::new(behavior, type_args)
}

fn push_expected_behavior_impl_edge(
    expected: &mut ExpectedBehaviorAssociations,
    type_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
) {
    expected.impls.push(type_name, behavior, behavior_type_args);
}

fn push_expected_behavior_required_edge(
    expected: &mut ExpectedBehaviorAssociations,
    type_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
) {
    expected
        .required
        .push(type_name, behavior, behavior_type_args);
}

fn push_expected_behavior_parent_edge(
    expected: &mut ExpectedBehaviorEdges,
    behavior: &str,
    parent: &str,
    parent_type_args: &[AstType],
) {
    expected.push(behavior, parent, parent_type_args);
}
