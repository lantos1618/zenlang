use super::*;

#[test]
fn resolve_primitive_types() {
    let tc = TypeChecker::new();
    assert_eq!(tc.resolve_type(&AstType::I32), Type::I32);
    assert_eq!(tc.resolve_type(&AstType::F64), Type::F64);
    assert_eq!(tc.resolve_type(&AstType::Bool), Type::Bool);
    assert_eq!(tc.resolve_type(&AstType::Void), Type::Void);
    assert_eq!(tc.resolve_type(&AstType::Str), Type::Str);
}

#[test]
fn resolve_pointer_types() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.resolve_type(&AstType::Ptr(Box::new(AstType::I32))),
        Type::Ptr(Box::new(Type::I32))
    );
}

#[test]
fn method_signature_key_helpers_share_receiver_parsing() {
    assert_eq!(method_signature_key("Point", "get"), "Point.get");
    assert_eq!(
        method_signature_key_parts("Point.get"),
        Some(("Point", "get"))
    );
    assert_eq!(method_signature_receiver_name("Point.get"), Some("Point"));
    assert_eq!(
        method_signature_method_name_for_receiver("Point.get", "Point"),
        Some("get")
    );
    assert_eq!(
        method_signature_method_name_for_receiver("Other.get", "Point"),
        None
    );
    assert!(is_method_signature_key("Point.get"));
    assert_eq!(method_signature_key_parts("plain"), None);
    assert_eq!(method_signature_receiver_name("plain"), None);
    assert!(!is_method_signature_key("plain"));
}

#[test]
fn resolver_symbol_lookup_helpers_share_definition_span_fallbacks() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let Declaration::Method { span, .. } = &program.declarations[1] else {
        panic!("expected method declaration");
    };
    let span = *span;

    assert_eq!(
        TypeChecker::resolver_symbol_name_for(&symbols, Namespace::Value, "Point.missing", span),
        "Point.get"
    );
    assert_eq!(
        TypeChecker::resolver_method_signature_name_for(
            &symbols,
            "Missing.missing",
            "Missing",
            span
        ),
        "Point.get"
    );
    assert_eq!(
        TypeChecker::resolver_method_signature_symbol_by_span(&symbols, span)
            .map(|symbol| symbol.name.as_str()),
        Some("Point.get")
    );
}

#[test]
fn resolver_count_display_formats_known_and_missing_counts() {
    assert_eq!(resolver_count_display(Some(2)), "2");
    assert_eq!(resolver_count_display(None), "unknown");
}

#[test]
fn count_validation_formats_message() {
    let validation = CountValidation {
        label: "parameter count",
        code: "COUNT",
    };

    assert_eq!(validation.code, "COUNT");
    assert_eq!(
        validation.message("value", "add", Some(1), 2),
        "resolver value symbol 'add' has parameter count 1, expected 2"
    );
    assert_eq!(
        validation.message("variant", "Some", None, 1),
        "resolver variant symbol 'Some' has parameter count unknown, expected 1"
    );
}

#[test]
fn count_validation_uses_value_parameter_resolver_code() {
    let validation = CountValidation::value_parameter_resolver_code();

    assert_eq!(validation.label, "parameter count");
    assert_eq!(validation.code, "E0211");
}

#[test]
fn count_validation_uses_field_resolver_code() {
    let validation = CountValidation::field_resolver_code();

    assert_eq!(validation.label, "field count");
    assert_eq!(validation.code, "E0214");
}

#[test]
fn count_validation_uses_variant_payload_resolver_code() {
    let validation = CountValidation::variant_payload_resolver_code();

    assert_eq!(validation.label, "payload count");
    assert_eq!(validation.code, "E0215");
}

#[test]
fn type_parameter_validation_formats_messages() {
    let validation = TypeParameterValidation {
        count_code: "COUNT",
        name_code: "NAMES",
        bound_code: "BOUNDS",
        bound_ref_code: "BOUND_REFS",
    };

    assert_eq!(validation.name_code, "NAMES");
    assert_eq!(
        validation.name_message("value", "identity", "(U)", "(T)"),
        "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
    );
    assert_eq!(
        validation.bound_message("type", "Box", "(T: Other)", "(T: Json)"),
        "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
    );
    assert_eq!(
            validation.bound_ref_message("behavior", "Serializable", "(T: Json<i32>)", "(T: Json<T>)"),
            "resolver behavior symbol 'Serializable' has type parameter bound refs '(T: Json<i32>)', expected '(T: Json<T>)'"
        );
}

#[test]
fn type_parameter_validation_uses_type_like_resolver_codes() {
    let validation = TypeParameterValidation::type_like_resolver_codes();

    assert_eq!(validation.count_code, "E0213");
    assert_eq!(validation.name_code, "E0346");
    assert_eq!(validation.bound_code, "E0222");
    assert_eq!(validation.bound_ref_code, "E0350");
}

#[test]
fn type_parameter_validation_uses_value_resolver_codes() {
    let validation = TypeParameterValidation::value_resolver_codes();

    assert_eq!(validation.count_code, "E0220");
    assert_eq!(validation.name_code, "E0347");
    assert_eq!(validation.bound_code, "E0221");
    assert_eq!(validation.bound_ref_code, "E0351");
}

#[test]
fn type_parameter_validation_builds_count_validation() {
    let validation = TypeParameterValidation {
        count_code: "COUNT",
        name_code: "NAMES",
        bound_code: "BOUNDS",
        bound_ref_code: "BOUND_REFS",
    }
    .count_validation();

    assert_eq!(validation.label, "type parameter count");
    assert_eq!(validation.code, "COUNT");
}

#[test]
fn value_parameter_validation_formats_messages() {
    let validation = ValueParameterValidation {
        name_code: "NAMES",
        display_type_code: "TYPES",
        typed_type_code: "TYPED_TYPES",
    };

    assert_eq!(validation.name_code, "NAMES");
    assert_eq!(
        validation.name_message("add", "(a, other)", "(a, b)"),
        "resolver value symbol 'add' has parameter names '(a, other)', expected '(a, b)'"
    );
    assert_eq!(
        validation.display_type_message("add", "(i32, i32)", "(i32, f64)"),
        "resolver value symbol 'add' has parameter types '(i32, i32)', expected '(i32, f64)'"
    );
    assert_eq!(
        validation.typed_type_message("apply", "(i32)", "((i32) i32)"),
        "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
    );
}

#[test]
fn value_parameter_validation_uses_resolver_codes() {
    let validation = ValueParameterValidation::resolver_codes();

    assert_eq!(validation.name_code, "E0223");
    assert_eq!(validation.display_type_code, "E0216");
    assert_eq!(validation.typed_type_code, "E0356");
}

#[test]
fn return_validation_formats_messages() {
    let validation = ReturnValidation {
        display_code: "RETURN",
        typed_code: "TYPED_RETURN",
    };

    assert_eq!(validation.display_code, "RETURN");
    assert_eq!(
        validation.display_message("main", "bool", "i32"),
        "resolver value symbol 'main' has return type 'bool', expected 'i32'"
    );
    assert_eq!(
        validation.typed_message("apply", "i32", "(i32) i32"),
        "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
    );
}

#[test]
fn return_validation_uses_resolver_codes() {
    let validation = ReturnValidation::resolver_codes();

    assert_eq!(validation.display_code, "E0212");
    assert_eq!(validation.typed_code, "E0357");
}

#[test]
fn behavior_method_validation_formats_messages() {
    let validation = BehaviorMethodValidation {
        display_code: "METHODS",
        typed_code: "TYPED_METHODS",
    };

    assert_eq!(validation.display_code, "METHODS");
    assert_eq!(
            validation.display_message("Serializable", "(encode(Self) bool)", "(encode(Self) str)"),
            "resolver behavior symbol 'Serializable' has methods '(encode(Self) bool)', expected '(encode(Self) str)'"
        );
    assert_eq!(
            validation.typed_message(
                "Mapper",
                "(map(__arg0: Self, __arg1: i32) i32)",
                "(map(__arg0: Self, __arg1: (i32) i32) i32)"
            ),
            "resolver behavior symbol 'Mapper' has typed methods '(map(__arg0: Self, __arg1: i32) i32)', expected '(map(__arg0: Self, __arg1: (i32) i32) i32)'"
        );
}

#[test]
fn behavior_method_validation_uses_resolver_codes() {
    let validation = BehaviorMethodValidation::resolver_codes();

    assert_eq!(validation.display_code, "E0219");
    assert_eq!(validation.typed_code, "E0355");
}

#[test]
fn field_validation_formats_messages() {
    let validation = FieldValidation {
        display_code: "FIELDS",
        typed_code: "TYPED_FIELDS",
    };

    assert_eq!(validation.display_code, "FIELDS");
    assert_eq!(
        validation.display_message("type", "Point", "(x: i32)", "(x: f64)"),
        "resolver type symbol 'Point' has fields '(x: i32)', expected '(x: f64)'"
    );
    assert_eq!(
            validation.typed_message("type", "Pipeline", "(callback: i32)", "(callback: (i32) i32)"),
            "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
        );
}

#[test]
fn field_validation_uses_resolver_codes() {
    let validation = FieldValidation::resolver_codes();

    assert_eq!(validation.display_code, "E0217");
    assert_eq!(validation.typed_code, "E0358");
}

#[test]
fn variant_payload_validation_formats_messages() {
    let validation = VariantPayloadValidation {
        display_code: "PAYLOAD",
        typed_code: "TYPED_PAYLOAD",
    };

    assert_eq!(validation.display_code, "PAYLOAD");
    assert_eq!(
        validation.display_message("Some", "bool", "i32"),
        "resolver variant symbol 'Some' has payload type 'bool', expected 'i32'"
    );
    assert_eq!(
        validation.typed_message("Wrap", "i32", "(i32) i32"),
        "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
    );
}

#[test]
fn variant_payload_validation_uses_resolver_codes() {
    let validation = VariantPayloadValidation::resolver_codes();

    assert_eq!(validation.display_code, "E0218");
    assert_eq!(validation.typed_code, "E0359");
}

#[test]
fn variant_owner_validation_formats_message() {
    let validation = VariantOwnerValidation { code: "OWNER" };

    assert_eq!(validation.code, "OWNER");
    assert_eq!(
        validation.message("Some", "Result", "Option"),
        "resolver variant symbol 'Some' has owner 'Result', expected 'Option'"
    );
}

#[test]
fn variant_owner_validation_uses_resolver_code() {
    let validation = VariantOwnerValidation::resolver_code();

    assert_eq!(validation.code, "E0242");
}

#[test]
fn variant_name_validation_formats_message() {
    let validation = VariantNameValidation { code: "VARIANTS" };

    assert_eq!(validation.code, "VARIANTS");
    assert_eq!(
        validation.message("Option", "(Some)", "(Some, None)"),
        "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'"
    );
}

#[test]
fn variant_name_validation_uses_resolver_code() {
    let validation = VariantNameValidation::resolver_code();

    assert_eq!(validation.code, "E0241");
}

#[test]
fn resolver_metadata_display_formats_known_and_missing_values() {
    assert_eq!(resolver_metadata_display(Some("Point")), "Point");
    assert_eq!(resolver_metadata_display(None), "unknown");
    assert_eq!(
        resolver_ast_type_metadata_display(Some(&AstType::I32)),
        "i32"
    );
    assert_eq!(resolver_ast_type_metadata_display(None), "unknown");
    assert_eq!(
        optional_ast_type_display(Some(&AstType::Bool), "none"),
        "bool"
    );
    assert_eq!(optional_ast_type_display(None, "none"), "none");
}

#[test]
fn resolver_string_list_display_formats_known_and_missing_lists() {
    let names = vec!["T".to_string(), "U".to_string()];
    assert_eq!(join_resolver_strings(&names), "T, U");
    assert_eq!(
        join_resolver_display_values(&[AstType::I32, AstType::Bool], AstType::display_name),
        "i32, bool"
    );
    assert_eq!(format_resolver_string_list(Some(&names)), "(T, U)");
    assert_eq!(format_resolver_string_list(None), "unknown");
}

#[test]
fn resolver_display_list_formats_mapped_known_and_missing_items() {
    let types = vec![AstType::I32, AstType::Bool];
    assert_eq!(format_ast_type_list(Some(&types)), "(i32, bool)");
    assert_eq!(format_ast_type_list(None), "unknown");

    let bounds = vec![("T".to_string(), "Display".to_string())];
    assert_eq!(format_type_parameter_bounds(Some(&bounds)), "(T: Display)");
    assert_eq!(format_type_parameter_bounds(None), "unknown");

    let bound_refs = vec![TypeParameterBoundRefMetadata {
        type_parameter: "T".to_string(),
        behavior: "Display".to_string(),
        type_args: vec![AstType::I32],
    }];
    assert_eq!(
        format_type_parameter_bound_refs(Some(&bound_refs)),
        "(T: Display<i32>)"
    );
    assert_eq!(format_type_parameter_bound_refs(None), "unknown");
}

#[test]
fn resolver_nonempty_joined_list_formats_present_empty_and_missing_items() {
    let names = vec!["Json".to_string(), "Debug".to_string()];
    assert_eq!(format_behavior_ref_names(Some(&names)), "Json, Debug");
    assert_eq!(format_behavior_ref_names(Some(&[])), "none");
    assert_eq!(format_behavior_ref_names(None), "none");

    let refs = vec![BehaviorRefMetadata {
        name: "Json".to_string(),
        type_args: vec![AstType::I32],
    }];
    assert_eq!(format_behavior_refs(Some(&refs)), "Json<i32>");
    assert_eq!(format_behavior_refs(Some(&[])), "none");
    assert_eq!(format_behavior_refs(None), "none");
}

#[test]
fn resolver_behavior_ref_helpers_share_pop_and_peek_selection() {
    let refs = VecDeque::from(vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ]);
    let mut refs_by_type = HashMap::from([("Point".to_string(), refs.clone())]);

    assert_eq!(
        TypeChecker::peek_resolver_behavior_ref(true, &refs_by_type, "Point", "Debug")
            .map(|reference| reference.name.as_str()),
        Some("Debug")
    );
    assert_eq!(
        TypeChecker::pop_resolver_behavior_ref(true, &mut refs_by_type, "Point", "Debug")
            .map(|reference| reference.name),
        Some("Debug".to_string())
    );

    let mut refs_by_type = HashMap::from([("Point".to_string(), refs)]);
    assert_eq!(
        TypeChecker::peek_resolver_behavior_ref(true, &refs_by_type, "Point", "Missing")
            .map(|reference| reference.name.as_str()),
        Some("Json")
    );
    assert_eq!(
        TypeChecker::pop_resolver_behavior_ref(true, &mut refs_by_type, "Point", "Missing")
            .map(|reference| reference.name),
        Some("Json".to_string())
    );
    assert!(
        TypeChecker::peek_resolver_behavior_ref(false, &refs_by_type, "Point", "Debug").is_none()
    );
    assert!(
        TypeChecker::pop_resolver_behavior_ref(false, &mut refs_by_type, "Point", "Debug")
            .is_none()
    );
}

#[test]
fn resolver_behavior_ref_for_selects_impl_and_required_queues_by_role() {
    let mut tc = TypeChecker::new();
    tc.resolver_backed_collection = true;
    tc.resolver_behavior_impl_refs.insert(
        "Point".to_string(),
        VecDeque::from([BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    tc.resolver_behavior_required_refs.insert(
        "Point".to_string(),
        VecDeque::from([BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        }]),
    );

    assert_eq!(
        tc.resolver_behavior_ref_for(BehaviorRefRole::Impl, "Point", "Json")
            .map(|reference| reference.name),
        Some("Json".to_string())
    );
    assert_eq!(
        tc.resolver_behavior_ref_for(BehaviorRefRole::Required, "Point", "Debug")
            .map(|reference| reference.name),
        Some("Debug".to_string())
    );
    assert!(tc
        .resolver_behavior_ref_for(BehaviorRefRole::Parent, "Point", "Json")
        .is_none());

    tc.resolver_behavior_impl_refs.insert(
        "Point".to_string(),
        VecDeque::from([BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![],
        }]),
    );
    tc.resolver_backed_collection = false;
    assert!(tc
        .resolver_behavior_ref_for(BehaviorRefRole::Impl, "Point", "Json")
        .is_none());
}

#[test]
fn resolver_params_from_metadata_preserves_ast_param_shape() {
    let existing_span = Span::new(1, 10, 15);
    let default_span = Span::new(1, 20, 25);
    let existing_params = vec![Param {
        name: "stale".to_string(),
        ty: AstType::Bool,
        mutable: true,
        span: existing_span,
    }];
    let parameter_names = vec!["value".to_string(), "mapper".to_string()];
    let parameter_types = vec![
        AstType::I32,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        },
    ];

    let params = TypeChecker::resolver_params_from_metadata(
        &existing_params,
        &parameter_names,
        &parameter_types,
        default_span,
    );

    assert_eq!(params[0].name, "value");
    assert_eq!(params[0].ty, AstType::I32);
    assert!(params[0].mutable);
    assert_eq!(params[0].span, existing_span);
    assert_eq!(params[1].name, "mapper");
    assert_eq!(params[1].ty, parameter_types[1]);
    assert!(!params[1].mutable);
    assert_eq!(params[1].span, default_span);
}

#[test]
fn resolver_optional_return_type_maps_void_to_missing_annotation() {
    assert_eq!(
        TypeChecker::resolver_optional_return_type(&AstType::Void),
        None
    );
    assert_eq!(
        TypeChecker::resolver_optional_return_type(&AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        }),
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        })
    );
}

#[test]
fn resolver_enum_variants_from_metadata_uses_owner_scoped_payloads() {
    let program = parse_program(
        r#"
First: Wrap(i32), None
Second: Wrap(str)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let variants = vec!["Wrap".to_string(), "None".to_string()];

    assert_eq!(
        TypeChecker::resolver_enum_variants_from_metadata(&symbols, "First", &variants),
        vec![
            ("Wrap".to_string(), Some(AstType::I32)),
            ("None".to_string(), None),
        ]
    );
    assert_eq!(
        TypeChecker::resolver_enum_variants_from_metadata(
            &symbols,
            "Second",
            &["Wrap".to_string()]
        ),
        vec![("Wrap".to_string(), Some(AstType::Str))]
    );
}

#[test]
fn resolver_struct_fields_from_metadata_restores_field_names_and_defaults() {
    let fields = vec![
        ("x".to_string(), AstType::I32),
        (
            "callback".to_string(),
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Str),
            },
        ),
    ];
    let ast_fields = vec![
        StructField {
            name: "stale_x".to_string(),
            ty: AstType::Bool,
            default: Some(Expression::BoolLiteral {
                value: true,
                span: Span::dummy(),
            }),
            mutable: false,
            span: Span::dummy(),
        },
        StructField {
            name: "stale_callback".to_string(),
            ty: AstType::Bool,
            default: None,
            mutable: false,
            span: Span::dummy(),
        },
    ];

    let (restored_fields, defaults) =
        TypeChecker::resolver_struct_fields_from_metadata(&fields, &ast_fields);

    assert_eq!(restored_fields, fields);
    assert!(defaults.contains_key("x"));
    assert!(!defaults.contains_key("stale_x"));
    assert!(!defaults.contains_key("callback"));
}

#[test]
fn resolver_behavior_methods_from_metadata_preserves_defaults_by_resolver_order() {
    let first_span = Span::new(1, 10, 15);
    let second_span = Span::new(1, 20, 25);
    let default_span = Span::new(1, 30, 35);
    let existing_methods = vec![
        ast::BehaviorMethod {
            name: "first".to_string(),
            params: vec![Param {
                name: "stale".to_string(),
                ty: AstType::Bool,
                mutable: false,
                span: first_span,
            }],
            return_type: Some(AstType::Bool),
            default_body: Some(Expression::IntLiteral {
                value: 1,
                span: first_span,
            }),
            span: first_span,
        },
        ast::BehaviorMethod {
            name: "second".to_string(),
            params: vec![],
            return_type: None,
            default_body: Some(Expression::IntLiteral {
                value: 2,
                span: second_span,
            }),
            span: second_span,
        },
    ];
    let method_types = vec![
        BehaviorMethodTypeMetadata {
            name: "second".to_string(),
            parameter_names: vec!["value".to_string()],
            parameter_types: vec![AstType::I32],
            return_type: AstType::Str,
        },
        BehaviorMethodTypeMetadata {
            name: "first".to_string(),
            parameter_names: vec![],
            parameter_types: vec![],
            return_type: AstType::Void,
        },
    ];

    let methods = TypeChecker::resolver_behavior_methods_from_metadata(
        existing_methods,
        &method_types,
        default_span,
    );

    assert_eq!(methods[0].name, "second");
    assert_eq!(methods[0].params[0].name, "value");
    assert_eq!(methods[0].params[0].ty, AstType::I32);
    assert_eq!(methods[0].return_type, Some(AstType::Str));
    assert_eq!(methods[0].span, second_span);
    assert!(matches!(
        methods[0].default_body,
        Some(Expression::IntLiteral { value: 2, .. })
    ));
    assert_eq!(methods[1].name, "first");
    assert_eq!(methods[1].return_type, None);
    assert_eq!(methods[1].span, first_span);
    assert!(matches!(
        methods[1].default_body,
        Some(Expression::IntLiteral { value: 1, .. })
    ));
}

#[test]
fn behavior_parent_refs_from_metadata_restores_keys_and_type_args() {
    let tc = TypeChecker::new();
    let metadata = vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ];

    let refs = tc.behavior_parent_refs_from_metadata(&metadata);

    assert_eq!(refs[0].behavior, "Json");
    assert_eq!(refs[0].type_args, vec![AstType::Named("T".to_string())]);
    assert_eq!(refs[0].key, "Json_T");
    assert_eq!(refs[1].behavior, "Debug");
    assert!(refs[1].type_args.is_empty());
    assert_eq!(refs[1].key, "Debug");
}

#[test]
fn behavior_impl_refs_from_metadata_restores_type_and_behavior_keys() {
    let tc = TypeChecker::new();
    let metadata = vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::Str],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ];

    assert_eq!(
        tc.behavior_impl_refs_from_metadata("Point", &metadata),
        vec![
            ("Point".to_string(), "Json_str".to_string()),
            ("Point".to_string(), "Debug".to_string()),
        ]
    );
}

#[test]
fn resolver_behavior_ref_queue_selection_prefers_exact_then_front() {
    let refs = VecDeque::from(vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ]);

    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Debug"),
        Some(1)
    );
    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Missing"),
        Some(0)
    );
    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&VecDeque::new(), "Missing"),
        None
    );
}

#[test]
fn named_queue_selection_prefers_exact_then_front() {
    let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

    assert_eq!(
        TypeChecker::named_queue_index(&items, "Debug", String::as_str),
        Some(1)
    );
    assert_eq!(
        TypeChecker::named_queue_index(&items, "Missing", String::as_str),
        Some(0)
    );
    assert_eq!(
        TypeChecker::named_queue_index(&VecDeque::<String>::new(), "Missing", String::as_str),
        None
    );
}

#[test]
fn named_queue_selection_can_preserve_front_for_future_match() {
    let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Debug",
            Vec::<&str>::new(),
            String::as_str,
        ),
        Some(1)
    );
    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Missing",
            ["Json"],
            String::as_str,
        ),
        None
    );
    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Missing",
            ["Other"],
            String::as_str,
        ),
        Some(0)
    );
}

#[test]
fn impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature() {
    let mut tc = TypeChecker::new();
    tc.resolver_backed_collection = true;
    tc.methods.insert(
        "Point.describe".to_string(),
        FuncInfo {
            name: "Point.describe".to_string(),
            params: Vec::new(),
            return_type: AstType::Void,
            type_params: Vec::new(),
            type_param_bounds: HashMap::new(),
        },
    );
    let mut unmatched = VecDeque::from([
        "encode".to_string(),
        "debug".to_string(),
        "describe".to_string(),
    ]);

    assert_eq!(
        tc.impl_effective_method_name(
            &mut unmatched,
            "stale",
            Some("Point.encode".to_string()),
            "Point",
        ),
        "encode"
    );
    assert_eq!(
        tc.impl_effective_method_name(&mut unmatched, "debug", None, "Point"),
        "debug"
    );
    assert_eq!(
        tc.impl_effective_method_name(&mut unmatched, "missing", None, "Point"),
        "describe"
    );
    assert!(unmatched.is_empty());
}

#[test]
fn resolver_backed_impl_method_key_requires_resolver_collection() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let (span, ast_key) = if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function { name, span, .. } = &mut methods[0] {
            *name = "missing".to_string();
            (*span, TypeChecker::method_key(type_name, name))
        } else {
            panic!("expected impl method");
        }
    } else {
        panic!("expected impl block");
    };
    let mut tc = TypeChecker::new();

    assert_eq!(
        tc.resolver_backed_impl_method_key(Some(&symbols), &ast_key, "Missing", span),
        None
    );
    tc.resolver_backed_collection = true;
    assert_eq!(
        tc.resolver_backed_impl_method_key(Some(&symbols), &ast_key, "Missing", span),
        Some("Point.encode".to_string())
    );
}

#[test]
fn effective_behavior_impl_methods_carry_named_declaration_and_method_name() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let Declaration::ImplBlock { methods, .. } = &program.declarations[2] else {
        panic!("expected impl block");
    };
    let mut tc = TypeChecker::new();
    tc.resolver_backed_collection = true;
    let mut unmatched = VecDeque::from(["encode".to_string()]);

    let effective =
        tc.effective_behavior_impl_methods(Some(&symbols), "Point", methods, &mut unmatched);

    assert_eq!(effective.len(), 1);
    assert_eq!(effective[0].method_name, "encode");
    assert!(matches!(
        effective[0].declaration,
        Declaration::Function { name, .. } if name == "encode"
    ));
    assert!(unmatched.is_empty());
}

#[test]
fn template_dependency_entries_use_named_fields() {
    let entry = TemplateDependencyEntry::<StructInfo> {
        name: "Point".to_string(),
        previous: None,
    };

    assert_eq!(entry.name, "Point");
    assert!(entry.previous.is_none());
}

#[test]
fn resolver_backed_behavior_impl_method_signature_name_prefers_resolver_key() {
    let tc = TypeChecker::new();
    let mut required = VecDeque::from([
        ast::BehaviorMethod {
            name: "encode".to_string(),
            params: Vec::new(),
            return_type: Some(AstType::Str),
            default_body: None,
            span: Span::dummy(),
        },
        ast::BehaviorMethod {
            name: "debug".to_string(),
            params: Vec::new(),
            return_type: Some(AstType::Str),
            default_body: None,
            span: Span::dummy(),
        },
    ]);

    assert_eq!(
        tc.resolver_backed_behavior_impl_method_signature_name(
            &mut required,
            "stale",
            Some("Point.encode"),
            "Point",
        ),
        Some("encode".to_string())
    );
    assert_eq!(
        tc.resolver_backed_behavior_impl_method_signature_name(
            &mut required,
            "debug",
            None,
            "Point",
        ),
        Some("debug".to_string())
    );
    assert!(required.is_empty());
}

#[test]
fn resolver_backed_method_signature_requires_resolver_collection() {
    let mut tc = TypeChecker::new();
    tc.methods.insert(
        "Point.encode".to_string(),
        FuncInfo {
            name: "Point.encode".to_string(),
            params: Vec::new(),
            return_type: AstType::Str,
            type_params: Vec::new(),
            type_param_bounds: HashMap::new(),
        },
    );

    assert!(tc
        .resolver_backed_method_signature("Point", "encode")
        .is_none());
    tc.resolver_backed_collection = true;
    assert_eq!(
        tc.resolver_backed_method_signature("Point", "encode")
            .map(|info| info.return_type.clone()),
        Some(AstType::Str)
    );
}

#[test]
fn behavior_default_synthesis_skip_requires_resolver_collection_and_missing_impl_ref() {
    let mut tc = TypeChecker::new();
    tc.resolver_missing_behavior_impl_refs
        .insert("Point".to_string());

    assert!(!tc.should_skip_behavior_default_synthesis("Point"));
    tc.resolver_backed_collection = true;
    assert!(tc.should_skip_behavior_default_synthesis("Point"));
    assert!(!tc.should_skip_behavior_default_synthesis("Other"));
}

#[test]
fn resolver_backed_behavior_collection_defers_generic_metadata_to_resolver() {
    let program = parse_program(
        r#"
Json<T: Json<T>>: behavior {
    encode: (Self) T {
        1
    }
}
"#,
    );
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });

    let behavior = tc.behaviors.get("Json").expect("behavior stub");
    assert!(
            behavior.type_params.is_empty(),
            "resolver-backed behavior collection should not keep AST generic names before resolver metadata"
        );
    assert!(
            behavior.type_param_bounds.is_empty(),
            "resolver-backed behavior collection should not keep AST generic bounds before resolver metadata"
        );
    assert!(
            behavior.methods[0].default_body.is_some(),
            "resolver-backed behavior collection should still keep default bodies for later resolver metadata restoration"
        );
}

#[test]
fn method_key_formats_type_qualified_method_name() {
    assert_eq!(TypeChecker::method_key("Point", "encode"), "Point.encode");
}

#[test]
fn resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks() {
    let tc = TypeChecker::new();
    let mut refs_by_type = HashMap::from([
        (
            "Point".to_string(),
            VecDeque::from(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        ),
        (
            "Label".to_string(),
            VecDeque::from(vec![BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            }]),
        ),
    ]);
    let missing_refs = HashSet::new();

    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Json", &[AstType::I32]),
        Some("Point".to_string())
    );
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        None
    );

    refs_by_type.remove("Label");
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        Some("Point".to_string())
    );

    refs_by_type.clear();
    let missing_refs = HashSet::from(["Recovered".to_string()]);
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        Some("Recovered".to_string())
    );
}

#[test]
fn resolver_symbol_metadata_helper_requires_symbol_and_selected_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    assert_eq!(
        TypeChecker::resolver_symbol_metadata(&symbols, Namespace::Type, "Point", |symbol| {
            symbol.field_types.as_ref()
        })
        .map(|(_, fields)| fields[0].0.as_str()),
        Some("x")
    );
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    assert!(
        TypeChecker::resolver_symbol_metadata(&symbols, Namespace::Type, "Point", |symbol| symbol
            .field_types
            .as_ref())
        .is_none()
    );
    assert!(TypeChecker::resolver_symbol_metadata(
        &symbols,
        Namespace::Type,
        "Missing",
        |symbol| symbol.field_types.as_ref()
    )
    .is_none());
}

#[test]
fn resolver_callable_signature_metadata_requires_complete_signature() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let signature = TypeChecker::resolver_callable_signature_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol"),
    )
    .expect("complete resolver signature");

    assert_eq!(signature.parameter_names, ["value"]);
    assert_eq!(signature.parameter_types, [AstType::Named("T".to_string())]);
    assert_eq!(signature.return_type, &AstType::Named("T".to_string()));

    symbols.set_parameter_types_for_test(Namespace::Value, "identity", None);
    assert!(TypeChecker::resolver_callable_signature_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol")
    )
    .is_none());
}

#[test]
fn resolver_struct_field_metadata_requires_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let fields = TypeChecker::resolver_struct_field_metadata(
        symbols
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol"),
    )
    .expect("complete resolver fields");

    assert_eq!(fields, [("x".to_string(), AstType::I32)]);

    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    assert!(TypeChecker::resolver_struct_field_metadata(
        symbols
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol")
    )
    .is_none());
}

#[test]
fn resolver_enum_variant_name_metadata_requires_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let variants = TypeChecker::resolver_enum_variant_name_metadata(
        symbols
            .lookup(Namespace::Type, "Option")
            .expect("Option symbol"),
    )
    .expect("complete resolver variants");

    assert_eq!(variants, ["Some", "None"]);

    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    assert!(TypeChecker::resolver_enum_variant_name_metadata(
        symbols
            .lookup(Namespace::Type, "Option")
            .expect("Option symbol")
    )
    .is_none());
}

#[test]
fn resolver_behavior_method_metadata_requires_method_types() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let methods = TypeChecker::resolver_behavior_method_metadata(
        symbols
            .lookup(Namespace::Behavior, "Json")
            .expect("Json symbol"),
    )
    .expect("complete resolver behavior methods");

    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].name, "encode");
    assert_eq!(methods[0].parameter_types, [AstType::SelfType]);
    assert_eq!(methods[0].return_type, AstType::Str);

    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Json", None);
    assert!(TypeChecker::resolver_behavior_method_metadata(
        symbols
            .lookup(Namespace::Behavior, "Json")
            .expect("Json symbol")
    )
    .is_none());
}

#[test]
fn resolver_type_parameter_metadata_requires_names_and_bound_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let metadata = resolver_type_parameter_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol"),
    )
    .expect("complete resolver type-parameter metadata");

    assert_eq!(metadata.names, ["T"]);
    assert_eq!(metadata.bound_refs.len(), 1);
    assert_eq!(metadata.bound_refs[0].type_parameter, "T");
    assert_eq!(metadata.bound_refs[0].behavior, "Json");
    assert_eq!(
        metadata.bound_refs[0].type_args,
        [AstType::Named("T".to_string())]
    );

    symbols.set_type_parameter_bound_refs_for_test(Namespace::Value, "identity", None);
    assert!(resolver_type_parameter_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol")
    )
    .is_none());
}
