use super::*;

#[test]
fn callable_signature_insert_routes_function_and_method_keys() {
    let mut tc = TypeChecker::new();
    let function = FuncInfo {
        name: "make".to_string(),
        params: vec![],
        return_type: AstType::I32,
        type_params: vec![],
        type_param_bounds: HashMap::new(),
    };
    let method = FuncInfo {
        name: "Point.get".to_string(),
        params: vec![("self".to_string(), AstType::Named("Point".to_string()))],
        return_type: AstType::I32,
        type_params: vec![],
        type_param_bounds: HashMap::new(),
    };

    tc.insert_callable_signature("make", function);
    tc.insert_callable_signature("Point.get", method);

    assert!(tc.functions.contains_key("make"));
    assert!(!tc.methods.contains_key("make"));
    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.functions.contains_key("Point.get"));
}

#[test]
fn generic_callable_template_mut_routes_function_and_method_keys() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T { value }

Box.get<T> = (self: Box<T>) T { self.value }
"#,
    );
    let ast::Declaration::Function {
        type_params: function_type_params,
        params: function_params,
        return_type: function_return_type,
        body: function_body,
        span: function_span,
        ..
    } = &program.declarations[1]
    else {
        panic!("expected generic function");
    };
    let ast::Declaration::Method {
        type_params: method_type_params,
        params: method_params,
        return_type: method_return_type,
        body: method_body,
        span: method_span,
        ..
    } = &program.declarations[2]
    else {
        panic!("expected generic method");
    };
    let mut tc = TypeChecker::new();
    tc.generic_functions.insert(
        "identity".to_string(),
        generic_template_from_type_params(
            function_type_params,
            function_params,
            function_return_type,
            function_body,
            *function_span,
        )
        .expect("generic function template"),
    );
    tc.generic_methods.insert(
        "Box.get".to_string(),
        generic_template_from_type_params(
            method_type_params,
            method_params,
            method_return_type,
            method_body,
            *method_span,
        )
        .expect("generic method template"),
    );

    tc.generic_callable_template_mut("identity")
        .expect("function template")
        .return_type = Some(AstType::I32);
    tc.generic_callable_template_mut("Box.get")
        .expect("method template")
        .return_type = Some(AstType::Bool);

    assert_eq!(
        tc.generic_functions
            .get("identity")
            .and_then(|template| template.return_type.as_ref()),
        Some(&AstType::I32)
    );
    assert_eq!(
        tc.generic_methods
            .get("Box.get")
            .and_then(|template| template.return_type.as_ref()),
        Some(&AstType::Bool)
    );
    assert!(!tc.generic_methods.contains_key("identity"));
    assert!(!tc.generic_functions.contains_key("Box.get"));
}

#[test]
fn callable_template_rekey_routes_function_and_method_keys() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T { value }

Box.get<T> = (self: Box<T>) T { self.value }
"#,
    );
    let ast::Declaration::Function {
        type_params: function_type_params,
        params: function_params,
        return_type: function_return_type,
        body: function_body,
        span: function_span,
        ..
    } = &program.declarations[1]
    else {
        panic!("expected generic function");
    };
    let ast::Declaration::Method {
        type_params: method_type_params,
        params: method_params,
        return_type: method_return_type,
        body: method_body,
        span: method_span,
        ..
    } = &program.declarations[2]
    else {
        panic!("expected generic method");
    };
    let mut tc = TypeChecker::new();
    tc.generic_functions.insert(
        "identity".to_string(),
        generic_template_from_type_params(
            function_type_params,
            function_params,
            function_return_type,
            function_body,
            *function_span,
        )
        .expect("generic function template"),
    );
    tc.generic_methods.insert(
        "Box.get".to_string(),
        generic_template_from_type_params(
            method_type_params,
            method_params,
            method_return_type,
            method_body,
            *method_span,
        )
        .expect("generic method template"),
    );

    tc.rekey_callable_template("identity", "renamed");
    tc.rekey_callable_template("Box.get", "Box.fetch");

    assert!(tc.generic_functions.contains_key("renamed"));
    assert!(!tc.generic_functions.contains_key("identity"));
    assert!(tc.generic_methods.contains_key("Box.fetch"));
    assert!(!tc.generic_methods.contains_key("Box.get"));
}

#[test]
fn resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

identity<T> = (mut value: T) T { value }

Box.get<T> = (self: Box<T>, mut fallback: T) T { fallback }
"#,
    );
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });

    let function_template = tc
        .generic_functions
        .get("identity")
        .expect("function template stub");
    assert!(
            function_template.type_params.is_empty(),
            "resolver-backed generic function templates should not keep AST generic names before resolver metadata"
        );
    assert_eq!(function_template.params.len(), 1);
    assert_eq!(function_template.params[0].name, "");
    assert_eq!(function_template.params[0].ty, AstType::Void);
    assert!(function_template.params[0].mutable);
    assert_eq!(function_template.return_type, None);

    let method_template = tc
        .generic_methods
        .get("Box.get")
        .expect("method template stub");
    assert!(
            method_template.type_params.is_empty(),
            "resolver-backed generic method templates should not keep AST generic names before resolver metadata"
        );
    assert_eq!(method_template.params.len(), 2);
    assert_eq!(method_template.params[1].name, "");
    assert_eq!(method_template.params[1].ty, AstType::Void);
    assert!(method_template.params[1].mutable);
    assert_eq!(method_template.return_type, None);
}

#[test]
fn behavior_ref_validation_maps_role_and_check_diagnostics() {
    let cases = [
        (
            BehaviorRefRole::Parent,
            BehaviorRefCheck::Contains,
            ("behavior", "parents", "parent refs", "E0235", "E0245"),
        ),
        (
            BehaviorRefRole::Parent,
            BehaviorRefCheck::List,
            ("behavior", "parents", "parent refs", "E0240", "E0246"),
        ),
        (
            BehaviorRefRole::Impl,
            BehaviorRefCheck::Contains,
            (
                "type",
                "behavior impls",
                "behavior impl refs",
                "E0236",
                "E0247",
            ),
        ),
        (
            BehaviorRefRole::Impl,
            BehaviorRefCheck::List,
            (
                "type",
                "behavior impls",
                "behavior impl refs",
                "E0238",
                "E0248",
            ),
        ),
        (
            BehaviorRefRole::Required,
            BehaviorRefCheck::Contains,
            (
                "type",
                "behavior requires",
                "behavior requires refs",
                "E0237",
                "E0249",
            ),
        ),
        (
            BehaviorRefRole::Required,
            BehaviorRefCheck::List,
            (
                "type",
                "behavior requires",
                "behavior requires refs",
                "E0239",
                "E0250",
            ),
        ),
    ];

    for (role, check, expected) in cases {
        let validation = BehaviorRefValidation::for_role(role, check);
        assert_eq!(
            (
                validation.symbol_kind,
                validation.name_label,
                validation.ref_label,
                validation.name_code,
                validation.ref_code,
            ),
            expected
        );
    }

    let contains =
        BehaviorRefValidation::for_role(BehaviorRefRole::Impl, BehaviorRefCheck::Contains);
    assert_eq!(
            contains.contains_name_message("Point", "PrettyJson", "Json<str>"),
            "resolver type symbol 'Point' has behavior impls 'PrettyJson', expected to include 'Json<str>'"
        );
    assert_eq!(
            contains.contains_ref_message("Point", "PrettyJson", "Json<str>"),
            "resolver type symbol 'Point' has behavior impl refs 'PrettyJson', expected to include 'Json<str>'"
        );

    let list = BehaviorRefValidation::for_role(BehaviorRefRole::Parent, BehaviorRefCheck::List);
    assert_eq!(
        list.list_name_message("PrettyJson", "Json, Debug", "Json"),
        "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'"
    );
    assert_eq!(
        list.list_ref_message("PrettyJson", "Json, Debug", "Json"),
        "resolver behavior symbol 'PrettyJson' has parent refs 'Json, Debug', expected 'Json'"
    );
}

#[test]
fn behavior_ref_validation_separates_role_labels_from_check_codes() {
    let parent = BehaviorRefValidation::role_labels(BehaviorRefRole::Parent);
    let implementation = BehaviorRefValidation::role_labels(BehaviorRefRole::Impl);
    let required = BehaviorRefValidation::role_labels(BehaviorRefRole::Required);
    let parent_contains =
        BehaviorRefValidation::codes_for(BehaviorRefRole::Parent, BehaviorRefCheck::Contains);
    let parent_list =
        BehaviorRefValidation::codes_for(BehaviorRefRole::Parent, BehaviorRefCheck::List);

    assert_eq!(parent, ("behavior", "parents", "parent refs"));
    assert_eq!(
        implementation,
        ("type", "behavior impls", "behavior impl refs")
    );
    assert_eq!(
        required,
        ("type", "behavior requires", "behavior requires refs")
    );
    assert_eq!(parent_contains, ("E0235", "E0245"));
    assert_eq!(parent_list, ("E0240", "E0246"));
}

#[test]
fn behavior_ref_actual_selects_role_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let behavior = symbols
        .lookup(Namespace::Behavior, "PrettyJson")
        .expect("behavior symbol");
    let ty = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");

    let parent = BehaviorRefActual::for_role(behavior, BehaviorRefRole::Parent);
    assert_eq!(format_behavior_ref_names(parent.names), "Json<str>");
    assert_eq!(format_behavior_refs(parent.refs), "Json<str>");

    let implementation = BehaviorRefActual::for_role(ty, BehaviorRefRole::Impl);
    assert_eq!(
        format_behavior_ref_names(implementation.names),
        "PrettyJson"
    );
    assert_eq!(format_behavior_refs(implementation.refs), "PrettyJson");

    let required = BehaviorRefActual::for_role(ty, BehaviorRefRole::Required);
    assert_eq!(format_behavior_ref_names(required.names), "Json<str>");
    assert_eq!(format_behavior_refs(required.refs), "Json<str>");
}

#[test]
fn behavior_ref_actual_exposes_role_metadata_selection() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let behavior = symbols
        .lookup(Namespace::Behavior, "PrettyJson")
        .expect("behavior symbol");
    let ty = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");

    let (parent_names, parent_refs) =
        BehaviorRefActual::metadata_for_role(behavior, BehaviorRefRole::Parent);
    let (impl_names, impl_refs) = BehaviorRefActual::metadata_for_role(ty, BehaviorRefRole::Impl);
    let (required_names, required_refs) =
        BehaviorRefActual::metadata_for_role(ty, BehaviorRefRole::Required);

    assert_eq!(format_behavior_ref_names(parent_names), "Json<str>");
    assert_eq!(format_behavior_refs(parent_refs), "Json<str>");
    assert_eq!(format_behavior_ref_names(impl_names), "PrettyJson");
    assert_eq!(format_behavior_refs(impl_refs), "PrettyJson");
    assert_eq!(format_behavior_ref_names(required_names), "Json<str>");
    assert_eq!(format_behavior_refs(required_refs), "Json<str>");
}

#[test]
fn behavior_ref_actual_matches_expected_edges() {
    let names = vec!["Json<i32>".to_string()];
    let refs = vec![BehaviorRefMetadata {
        name: "Json".to_string(),
        type_args: vec![AstType::I32],
    }];
    let actual = BehaviorRefActual {
        names: Some(&names),
        refs: Some(&refs),
    };
    let expected = expected_behavior_edge("Json", &[AstType::I32]);
    let mismatch = expected_behavior_edge("Debug", &[]);
    let expected_list = ExpectedBehaviorEdgeMetadata::from_edges(std::slice::from_ref(&expected));

    assert!(actual.contains_display(&expected.display));
    assert!(actual.contains_metadata(&expected.metadata));
    assert!(!actual.contains_display(&mismatch.display));
    assert!(!actual.contains_metadata(&mismatch.metadata));
    assert!(actual.names_match(&expected_list.names));
    assert!(actual.refs_match(&expected_list.refs));
}

#[test]
fn expected_parameter_builds_name_display_and_type_together() {
    let parameter = ExpectedParameter::new(
        "mapper",
        &AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        },
    );

    assert_eq!(parameter.name, "mapper");
    assert_eq!(parameter.display, "(i32) str");
    assert_eq!(
        parameter.typed,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        }
    );
}

#[test]
fn expected_return_metadata_defaults_and_displays_together() {
    let explicit = ExpectedReturnMetadata::new(&Some(AstType::Named("Point".to_string())));
    let implicit = ExpectedReturnMetadata::new(&None);

    assert_eq!(explicit.display, "Point");
    assert_eq!(explicit.typed, AstType::Named("Point".to_string()));
    assert_eq!(implicit.display, "void");
    assert_eq!(implicit.typed, AstType::Void);
}

#[test]
fn expected_type_parameter_builds_bound_display_and_ref_together() {
    let type_param = ast::TypeParam {
        name: "T".to_string(),
        constraint: Some("Json".to_string()),
        constraint_type_args: vec![AstType::Named("T".to_string())],
        span: Span::dummy(),
    };

    let expected = ExpectedTypeParameter::new(&type_param);
    let bound = expected.bound.expect("expected bound");

    assert_eq!(expected.name, "T");
    assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
    assert_eq!(bound.reference.type_parameter, "T");
    assert_eq!(bound.reference.behavior, "Json");
    assert_eq!(
        bound.reference.type_args,
        vec![AstType::Named("T".to_string())]
    );
}

#[test]
fn expected_field_builds_display_and_type_together() {
    let field = ExpectedField::new(
        "mapper",
        &AstType::Function {
            params: vec![AstType::Named("Input".to_string())],
            ret: Box::new(AstType::Named("Output".to_string())),
        },
    );

    assert_eq!(
        field.display,
        ("mapper".to_string(), "(Input) Output".to_string())
    );
    assert_eq!(
        field.typed,
        (
            "mapper".to_string(),
            AstType::Function {
                params: vec![AstType::Named("Input".to_string())],
                ret: Box::new(AstType::Named("Output".to_string())),
            }
        )
    );
}

#[test]
fn expected_variant_payload_builds_display_and_type_together() {
    let payload = ExpectedVariantPayloadType::new(&Some(AstType::Function {
        params: vec![AstType::I32],
        ret: Box::new(AstType::Bool),
    }));
    let empty_payload = ExpectedVariantPayloadType::new(&None);

    assert_eq!(payload.display, Some("(i32) bool".to_string()));
    assert_eq!(
        payload.typed,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Bool),
        })
    );
    assert_eq!(empty_payload.display, None);
    assert_eq!(empty_payload.typed, None);
}

#[test]
fn expected_behavior_method_builds_signature_and_metadata_together() {
    let method = ast::BehaviorMethod {
        name: "map".to_string(),
        params: vec![Param {
            name: "mapper".to_string(),
            ty: AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Str),
            },
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Str),
        default_body: None,
        span: Span::dummy(),
    };

    let expected = ExpectedBehaviorMethod::new(&method);

    assert_eq!(
        expected.signature,
        (
            "map".to_string(),
            vec!["(i32) str".to_string()],
            "str".to_string(),
        )
    );
    assert_eq!(expected.metadata.name, "map");
    assert_eq!(
        expected.metadata.parameter_names,
        vec!["mapper".to_string()]
    );
    assert_eq!(
        expected.metadata.parameter_types,
        vec![AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        }]
    );
    assert_eq!(expected.metadata.return_type, AstType::Str);
}

#[test]
fn expected_value_signature_builds_components_together() {
    let params = vec![Param {
        name: "value".to_string(),
        ty: AstType::Named("T".to_string()),
        mutable: false,
        span: Span::dummy(),
    }];
    let return_type = Some(AstType::Named("T".to_string()));
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: Some("Json".to_string()),
        constraint_type_args: vec![AstType::Named("T".to_string())],
        span: Span::dummy(),
    }];

    let signature = ExpectedValueSignature::new(&params, &return_type, &type_params);

    assert_eq!(signature.params[0].name, "value");
    assert_eq!(signature.params[0].display, "T");
    assert_eq!(signature.params[0].typed, AstType::Named("T".to_string()));
    assert_eq!(signature.return_type.display, "T");
    assert_eq!(signature.return_type.typed, AstType::Named("T".to_string()));
    assert_eq!(signature.type_params[0].name, "T");
    let bound = signature.type_params[0]
        .bound
        .as_ref()
        .expect("expected bound");
    assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
    assert_eq!(bound.reference.behavior, "Json");
    assert_eq!(
        bound.reference.type_args,
        vec![AstType::Named("T".to_string())]
    );
}

#[test]
fn expected_value_symbol_builds_signature_and_visibility_together() {
    let params = vec![Param {
        name: "value".to_string(),
        ty: AstType::I32,
        mutable: false,
        span: Span::dummy(),
    }];
    let return_type = Some(AstType::Bool);

    let symbol = ExpectedValueSymbol::new(&params, &return_type, &[], true);

    assert!(symbol.is_public);
    assert_eq!(symbol.signature.params[0].name, "value");
    assert_eq!(symbol.signature.params[0].display, "i32");
    assert_eq!(symbol.signature.params[0].typed, AstType::I32);
    assert_eq!(symbol.signature.return_type.display, "bool");
    assert_eq!(symbol.signature.return_type.typed, AstType::Bool);
    assert!(symbol.signature.type_params.is_empty());
}

#[test]
fn expected_type_like_symbol_builds_type_params_and_visibility_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: Some("Json".to_string()),
        constraint_type_args: vec![AstType::Named("T".to_string())],
        span: Span::dummy(),
    }];

    let symbol = ExpectedTypeLikeSymbol::new(&type_params, Some(true));

    assert_eq!(symbol.is_public, Some(true));
    assert_eq!(symbol.type_params[0].name, "T");
    let bound = symbol.type_params[0]
        .bound
        .as_ref()
        .expect("expected bound");
    assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
    assert_eq!(bound.reference.type_parameter, "T");
    assert_eq!(bound.reference.behavior, "Json");
    assert_eq!(
        bound.reference.type_args,
        vec![AstType::Named("T".to_string())]
    );
}

#[test]
fn expected_behavior_symbol_builds_type_like_and_methods_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: None,
        constraint_type_args: vec![],
        span: Span::dummy(),
    }];
    let methods = vec![ast::BehaviorMethod {
        name: "encode".to_string(),
        params: vec![Param {
            name: "value".to_string(),
            ty: AstType::Named("Self".to_string()),
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Named("T".to_string())),
        default_body: None,
        span: Span::dummy(),
    }];

    let symbol = ExpectedBehaviorSymbol::new(&type_params, &methods, true);

    assert_eq!(symbol.type_like.is_public, Some(true));
    assert_eq!(symbol.type_like.type_params[0].name, "T");
    assert_eq!(symbol.methods[0].signature.0, "encode");
    assert_eq!(symbol.methods[0].signature.1, vec!["Self".to_string()]);
    assert_eq!(symbol.methods[0].signature.2, "T");
    assert_eq!(symbol.methods[0].metadata.name, "encode");
    assert_eq!(
        symbol.methods[0].metadata.parameter_names,
        vec!["value".to_string()]
    );
    assert_eq!(
        symbol.methods[0].metadata.return_type,
        AstType::Named("T".to_string())
    );
}

#[test]
fn expected_struct_symbol_builds_type_like_and_fields_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: None,
        constraint_type_args: vec![],
        span: Span::dummy(),
    }];
    let fields = vec![StructField {
        name: "value".to_string(),
        ty: AstType::Named("T".to_string()),
        default: None,
        mutable: false,
        span: Span::dummy(),
    }];

    let symbol = ExpectedStructSymbol::new(&type_params, &fields, true);

    assert_eq!(symbol.type_like.is_public, Some(true));
    assert_eq!(symbol.type_like.type_params[0].name, "T");
    assert_eq!(
        symbol.fields[0].display,
        ("value".to_string(), "T".to_string())
    );
    assert_eq!(
        symbol.fields[0].typed,
        ("value".to_string(), AstType::Named("T".to_string()))
    );
}

#[test]
fn expected_enum_symbol_builds_type_like_and_variants_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: None,
        constraint_type_args: vec![],
        span: Span::dummy(),
    }];
    let variants = vec![
        EnumVariant {
            name: "Some".to_string(),
            payload: Some(AstType::Named("T".to_string())),
            span: Span::dummy(),
        },
        EnumVariant {
            name: "None".to_string(),
            payload: None,
            span: Span::dummy(),
        },
    ];

    let symbol = ExpectedEnumSymbol::new(&type_params, &variants, true);

    assert_eq!(symbol.type_like.is_public, Some(true));
    assert_eq!(symbol.type_like.type_params[0].name, "T");
    assert_eq!(
        symbol.variant_names,
        vec!["Some".to_string(), "None".to_string()]
    );
}

#[test]
fn expected_variant_symbol_builds_owner_visibility_and_payload_together() {
    let payload = Some(AstType::Function {
        params: vec![AstType::I32],
        ret: Box::new(AstType::Bool),
    });

    let symbol = ExpectedVariantSymbol::new("Result", true, &payload);

    assert_eq!(symbol.owner_name, "Result");
    assert!(symbol.is_public);
    assert_eq!(symbol.payload.display, Some("(i32) bool".to_string()));
    assert_eq!(
        symbol.payload.typed,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Bool),
        })
    );
}

#[test]
fn expected_import_symbol_builds_source_and_visibility_together() {
    let symbol = ExpectedImportSymbol::new("std.io");

    assert_eq!(symbol.source, "std.io");
    assert!(!symbol.is_public);
}

#[test]
fn expected_module_symbol_builds_name_source_and_visibility_together() {
    let symbol = ExpectedModuleSymbol::new("std.io");

    assert_eq!(symbol.name, "std.io");
    assert_eq!(symbol.source, None);
    assert!(!symbol.is_public);
}

#[test]
fn expected_local_symbol_builds_scope_mutability_source_and_visibility_together() {
    let symbol = ExpectedLocalSymbol::new(true, 42);

    assert_eq!(symbol.scope_id, 42);
    assert!(symbol.is_mutable);
    assert_eq!(symbol.source, None);
    assert!(!symbol.is_public);
}

#[test]
fn expected_behavior_edge_builds_display_and_metadata_together() {
    let edge = ExpectedBehaviorEdge::new("Json", &[AstType::I32]);

    assert_eq!(edge.display, "Json<i32>");
    assert_eq!(edge.metadata.name, "Json");
    assert_eq!(edge.metadata.type_args, vec![AstType::I32]);
}

#[test]
fn expected_behavior_associations_build_impl_and_required_edges_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );

    let expected = ExpectedBehaviorAssociations::new(&program);
    let impl_edge = &expected.impls.edges_for("Point")[0];
    let required_edge = &expected.required.edges_for("Point")[0];

    assert_eq!(impl_edge.display, "Json<str>");
    assert_eq!(impl_edge.metadata.name, "Json");
    assert_eq!(impl_edge.metadata.type_args, vec![AstType::Str]);
    assert_eq!(required_edge.display, "Json<str>");
    assert_eq!(required_edge.metadata.name, "Json");
    assert_eq!(required_edge.metadata.type_args, vec![AstType::Str]);
}

#[test]
fn resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks = TypeChecker::collect_resolver_behavior_association_list_tasks(&program, &symbols);

    assert_eq!(tasks.type_associations.len(), 1);
    let type_task = &tasks.type_associations[0];
    assert_eq!(type_task.name, "Point");
    assert_eq!(type_task.impl_edges[0].display, "Json<str>");
    assert_eq!(type_task.required_edges[0].display, "Json<str>");

    assert_eq!(tasks.behavior_parents.len(), 2);
    let pretty_task = tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "PrettyJson")
        .expect("PrettyJson parent task");
    assert_eq!(pretty_task.parent_edges[0].display, "Json<str>");
    let json_task = tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "Json")
        .expect("Json empty parent task");
    assert!(json_task.parent_edges.is_empty());
}

#[test]
fn resolver_expected_symbol_sets_collect_declarations_and_locals_together() {
    let program = parse_program(
        r#"
{ io } = std

Point: { x: i32 = 1 }

main = (input: i32) i32 {
    value := input
    value
}
"#,
    );

    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let expected =
        TypeChecker::collect_resolver_validation_replay_tasks(&program, &symbols).expected_symbols;

    assert!(expected.validate_imports);
    assert!(expected
        .declarations
        .contains(&(Namespace::Module, "std".to_string())));
    assert!(expected
        .declarations
        .contains(&(Namespace::Import, "io".to_string())));
    assert!(expected
        .declarations
        .contains(&(Namespace::Type, "Point".to_string())));
    assert!(expected
        .declarations
        .contains(&(Namespace::Value, "main".to_string())));
    assert!(expected.locals.contains(&("input".to_string(), 2)));
    assert!(expected.locals.contains(&("value".to_string(), 3)));
}

#[test]
fn resolver_validation_replay_declaration_tasks_collect_sources_and_edges() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks =
        TypeChecker::collect_resolver_validation_replay_declaration_tasks(&program, &symbols);

    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Type, "Point".to_string())));
    assert_eq!(tasks.type_declarations.len(), 1);
    assert_eq!(tasks.type_declarations[0].name, "Point");
    assert_eq!(
        tasks.expected_associations.impls.owned_edges_for("Point")[0].display,
        "Json<str>"
    );
    assert_eq!(
        tasks
            .expected_associations
            .required
            .owned_edges_for("Point")[0]
            .display,
        "Json<str>"
    );
    assert_eq!(tasks.behavior_declarations.len(), 1);
    assert_eq!(tasks.behavior_declarations[0].name, "Json");
}

#[test]
fn resolver_type_reference_validation_tasks_collect_only_type_reference_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = (input: Point) Point {
    input
}
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_type_reference_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert!(matches!(
        tasks[0],
        ResolverTypeReferenceValidationTask::Struct { name: "Point", .. }
    ));
    assert!(matches!(
        tasks[1],
        ResolverTypeReferenceValidationTask::Function { name: "main", .. }
    ));
}

#[test]
fn resolver_validation_replay_tasks_collect_symbols_and_behavior_associations_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)

main = (input: i32) i32 {
    input
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks = TypeChecker::collect_resolver_validation_replay_tasks(&program, &symbols);

    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Type, "Point".to_string())));
    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Behavior, "Json".to_string())));
    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Value, "main".to_string())));
    assert!(tasks
        .expected_symbols
        .locals
        .iter()
        .any(|(name, _)| name == "input"));

    let type_task = &tasks.behavior_associations.type_associations[0];
    assert_eq!(type_task.name, "Point");
    assert_eq!(type_task.impl_edges[0].display, "Json<str>");
    assert_eq!(type_task.required_edges[0].display, "Json<str>");
    assert_eq!(tasks.behavior_associations.behavior_parents.len(), 1);
}

#[test]
fn resolver_declaration_metadata_tasks_collect_top_level_type_reference_tasks() {
    let program = parse_program(
        r#"
value := 1
"#,
    );

    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);

    assert!(
        tasks.type_references.iter().any(|task| matches!(
            task,
            ResolverTypeReferenceValidationTask::TopLevelExpr { .. }
        )),
        "top-level expression type references should stay in the shared resolver task collector"
    );
}

#[test]
fn behavior_extends_replay_task_helper_pushes_parent_validation() {
    let program = parse_program(
        r#"
Json<T>: behavior {
}
Pretty<T>: behavior {
}

Pretty.extends(Json<T>)
"#,
    );
    let mut tasks = Vec::new();

    let handled =
        TypeChecker::push_behavior_extends_replay_task(&program.declarations[2], &mut tasks);

    assert!(handled);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].behavior, "Pretty");
    assert_eq!(tasks[0].parent, "Json");
    assert_eq!(
        tasks[0].parent_type_args,
        &[AstType::Named("T".to_string())]
    );
}

#[test]
fn behavior_requires_replay_task_helper_pushes_requires_validation() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<str>)
"#,
    );
    let mut tasks = Vec::new();

    let handled =
        TypeChecker::push_behavior_requires_replay_task(&program.declarations[2], &mut tasks);

    assert!(handled);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].type_name, "Point");
    assert_eq!(tasks[0].behavior, "Json");
    assert_eq!(tasks[0].behavior_type_args, &[AstType::Str]);
}

#[test]
fn behavior_association_validation_tasks_collect_extends_impls_and_requires_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) T
}

Pretty.extends(Json<T>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );

    let tasks = TypeChecker::collect_behavior_association_validation_tasks(&program.declarations);

    assert_eq!(tasks.extends.len(), 1);
    assert_eq!(tasks.extends[0].behavior, "Pretty");
    assert_eq!(tasks.extends[0].parent, "Json");
    assert_eq!(
        tasks.extends[0].parent_type_args,
        &[AstType::Named("T".to_string())]
    );
    assert_eq!(tasks.impls.len(), 1);
    assert_eq!(tasks.impls[0].ast_type_name, "Point");
    assert_eq!(tasks.impls[0].behavior, "Json");
    assert_eq!(tasks.impls[0].behavior_type_args, &[AstType::Str]);
    assert_eq!(tasks.requires.len(), 1);
    assert_eq!(tasks.requires[0].type_name, "Point");
    assert_eq!(tasks.requires[0].behavior, "Json");
    assert_eq!(tasks.requires[0].behavior_type_args, &[AstType::Str]);
}

#[test]
fn behavior_association_validation_helper_replays_impls_and_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut checker = TypeChecker::new();
    checker.collect_declarations(&program.declarations);
    let tasks = TypeChecker::collect_behavior_association_validation_tasks(&program.declarations);

    checker.validate_behavior_association_tasks(&tasks, None);

    assert!(
        checker.diagnostics().is_empty(),
        "valid impl+requires replay should not emit diagnostics: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn impl_block_declaration_tasks_collect_behavior_and_plain_impls() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Point.impl = {
    x_value = (value: Point) i32 { value.x }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );

    let tasks = TypeChecker::collect_impl_block_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].type_name, "Point");
    assert_eq!(tasks[0].behavior, None);
    assert_eq!(tasks[1].type_name, "Point");
    assert_eq!(tasks[1].behavior, Some("Json"));
    assert_eq!(tasks[1].methods.len(), 1);
}

#[test]
fn callable_declaration_tasks_collect_functions_and_methods() {
    let program = parse_program(
        r#"
Point: { x: i32 }

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }
"#,
    );

    let tasks = TypeChecker::collect_callable_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    match &tasks[0] {
        CallableDeclarationTask::Function { name, .. } => assert_eq!(*name, "make"),
        _ => panic!("expected function task"),
    }
    match &tasks[1] {
        CallableDeclarationTask::Method {
            type_name,
            method_name,
            ..
        } => {
            assert_eq!(*type_name, "Point");
            assert_eq!(*method_name, "get");
        }
        _ => panic!("expected method task"),
    }
}

#[test]
fn ast_type_declaration_tasks_collect_structs_and_enums() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Option<T>: Some(T), None
"#,
    );

    let tasks = TypeChecker::collect_ast_type_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    match &tasks[0] {
        AstTypeDeclarationTask::Struct { name, fields, .. } => {
            assert_eq!(*name, "Point");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("expected struct task"),
    }
    match &tasks[1] {
        AstTypeDeclarationTask::Enum {
            name,
            type_params,
            variants,
        } => {
            assert_eq!(*name, "Option");
            assert_eq!(type_params.len(), 1);
            assert_eq!(variants.len(), 2);
        }
        _ => panic!("expected enum task"),
    }
}

#[test]
fn behavior_declaration_tasks_collect_behavior_signatures() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );

    let tasks = TypeChecker::collect_behavior_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Json");
    assert_eq!(tasks[0].type_params.len(), 1);
    assert_eq!(tasks[0].methods.len(), 1);
    assert_eq!(tasks[0].methods[0].name, "encode");
}

#[test]
fn expected_resolver_impl_method_symbols_collect_value_symbols_and_locals() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    x_value = (value: Point) i32 { value.x }
}
"#,
    );
    let Declaration::ImplBlock {
        type_name, methods, ..
    } = &program.declarations[1]
    else {
        panic!("expected impl block");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut expected = ResolverExpectedSymbolSets::default();

    collect_expected_resolver_impl_method_symbols(
        type_name,
        methods,
        &mut scope_cursor,
        &mut expected,
    );

    assert!(expected
        .declarations
        .contains(&(Namespace::Value, "Point.x_value".to_string())));
    assert!(expected.locals.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_callable_locals_collect_params_and_body() {
    let program = parse_program(
        r#"
main = (input: i32) i32 {
    value := input
    value
}
"#,
    );
    let Declaration::Function { params, body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut expected = HashSet::new();

    expected_resolver_callable_locals(params, body, &mut scope_cursor, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "input"));
    assert!(expected.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_scoped_expr_locals_collects_block_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    value
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut expected = HashSet::new();

    expected_resolver_scoped_expr_locals(body, &mut scope_cursor, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_child_expr_locals_collects_branch_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    loop {
        value := 1
        break
    }
    value
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let Expression::Block { statements, .. } = body else {
        panic!("expected block");
    };
    let Some(ast::Statement::Expression { expr, .. }) = statements.first() else {
        panic!("expected expression statement");
    };
    let Expression::Loop { body, .. } = expr else {
        panic!("expected loop expression");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_child_expr_locals(body, &mut scope_cursor, &locals, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_pattern_expr_locals_collects_pattern_and_body_bindings() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    value ?
        | Some(inner) {
            doubled := inner
            doubled
        }
        | None { 0 }
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[1] else {
        panic!("expected function");
    };
    let Expression::Block {
        expr: Some(expr), ..
    } = body
    else {
        panic!("expected block");
    };
    let Expression::Match { arms, .. } = expr.as_ref() else {
        panic!("expected match expression");
    };
    let arm = arms.first().expect("first match arm");
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_pattern_expr_locals(
        &arm.pattern,
        &arm.body,
        &mut scope_cursor,
        &locals,
        &mut expected,
    );

    assert!(expected.iter().any(|(name, _)| name == "inner"));
    assert!(expected.iter().any(|(name, _)| name == "doubled"));
}

#[test]
fn expected_resolver_pattern_locals_collects_struct_shorthand_bindings() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }

main = (point: Point) i32 {
    point ?
        | Point { x, y } { x + y }
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[1] else {
        panic!("expected function");
    };
    let Expression::Block {
        expr: Some(expr), ..
    } = body
    else {
        panic!("expected block");
    };
    let Expression::Match { arms, .. } = expr.as_ref() else {
        panic!("expected match expression");
    };
    let arm = arms.first().expect("first match arm");
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_pattern_locals(&arm.pattern, &mut scope_cursor, &mut locals, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "x"));
    assert!(expected.iter().any(|(name, _)| name == "y"));
}

#[test]
fn expected_resolver_block_locals_collects_statement_and_final_expr_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    (input: i32) i32 {
        input
    }
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let Expression::Block {
        statements, expr, ..
    } = body
    else {
        panic!("expected block");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_block_locals(
        statements,
        expr.as_deref(),
        &mut scope_cursor,
        &locals,
        &mut expected,
    );

    assert!(expected.iter().any(|(name, _)| name == "value"));
    assert!(expected.iter().any(|(name, _)| name == "input"));
}

#[test]
fn expected_resolver_statement_locals_preserve_mutable_handoff() {
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut locals = scope_cursor.new_scope();
    locals.insert("value".to_string(), true);
    let mut expected = HashSet::new();

    if resolver_var_decl_binds_local("value", false, false, &locals) {
        expected_resolver_var_decl_local("value", false, &mut locals, &mut expected);
    }

    assert!(
        expected.iter().all(|(name, _)| name != "value"),
        "immutable declaration should reuse the mutable handoff binding"
    );
}

#[test]
fn expected_resolver_closure_locals_collects_params_and_body_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (mut input: i32) i32 {
        inner := input
        inner
    }
    0
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let Expression::Block { statements, .. } = body else {
        panic!("expected block");
    };
    let Some(ast::Statement::VarDecl {
        value:
            Expression::Closure {
                params,
                body: closure_body,
                ..
            },
        ..
    }) = statements.first()
    else {
        panic!("expected closure var declaration");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_closure_locals(
        params,
        closure_body,
        &mut scope_cursor,
        &locals,
        &mut expected,
    );

    assert!(expected.iter().any(|(name, _)| name == "input"));
    assert!(expected.iter().any(|(name, _)| name == "inner"));
}

#[test]
fn resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Option: Some(i32), None

Json: behavior {
    encode: (Self) str
}

Point.impl = {
    x_value = (value: Point) i32 { value.x }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json)

main = () i32 { 0 }
"#,
    );

    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    assert_eq!(tasks.types.len(), 2);
    assert_eq!(tasks.behaviors.len(), 1);
    assert_eq!(tasks.behaviors[0].name, "Json");
    assert_eq!(tasks.callable.len(), 2);
    assert!(tasks.behavior_associations.extends.is_empty());
    assert_eq!(tasks.behavior_associations.impls.len(), 1);
    let behavior_impl = &tasks.behavior_associations.impls[0];
    assert_eq!(behavior_impl.ast_type_name, "Point");
    assert_eq!(behavior_impl.behavior, "Json");
    assert_eq!(behavior_impl.methods.len(), 1);
    assert_eq!(tasks.behavior_associations.requires.len(), 1);
    let requires = &tasks.behavior_associations.requires[0];
    assert_eq!(requires.type_name, "Point");
    assert_eq!(requires.behavior, "Json");
    assert_eq!(tasks.type_references.len(), 6);

    let tc = TypeChecker::new();
    let refresh_tasks = tc.resolver_type_behavior_refresh_tasks(&tasks, &symbols);
    assert_eq!(refresh_tasks.len(), 2);
    assert_eq!(refresh_tasks[0].restored_name, "Point");
    assert_eq!(refresh_tasks[1].restored_name, "Option");
}

#[test]
fn expected_behavior_edges_build_parent_edges_from_extends_together() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );

    let expected = ExpectedBehaviorEdges::parents_from(&program);
    let edge = &expected.edges_for("PrettyJson")[0];

    assert_eq!(edge.display, "Json");
    assert_eq!(edge.metadata.name, "Json");
    assert_eq!(edge.metadata.type_args, Vec::<AstType>::new());
}

#[test]
fn behavior_ref_role_validation_emits_selected_contains_diagnostics() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "pretty" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let ty = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");
    let mut tc = TypeChecker::new();

    tc.validate_resolver_behavior_ref_contains_for_role(
        BehaviorRefRole::Impl,
        ty,
        "Point",
        expected_behavior_edge("Json", &[AstType::Str]),
        Span::dummy(),
    );

    assert!(tc.diagnostics.iter().any(|d| d.code == "E0236" && d.message.contains(
            "resolver type symbol 'Point' has behavior impls 'PrettyJson', expected to include 'Json<str>'"
        )));
    assert!(tc.diagnostics.iter().any(|d| d.code == "E0247" && d.message.contains(
            "resolver type symbol 'Point' has behavior impl refs 'PrettyJson', expected to include 'Json<str>'"
        )));
}

#[test]
fn behavior_association_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");
    let entries = BehaviorAssociationAbsenceValidation {
        impl_name_code: "IMPL_NAMES",
        impl_ref_code: "IMPL_REFS",
        required_name_code: "REQUIRED_NAMES",
        required_ref_code: "REQUIRED_REFS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "IMPL_NAMES", "behavior impls"),
            AbsentMetadataEntry::new(true, "IMPL_REFS", "typed behavior impls"),
            AbsentMetadataEntry::new(true, "REQUIRED_NAMES", "behavior requires"),
            AbsentMetadataEntry::new(true, "REQUIRED_REFS", "typed behavior requires"),
        ]
    );
}

#[test]
fn behavior_association_absence_validation_uses_module_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0279");
    assert_eq!(validation.impl_ref_code, "E0378");
    assert_eq!(validation.required_name_code, "E0280");
    assert_eq!(validation.required_ref_code, "E0379");
}

#[test]
fn behavior_association_absence_validation_uses_import_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0295");
    assert_eq!(validation.impl_ref_code, "E0369");
    assert_eq!(validation.required_name_code, "E0296");
    assert_eq!(validation.required_ref_code, "E0370");
}

#[test]
fn behavior_association_absence_validation_uses_local_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0263");
    assert_eq!(validation.impl_ref_code, "E0387");
    assert_eq!(validation.required_name_code, "E0264");
    assert_eq!(validation.required_ref_code, "E0388");
}

#[test]
fn behavior_association_absence_validation_uses_variant_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0341");
    assert_eq!(validation.impl_ref_code, "E0395");
    assert_eq!(validation.required_name_code, "E0342");
    assert_eq!(validation.required_ref_code, "E0396");
}

#[test]
fn behavior_association_absence_validation_uses_behavior_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0327");
    assert_eq!(validation.impl_ref_code, "E0401");
    assert_eq!(validation.required_name_code, "E0328");
    assert_eq!(validation.required_ref_code, "E0402");
}

#[test]
fn behavior_association_absence_validation_uses_value_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0306");
    assert_eq!(validation.impl_ref_code, "E0407");
    assert_eq!(validation.required_name_code, "E0307");
    assert_eq!(validation.required_ref_code, "E0408");
}

#[test]
fn behavior_declaration_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Behavior, "PrettyJson")
        .expect("behavior symbol");
    let entries = BehaviorDeclarationAbsenceValidation {
        method_signature_code: "METHODS",
        method_type_code: "TYPED_METHODS",
        parent_name_code: "PARENTS",
        parent_ref_code: "TYPED_PARENTS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "METHODS", "behavior methods"),
            AbsentMetadataEntry::new(true, "TYPED_METHODS", "typed behavior methods"),
            AbsentMetadataEntry::new(true, "PARENTS", "behavior parents"),
            AbsentMetadataEntry::new(true, "TYPED_PARENTS", "typed behavior parents"),
        ]
    );
}

#[test]
fn behavior_declaration_absence_validation_uses_module_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0277");
    assert_eq!(validation.method_type_code, "E0376");
    assert_eq!(validation.parent_name_code, "E0278");
    assert_eq!(validation.parent_ref_code, "E0377");
}

#[test]
fn behavior_declaration_absence_validation_uses_import_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0293");
    assert_eq!(validation.method_type_code, "E0367");
    assert_eq!(validation.parent_name_code, "E0294");
    assert_eq!(validation.parent_ref_code, "E0368");
}

#[test]
fn behavior_declaration_absence_validation_uses_local_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0261");
    assert_eq!(validation.method_type_code, "E0385");
    assert_eq!(validation.parent_name_code, "E0262");
    assert_eq!(validation.parent_ref_code, "E0386");
}

#[test]
fn behavior_declaration_absence_validation_uses_variant_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0339");
    assert_eq!(validation.method_type_code, "E0393");
    assert_eq!(validation.parent_name_code, "E0340");
    assert_eq!(validation.parent_ref_code, "E0394");
}

#[test]
fn behavior_declaration_absence_validation_uses_value_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0304");
    assert_eq!(validation.method_type_code, "E0405");
    assert_eq!(validation.parent_name_code, "E0305");
    assert_eq!(validation.parent_ref_code, "E0406");
}
