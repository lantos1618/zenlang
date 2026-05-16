use super::*;
use crate::ast::declarations::StructField;
use crate::ast::expressions::BinaryOp;
use crate::error::Span;

fn parse_program(src: &str) -> ast::Program {
    let mut files = crate::error::FileTable::new();
    let file_id = files.add_file("test.zen".to_string(), src.to_string());
    let tokens = crate::lexer::tokenize(src, file_id).expect("tokenize");
    crate::parser::parse(tokens, file_id).expect("parse")
}

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

#[test]
fn value_signature_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
add = (left: i32, right: i32) i32 { left + right }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Value, "add")
        .expect("value symbol");
    let entries = ValueSignatureAbsenceValidation {
        parameter_count_code: "PARAM_COUNT",
        parameter_name_code: "PARAM_NAMES",
        parameter_type_name_code: "PARAM_TYPES",
        parameter_type_code: "TYPED_PARAM_TYPES",
        return_type_code: "RETURN_TYPE",
        typed_return_type_code: "TYPED_RETURN_TYPE",
    }
    .entries(symbol);

    assert!(entries.iter().all(|entry| entry.present));
    assert_eq!(
        entries.map(|entry| entry.message("value", "add")),
        [
            "resolver value symbol 'add' has parameter count metadata, expected none",
            "resolver value symbol 'add' has parameter names metadata, expected none",
            "resolver value symbol 'add' has parameter types metadata, expected none",
            "resolver value symbol 'add' has typed parameter types metadata, expected none",
            "resolver value symbol 'add' has return type metadata, expected none",
            "resolver value symbol 'add' has typed return type metadata, expected none",
        ]
    );
}

#[test]
fn value_signature_absence_validation_uses_module_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0265");
    assert_eq!(validation.parameter_name_code, "E0267");
    assert_eq!(validation.parameter_type_name_code, "E0268");
    assert_eq!(validation.parameter_type_code, "E0371");
    assert_eq!(validation.return_type_code, "E0266");
    assert_eq!(validation.typed_return_type_code, "E0372");
}

#[test]
fn value_signature_absence_validation_uses_import_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0281");
    assert_eq!(validation.parameter_name_code, "E0283");
    assert_eq!(validation.parameter_type_name_code, "E0284");
    assert_eq!(validation.parameter_type_code, "E0362");
    assert_eq!(validation.return_type_code, "E0282");
    assert_eq!(validation.typed_return_type_code, "E0363");
}

#[test]
fn value_signature_absence_validation_uses_local_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0249");
    assert_eq!(validation.parameter_name_code, "E0251");
    assert_eq!(validation.parameter_type_name_code, "E0252");
    assert_eq!(validation.parameter_type_code, "E0380");
    assert_eq!(validation.return_type_code, "E0250");
    assert_eq!(validation.typed_return_type_code, "E0381");
}

#[test]
fn value_signature_absence_validation_uses_type_like_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0310");
    assert_eq!(validation.parameter_name_code, "E0312");
    assert_eq!(validation.parameter_type_name_code, "E0313");
    assert_eq!(validation.parameter_type_code, "E0360");
    assert_eq!(validation.return_type_code, "E0311");
    assert_eq!(validation.typed_return_type_code, "E0361");
}

#[test]
fn value_signature_absence_validation_uses_variant_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0330");
    assert_eq!(validation.parameter_name_code, "E0332");
    assert_eq!(validation.parameter_type_name_code, "E0333");
    assert_eq!(validation.parameter_type_code, "E0389");
    assert_eq!(validation.return_type_code, "E0331");
    assert_eq!(validation.typed_return_type_code, "E0390");
}

#[test]
fn type_parameter_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

identity<T: Json> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Value, "identity")
        .expect("value symbol");
    let entries = TypeParameterAbsenceValidation {
        count_code: "COUNT",
        name_code: "NAMES",
        bound_code: "BOUNDS",
        bound_ref_code: "BOUND_REFS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT", "type parameter count"),
            AbsentMetadataEntry::new(true, "NAMES", "type parameter names"),
            AbsentMetadataEntry::new(true, "BOUNDS", "type parameter bounds"),
            AbsentMetadataEntry::new(true, "BOUND_REFS", "typed type parameter bound refs"),
        ]
    );
}

#[test]
fn type_parameter_absence_validation_uses_module_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E0269");
    assert_eq!(validation.name_code, "E0348");
    assert_eq!(validation.bound_code, "E0270");
    assert_eq!(validation.bound_ref_code, "E0373");
}

#[test]
fn type_parameter_absence_validation_uses_import_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E0285");
    assert_eq!(validation.name_code, "E0349");
    assert_eq!(validation.bound_code, "E0286");
    assert_eq!(validation.bound_ref_code, "E0364");
}

#[test]
fn type_parameter_absence_validation_uses_local_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E0253");
    assert_eq!(validation.name_code, "E0350");
    assert_eq!(validation.bound_code, "E0254");
    assert_eq!(validation.bound_ref_code, "E0382");
}

#[test]
fn type_parameter_absence_validation_uses_variant_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E0334");
    assert_eq!(validation.name_code, "E0351");
    assert_eq!(validation.bound_code, "E0335");
    assert_eq!(validation.bound_ref_code, "E0391");
}

#[test]
fn field_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");
    let entries = FieldAbsenceValidation {
        count_code: "COUNT",
        type_name_code: "FIELD_TYPES",
        typed_code: "TYPED_FIELDS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT", "field count"),
            AbsentMetadataEntry::new(true, "FIELD_TYPES", "field types"),
            AbsentMetadataEntry::new(true, "TYPED_FIELDS", "typed field types"),
        ]
    );
}

#[test]
fn field_absence_validation_uses_module_resolver_codes() {
    let validation = FieldAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E0271");
    assert_eq!(validation.type_name_code, "E0272");
    assert_eq!(validation.typed_code, "E0374");
}

#[test]
fn field_absence_validation_uses_import_resolver_codes() {
    let validation = FieldAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E0287");
    assert_eq!(validation.type_name_code, "E0288");
    assert_eq!(validation.typed_code, "E0365");
}

#[test]
fn field_absence_validation_uses_local_resolver_codes() {
    let validation = FieldAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E0255");
    assert_eq!(validation.type_name_code, "E0256");
    assert_eq!(validation.typed_code, "E0383");
}

#[test]
fn field_absence_validation_uses_type_like_resolver_codes() {
    let validation = FieldAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.count_code, "E0319");
    assert_eq!(validation.type_name_code, "E0320");
    assert_eq!(validation.typed_code, "E0398");
}

#[test]
fn field_absence_validation_uses_variant_resolver_codes() {
    let validation = FieldAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E0336");
    assert_eq!(validation.type_name_code, "E0337");
    assert_eq!(validation.typed_code, "E0392");
}

#[test]
fn field_absence_validation_uses_behavior_resolver_codes() {
    let validation = FieldAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.count_code, "E0321");
    assert_eq!(validation.type_name_code, "E0322");
    assert_eq!(validation.typed_code, "E0399");
}

#[test]
fn field_absence_validation_uses_value_resolver_codes() {
    let validation = FieldAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.count_code, "E0298");
    assert_eq!(validation.type_name_code, "E0299");
    assert_eq!(validation.typed_code, "E0403");
}

#[test]
fn variant_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup_variant("Option", "Some")
        .expect("variant symbol");
    let entries = VariantAbsenceValidation {
        names_code: "NAMES",
        owner_code: "OWNER",
        payload_count_code: "PAYLOAD_COUNT",
        payload_type_name_code: "PAYLOAD_TYPE",
        payload_type_code: "TYPED_PAYLOAD",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(false, "NAMES", "variant names"),
            AbsentMetadataEntry::new(true, "OWNER", "variant owner"),
            AbsentMetadataEntry::new(true, "PAYLOAD_COUNT", "variant payload count"),
            AbsentMetadataEntry::new(true, "PAYLOAD_TYPE", "variant payload type"),
            AbsentMetadataEntry::new(true, "TYPED_PAYLOAD", "typed variant payload type"),
        ]
    );
}

#[test]
fn variant_absence_validation_uses_module_resolver_codes() {
    let validation = VariantAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.names_code, "E0273");
    assert_eq!(validation.owner_code, "E0274");
    assert_eq!(validation.payload_count_code, "E0275");
    assert_eq!(validation.payload_type_name_code, "E0276");
    assert_eq!(validation.payload_type_code, "E0375");
}

#[test]
fn variant_absence_validation_uses_import_resolver_codes() {
    let validation = VariantAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.names_code, "E0289");
    assert_eq!(validation.owner_code, "E0290");
    assert_eq!(validation.payload_count_code, "E0291");
    assert_eq!(validation.payload_type_name_code, "E0292");
    assert_eq!(validation.payload_type_code, "E0366");
}

#[test]
fn variant_absence_validation_uses_local_resolver_codes() {
    let validation = VariantAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.names_code, "E0257");
    assert_eq!(validation.owner_code, "E0258");
    assert_eq!(validation.payload_count_code, "E0259");
    assert_eq!(validation.payload_type_name_code, "E0260");
    assert_eq!(validation.payload_type_code, "E0384");
}

#[test]
fn variant_absence_validation_uses_type_like_resolver_codes() {
    let validation = VariantAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.names_code, "E0315");
    assert_eq!(validation.owner_code, "E0316");
    assert_eq!(validation.payload_count_code, "E0317");
    assert_eq!(validation.payload_type_name_code, "E0318");
    assert_eq!(validation.payload_type_code, "E0397");
}

#[test]
fn variant_absence_validation_uses_behavior_resolver_codes() {
    let validation = VariantAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.names_code, "E0323");
    assert_eq!(validation.owner_code, "E0324");
    assert_eq!(validation.payload_count_code, "E0325");
    assert_eq!(validation.payload_type_name_code, "E0326");
    assert_eq!(validation.payload_type_code, "E0400");
}

#[test]
fn variant_absence_validation_uses_value_resolver_codes() {
    let validation = VariantAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.names_code, "E0300");
    assert_eq!(validation.owner_code, "E0301");
    assert_eq!(validation.payload_count_code, "E0302");
    assert_eq!(validation.payload_type_name_code, "E0303");
    assert_eq!(validation.payload_type_code, "E0404");
}

#[test]
fn mutability_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
main = (mut input: i32) i32 {
    value ::= input
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup_scoped(Namespace::Local, "input")
        .expect("local symbol");
    let entries = MutabilityAbsenceValidation { code: "MUTABLE" }.entries(symbol);

    assert_eq!(
        entries,
        [AbsentMetadataEntry::new(true, "MUTABLE", "mutability")]
    );
}

#[test]
fn mutability_absence_validation_uses_module_resolver_code() {
    let validation = MutabilityAbsenceValidation::module_resolver_code();

    assert_eq!(validation.code, "E0345");
}

#[test]
fn mutability_absence_validation_uses_import_resolver_code() {
    let validation = MutabilityAbsenceValidation::import_resolver_code();

    assert_eq!(validation.code, "E0344");
}

#[test]
fn mutability_absence_validation_uses_type_like_resolver_code() {
    let validation = MutabilityAbsenceValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0314");
}

#[test]
fn mutability_absence_validation_uses_variant_resolver_code() {
    let validation = MutabilityAbsenceValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0343");
}

#[test]
fn mutability_absence_validation_uses_value_resolver_code() {
    let validation = MutabilityAbsenceValidation::value_resolver_code();

    assert_eq!(validation.code, "E0308");
}

#[test]
fn mutability_validation_formats_actual_and_expected() {
    let validation = MutabilityValidation { code: "MUTABLE" };

    assert_eq!(validation.code, "MUTABLE");
    assert_eq!(
        validation.display(Some(false), true),
        ("immutable", "mutable")
    );
    assert_eq!(validation.display(None, false), ("unknown", "immutable"));
    assert_eq!(
        validation.message("local", "value", Some(false), true),
        "resolver local symbol 'value' has mutability immutable, expected mutable"
    );
}

#[test]
fn mutability_validation_uses_resolver_code() {
    let validation = MutabilityValidation::resolver_code();

    assert_eq!(validation.code, "E0231");
}

#[test]
fn visibility_validation_formats_actual_and_expected() {
    let validation = VisibilityValidation { code: "VISIBLE" };

    assert_eq!(validation.code, "VISIBLE");
    assert_eq!(validation.display(true, false), ("public", "private"));
    assert_eq!(validation.display(false, true), ("private", "public"));
    assert_eq!(
        validation.message("import", "io", true, false),
        "resolver import symbol 'io' has visibility public, expected private"
    );
}

#[test]
fn visibility_validation_uses_local_resolver_code() {
    let validation = VisibilityValidation::local_resolver_code();

    assert_eq!(validation.code, "E0247");
}

#[test]
fn visibility_validation_uses_module_resolver_code() {
    let validation = VisibilityValidation::module_resolver_code();

    assert_eq!(validation.code, "E0229");
}

#[test]
fn visibility_validation_uses_import_resolver_code() {
    let validation = VisibilityValidation::import_resolver_code();

    assert_eq!(validation.code, "E0245");
}

#[test]
fn visibility_validation_uses_type_like_resolver_code() {
    let validation = VisibilityValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0225");
}

#[test]
fn visibility_validation_uses_variant_resolver_code() {
    let validation = VisibilityValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0226");
}

#[test]
fn visibility_validation_uses_value_resolver_code() {
    let validation = VisibilityValidation::value_resolver_code();

    assert_eq!(validation.code, "E0224");
}

#[test]
fn resolver_symbol_presence_validation_formats_messages() {
    let extra = ResolverSymbolPresenceValidation {
        code: "EXTRA",
        presence: ResolverSymbolPresence::Extra,
    };
    let missing = ResolverSymbolPresenceValidation {
        code: "MISSING",
        presence: ResolverSymbolPresence::Missing,
    };

    assert_eq!(extra.code, "EXTRA");
    assert_eq!(
        extra.message("value", "main"),
        "resolver symbol table has extra value symbol 'main'"
    );
    assert_eq!(missing.code, "MISSING");
    assert_eq!(
        missing.message("local", "value"),
        "resolver symbol table missing local symbol 'value'"
    );
}

#[test]
fn resolver_symbol_presence_validation_uses_resolver_codes() {
    let missing = ResolverSymbolPresenceValidation::missing_resolver_code();
    let missing_local = ResolverSymbolPresenceValidation::missing_local_resolver_code();
    let extra_declaration = ResolverSymbolPresenceValidation::extra_declaration_resolver_code();
    let extra_local = ResolverSymbolPresenceValidation::extra_local_resolver_code();

    assert_eq!(missing.code, "E0210");
    assert!(matches!(missing.presence, ResolverSymbolPresence::Missing));
    assert_eq!(missing_local.code, "E0228");
    assert!(matches!(
        missing_local.presence,
        ResolverSymbolPresence::Missing
    ));
    assert_eq!(extra_declaration.code, "E0243");
    assert!(matches!(
        extra_declaration.presence,
        ResolverSymbolPresence::Extra
    ));
    assert_eq!(extra_local.code, "E0244");
    assert!(matches!(
        extra_local.presence,
        ResolverSymbolPresence::Extra
    ));
}

#[test]
fn resolver_symbol_presence_validation_pushes_diagnostic() {
    let mut tc = TypeChecker::new();

    tc.validate_resolver_symbol_presence(
        "value",
        "main",
        ResolverSymbolPresenceValidation {
            code: "EXTRA",
            presence: ResolverSymbolPresence::Extra,
        },
        Span::dummy(),
    );

    assert_eq!(tc.diagnostics.len(), 1);
    assert_eq!(tc.diagnostics[0].code, "EXTRA");
    assert_eq!(
        tc.diagnostics[0].message,
        "resolver symbol table has extra value symbol 'main'"
    );
}

#[test]
fn source_absence_validation_builds_source_validation() {
    let validation = SourceAbsenceValidation { code: "SOURCE" }.source_validation();

    assert_eq!(validation.code, "SOURCE");
    assert_eq!(validation.actual_missing, "none");
    assert_eq!(validation.expected_missing, "none");
    assert!(!validation.quote_expected);
}

#[test]
fn source_absence_validation_uses_type_like_resolver_code() {
    let validation = SourceAbsenceValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0309");
}

#[test]
fn source_absence_validation_uses_variant_resolver_code() {
    let validation = SourceAbsenceValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0329");
}

#[test]
fn source_absence_validation_uses_value_resolver_code() {
    let validation = SourceAbsenceValidation::value_resolver_code();

    assert_eq!(validation.code, "E0297");
}

#[test]
fn source_validation_formats_message() {
    let quoted = SourceValidation {
        code: "SOURCE",
        actual_missing: "unknown",
        expected_missing: "none",
        quote_expected: true,
    };
    let unquoted = SourceValidation {
        code: "SOURCE",
        actual_missing: "none",
        expected_missing: "none",
        quote_expected: false,
    };

    assert_eq!(
        quoted.message("import", "io", Some("other"), Some("std")),
        "resolver import symbol 'io' has source 'other', expected 'std'"
    );
    assert_eq!(
        unquoted.message("value", "main", Some("std"), None),
        "resolver value symbol 'main' has source 'std', expected none"
    );
}

#[test]
fn source_validation_uses_resolver_codes() {
    let module = SourceValidation::module_resolver_code();
    let stripped_import = SourceValidation::stripped_import_resolver_code();
    let import = SourceValidation::import_resolver_code();
    let local = SourceValidation::local_resolver_code();

    assert_eq!(module.code, "E0230");
    assert_eq!(module.actual_missing, "none");
    assert_eq!(module.expected_missing, "none");
    assert!(!module.quote_expected);
    assert_eq!(stripped_import.code, "E0246");
    assert_eq!(stripped_import.actual_missing, "unknown");
    assert_eq!(stripped_import.expected_missing, "a module source");
    assert!(!stripped_import.quote_expected);
    assert_eq!(import.code, "E0227");
    assert_eq!(import.actual_missing, "unknown");
    assert_eq!(import.expected_missing, "none");
    assert!(import.quote_expected);
    assert_eq!(local.code, "E0248");
    assert_eq!(local.actual_missing, "none");
    assert_eq!(local.expected_missing, "none");
    assert!(!local.quote_expected);
}

#[test]
fn absent_metadata_entry_formats_message() {
    let entry = AbsentMetadataEntry {
        present: true,
        code: "ABSENT",
        label: "parameter count",
    };

    assert_eq!(entry.code, "ABSENT");
    assert_eq!(
        entry.message("value", "main"),
        "resolver value symbol 'main' has parameter count metadata, expected none"
    );
}

#[test]
fn resolver_named_list_display_formats_known_and_missing_items() {
    let fields = vec![("value".to_string(), "i32".to_string())];
    assert_eq!(
        format_resolver_named_list(Some(&fields), |ty: &String| ty.clone()),
        "(value: i32)"
    );
    assert_eq!(
        format_resolver_named_list::<String>(None, |ty: &String| ty.clone()),
        "unknown"
    );
}

#[test]
fn check_program_rejects_self_type_outside_method_or_behavior() {
    let program = parse_program(
        r#"
main = (value: Self) i32 { 0 }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("Self should require a method or behavior context");

    assert!(
        err.iter()
            .any(|d| d.message.contains("Self type is only valid")),
        "expected invalid Self type diagnostic, got {err:?}"
    );
}

#[test]
fn self_type_context_validation_tasks_collect_declarations() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option<T>: Some(T), None

Json: behavior {
    encode: (Self) str
}

Pretty.extends(Json)

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    x_value = (self: Point) i32 { self.x }
}

Point.requires(Json)

result := 1
"#,
    );

    let tasks = TypeChecker::collect_self_type_context_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 9);
    assert!(matches!(
        tasks[0],
        SelfTypeContextValidationTask::Struct { .. }
    ));
    assert!(matches!(
        tasks[1],
        SelfTypeContextValidationTask::Enum { .. }
    ));
    assert!(matches!(
        tasks[2],
        SelfTypeContextValidationTask::Behavior { .. }
    ));
    assert!(matches!(
        tasks[3],
        SelfTypeContextValidationTask::BehaviorExtends { .. }
    ));
    assert!(matches!(
        tasks[4],
        SelfTypeContextValidationTask::Function { .. }
    ));
    assert!(matches!(
        tasks[5],
        SelfTypeContextValidationTask::Method { .. }
    ));
    assert!(matches!(
        tasks[6],
        SelfTypeContextValidationTask::ImplBlock { .. }
    ));
    assert!(matches!(
        tasks[7],
        SelfTypeContextValidationTask::Requires { .. }
    ));
    assert!(matches!(
        tasks[8],
        SelfTypeContextValidationTask::TopLevelExpr { .. }
    ));
}

#[test]
fn ast_precollection_validation_tasks_collect_self_and_extends_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Pretty.extends(Json)

Point.impl = {
    x_value = (self: Point) i32 { self.x }
}

result := 1
"#,
    );

    let tasks = TypeChecker::collect_ast_precollection_validation_tasks(&program.declarations);

    assert_eq!(tasks.self_type_contexts.len(), 5);
    assert_eq!(tasks.behavior_associations.extends.len(), 1);
    assert_eq!(tasks.behavior_associations.extends[0].behavior, "Pretty");
    assert_eq!(tasks.behavior_associations.extends[0].parent, "Json");
    assert!(tasks.behavior_associations.impls.is_empty());
    assert!(tasks.behavior_associations.requires.is_empty());
}

#[test]
fn ast_declaration_collection_tasks_include_precollection_validation_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Pretty.extends(Json)

make = () Point { Point { x: 1 } }
"#,
    );

    let tasks = TypeChecker::collect_ast_declaration_collection_tasks(&program.declarations);

    assert_eq!(tasks.types.len(), 1);
    assert_eq!(tasks.behaviors.len(), 1);
    assert_eq!(tasks.callable.len(), 1);
    assert_eq!(tasks.precollection_validations.self_type_contexts.len(), 4);
    assert_eq!(
        tasks
            .precollection_validations
            .behavior_associations
            .extends
            .len(),
        1
    );
}

#[test]
fn check_program_rejects_unknown_type_references() {
    let program = parse_program(
        r#"
main = (value: Missing, items: Bag<i32>) i32 { 0 }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("unknown type reference should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown type diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Bag'")),
        "expected unknown generic type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_rejects_unknown_type_references_in_struct_field_defaults() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T = {
        same: Missing = 1
        same
    }
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("unknown struct field default type reference should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown field default type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_rejects_struct_field_default_type_mismatch() {
    let program = parse_program(
        r#"
Point: { x: i32 = "bad" }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("struct field default type mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("field `x` default expects `i32`, found `str`")),
        "expected field default type mismatch diagnostic, got {err:?}"
    );
}

#[test]
fn ast_struct_field_default_validation_tasks_collect_structs() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
Box<T>: { value: T }
"#,
    );

    let tasks =
        TypeChecker::collect_ast_struct_field_default_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].type_params.len(), 0);
    assert_eq!(tasks[0].fields.len(), 1);
    assert_eq!(tasks[1].type_params.len(), 1);
    assert_eq!(tasks[1].fields.len(), 1);
}

#[test]
fn resolver_type_declaration_metadata_tasks_collect_only_type_work() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
Option<T>: Some(T), None

main = () i32 { 1 }
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_type_declaration_metadata_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert!(matches!(
        tasks[0],
        ResolverTypeDeclarationMetadataTask::Struct { name: "Point", .. }
    ));
    assert!(matches!(
        tasks[1],
        ResolverTypeDeclarationMetadataTask::Enum { name: "Option", .. }
    ));
}

#[test]
fn resolver_type_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
"#,
    );
    let mut type_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_type_replay_tasks(
        &program.declarations[0],
        &mut type_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert!(matches!(
        type_tasks.as_slice(),
        [ResolverTypeDeclarationMetadataTask::Struct { name: "Point", .. }]
    ));
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::Struct { name: "Point", .. }]
    ));
}

#[test]
fn resolver_behavior_declaration_metadata_tasks_collect_only_behavior_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

main = () i32 { 1 }
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_behavior_declaration_metadata_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Json");
}

#[test]
fn resolver_behavior_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let mut behavior_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_behavior_replay_tasks(
        &program.declarations[0],
        &mut behavior_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert_eq!(behavior_tasks.len(), 1);
    assert_eq!(behavior_tasks[0].name, "Json");
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::Behavior { name: "Json", .. }]
    ));
}

#[test]
fn resolver_behavior_impl_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (self: Point) str { "point" }
}
"#,
    );
    let mut behavior_impl_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_behavior_impl_replay_tasks(
        &program.declarations[2],
        &mut behavior_impl_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert_eq!(behavior_impl_tasks.len(), 1);
    assert_eq!(behavior_impl_tasks[0].ast_type_name, "Point");
    assert_eq!(behavior_impl_tasks[0].behavior, "Json");
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::ImplBlock {
            type_name: "Point",
            ..
        }]
    ));
}

#[test]
fn resolver_callable_declaration_metadata_tasks_collect_callable_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    plus = (self: Point, other: Point) i32 { self.x + other.x }
}
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_callable_declaration_metadata_tasks(&program.declarations);

    assert_eq!(tasks.len(), 3);
    assert!(matches!(
        tasks[0],
        ResolverCallableDeclarationMetadataTask::Function { name: "make", .. }
    ));
    assert!(matches!(
        tasks[1],
        ResolverCallableDeclarationMetadataTask::Method {
            type_name: "Point",
            method_name: "get",
            ..
        }
    ));
    assert!(matches!(
        tasks[2],
        ResolverCallableDeclarationMetadataTask::TypeImpl {
            type_name: "Point",
            ..
        }
    ));
}

#[test]
fn resolver_callable_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

make = () Point { Point { x: 1 } }
"#,
    );
    let mut callable_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_callable_replay_tasks(
        &program.declarations[1],
        &mut callable_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert!(matches!(
        callable_tasks.as_slice(),
        [ResolverCallableDeclarationMetadataTask::Function { name: "make", .. }]
    ));
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::Function { name: "make", .. }]
    ));
}

#[test]
fn resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls() {
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

    let tasks =
        TypeChecker::collect_resolver_behavior_impl_block_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].ast_type_name, "Point");
    assert_eq!(tasks[0].behavior, "Json");
    assert_eq!(tasks[0].methods.len(), 1);
}

#[test]
fn ast_type_reference_validation_tasks_collect_declarations() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option<T>: Some(T), None

Json<T>: behavior {
    encode: (Self) T
}

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    x_value = (self: Point) i32 { self.x }
}

result := make()
"#,
    );

    let tasks = TypeChecker::collect_ast_type_reference_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 7);
    assert!(matches!(
        tasks[0],
        AstTypeReferenceValidationTask::Struct { .. }
    ));
    assert!(matches!(
        tasks[1],
        AstTypeReferenceValidationTask::Enum { .. }
    ));
    assert!(matches!(
        tasks[2],
        AstTypeReferenceValidationTask::Behavior { .. }
    ));
    assert!(matches!(
        tasks[3],
        AstTypeReferenceValidationTask::Function { .. }
    ));
    assert!(matches!(
        tasks[4],
        AstTypeReferenceValidationTask::Method { .. }
    ));
    assert!(matches!(
        tasks[5],
        AstTypeReferenceValidationTask::ImplBlock { .. }
    ));
    assert!(matches!(
        tasks[6],
        AstTypeReferenceValidationTask::TopLevelExpr { .. }
    ));
}

#[test]
fn ast_declaration_validation_tasks_collect_semantic_validation_work() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
Option<T>: Some(T), None

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (self: Point) str { "point" }
}

Point.requires(Json<str>)
JsonString.extends(Json<str>)

main = () i32 { 1 }
"#,
    );

    let tasks = TypeChecker::collect_ast_declaration_validation_tasks(&program.declarations);

    assert_eq!(tasks.behavior_associations.extends.len(), 1);
    assert_eq!(tasks.behavior_associations.impls.len(), 1);
    assert_eq!(tasks.behavior_associations.requires.len(), 1);
    assert_eq!(tasks.type_references.len(), 5);
    assert_eq!(tasks.struct_field_defaults.len(), 1);
}

#[test]
fn scope_variable_lookup() {
    let mut tc = TypeChecker::new();
    tc.define_var("x", Type::I32);
    assert_eq!(tc.lookup_var("x"), Some(Type::I32));

    tc.push_scope();
    tc.define_var("y", Type::Bool);
    assert_eq!(tc.lookup_var("y"), Some(Type::Bool));
    assert_eq!(tc.lookup_var("x"), Some(Type::I32)); // parent scope

    tc.pop_scope();
    assert_eq!(tc.lookup_var("y"), None); // out of scope
}

#[test]
fn collect_struct_info() {
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Struct {
        name: "Point".into(),
        type_params: Vec::new(),
        fields: vec![
            StructField {
                name: "x".into(),
                ty: AstType::F64,
                default: None,
                mutable: false,
                span: Span::dummy(),
            },
            StructField {
                name: "y".into(),
                ty: AstType::F64,
                default: None,
                mutable: false,
                span: Span::dummy(),
            },
        ],
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    assert!(tc.structs.contains_key("Point"));
    assert_eq!(tc.structs["Point"].fields.len(), 2);
}

#[test]
fn collect_enum_info() {
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Enum {
        name: "OptionI32".into(),
        type_params: Vec::new(),
        variants: vec![
            EnumVariant {
                name: "Some".into(),
                payload: Some(AstType::I32),
                span: Span::dummy(),
            },
            EnumVariant {
                name: "None".into(),
                payload: None,
                span: Span::dummy(),
            },
        ],
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    assert!(tc.enums.contains_key("OptionI32"));
    assert_eq!(tc.enums["OptionI32"].variants.len(), 2);
}

#[test]
fn collect_import_info() {
    let program = parse_program(
        r#"
{ io, fmt } = std

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.collect_declarations(&program.declarations);
    assert_eq!(tc.imports.get("io"), Some(&vec!["std".to_string()]));
    assert_eq!(tc.imports.get("fmt"), Some(&vec!["std".to_string()]));
}

#[test]
fn ast_import_declaration_tasks_collect_import_bindings() {
    let program = parse_program("{ Channel, Mutex } = std.sync");

    let tasks = TypeChecker::collect_ast_import_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks[0].names,
        &["Channel".to_string(), "Mutex".to_string()]
    );
    assert_eq!(
        tasks[0].module_path,
        &["std".to_string(), "sync".to_string()]
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_type_metadata() {
    let mut program = parse_program(
        r#"
apply = (callback: (i32) i32) (i32) i32 {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.functions.get("apply").expect("function info");
    assert_eq!(
        info.params[0].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.return_type,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_function_signature() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "main", None);
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should not keep AST-only function metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_function_signature_after_name_restore() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "main", None);
    if let Declaration::Function {
        name,
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST function signature key after resolver name restoration"
        );
    assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should clear the restored function signature key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_function_template() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should not keep AST-only generic function templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_generic_function_body_refs_when_signature_incomplete(
) {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T {
    same: T = value
    same
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should remove generic template when resolver signature metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST generic body refs when resolver signature metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_signature_for_type_refs() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Missing".to_string());
        *return_type = Some(AstType::Named("AlsoMissing".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.get = (self: Box, value: i32) i32 { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        params[1].ty = AstType::Named("Missing".to_string());
        *return_type = Some(AstType::Named("AlsoMissing".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored method signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { method_name, .. } = &mut program.declarations[1] {
        *method_name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Point.missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        type_name,
        method_name,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Missing.missing"));
}

#[test]
fn collect_declarations_with_symbols_clears_stale_method_signature_after_key_restore() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.get", None);
    if let Declaration::Method {
        type_name,
        method_name,
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST method signature key after resolver key restoration"
        );
    assert!(
            !tc.methods.contains_key("Point.get"),
            "resolver-backed collection should clear the restored method signature key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_name_metadata() {
    let mut program = parse_program(
        r#"
main = () i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { name, .. } = &mut program.declarations[0] {
        *name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.functions.contains_key("main"));
    assert!(!tc.functions.contains_key("missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_name_metadata() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { name, .. } = &mut program.declarations[0] {
        *name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.generic_functions.contains_key("identity"));
    assert!(!tc.generic_functions.contains_key("missing"));
}

#[test]
fn collect_declarations_with_symbols_clears_stale_generic_function_template_after_name_restore() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function {
        name,
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST generic function template key after resolver name restoration"
        );
    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should clear the restored generic function template key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_type_params_for_type_refs() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs() {
    let mut program = parse_program(
        r#"
Box<T>: { value: T }
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "StaleBox".to_string();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[1] {
        type_params[0].name = "StaleOption".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored type metadata should avoid stale AST type-ref diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_type_params_for_type_refs() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored function bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("MissingBox".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
        type_params[0].constraint = Some("MissingOption".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored type bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_type_bounds() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Box", None);
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Option", None);
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("MissingBox".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
        type_params[0].constraint = Some("MissingOption".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.structs
                .get("Box")
                .expect("struct info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed struct collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
    assert!(
            tc.enums
                .get("Option")
                .expect("enum info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed enum collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_bounds() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Behavior, "Serializable", None);
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Serializable").expect("behavior info");
    assert!(
            info.type_param_bounds.is_empty(),
            "resolver-backed behavior collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box: { value: i32 }
Box.impl = {
    keep<T: Json<T>> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { type_params, .. } = &mut methods[0] {
            type_params[0].constraint = Some("Missing".to_string());
            type_params[0].constraint_type_args.clear();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method bounds should avoid stale AST generic-bound diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Point.missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_target_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[1] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Missing.get"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_impl_method_signature() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.get", None);
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Point.get"),
            "resolver-backed collection should not keep AST-only impl method metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_name_metadata()
{
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert!(!tc.generic_methods.contains_key("Box.missing"));
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "value");
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Box: { value: i32 }

Box.impl = {
    apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
        callback
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[3] {
        if let Declaration::Function {
            type_params,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.apply")
        .expect("generic impl method template");
    assert_eq!(template.type_params, vec!["U".to_string()]);
    assert_eq!(
        tc.methods
            .get("Box.apply")
            .expect("impl method info")
            .type_param_bounds
            .get("U"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("U".to_string())],
        })
    );
    assert_eq!(
        template.params[1].ty,
        AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_return_presence(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T {
        value
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { return_type, .. } = &mut methods[0] {
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_parameter_count(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    choose<T> = (self: Box, left: T, right: T) T {
        left
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.pop();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic impl method template");
    assert_eq!(template.params.len(), 3);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("Box".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[2].ty, AstType::Named("T".to_string()));
}

#[test]
fn collect_declarations_with_symbols_preserves_type_impl_generic_template_param_mutability_by_position(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, mut value: T) T {
        value = value
        value
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params[1].name = "stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert_eq!(template.params[1].name, "value");
    assert!(
        template.params[1].mutable,
        "resolver-restored impl method parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_type_impl_generic_template_param_names_for_mutability(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    choose<T> = (self: Box, left: T, mut right: T) T {
        right = right
        right
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.swap(1, 2);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic impl method template");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert!(
        template.params[1].mutable,
        "resolver-restored first non-self impl parameter should keep first AST position mutability"
    );
    assert!(
            !template.params[2].mutable,
            "resolver-restored second non-self impl parameter should keep second AST position mutability"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_impl_method_template() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[1].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should not keep AST-only generic impl method templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_generic_impl_method_template_after_key_restore() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[1].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST generic impl method template key after resolver key restoration"
        );
    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should clear the restored generic impl method template key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_target_and_name_metadata(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert!(!tc.generic_methods.contains_key("Missing.missing"));
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "value");
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_name_for_body_type_refs(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T {
        same: T = value
        same
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            name, type_params, ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            type_params[0].name = "Stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic impl method name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
apply<T: Json<T>> = (callback: (T) T) (T) T {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        type_params,
        params,
        return_type,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        params[0].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc.generic_functions.get("apply").expect("generic template");
    assert_eq!(template.type_params, vec!["T".to_string()]);
    assert_eq!(
        tc.functions
            .get("apply")
            .expect("function info")
            .type_param_bounds
            .get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        template.params[0].ty,
        AstType::Function {
            params: vec![AstType::Named("T".to_string())],
            ret: Box::new(AstType::Named("T".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("T".to_string())],
            ret: Box::new(AstType::Named("T".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_name_for_body_type_refs() {
    let mut program = parse_program(
        r#"
keep<T> = (value: T) T {
    same: T = value
    same
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        name, type_params, ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic function name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_generic_function_template_type_params_when_resolver_bounds_missing(
) {
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
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Value, "identity", None);
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("identity")
        .expect("generic function template");
    assert!(
            template.type_params.is_empty(),
            "resolver-backed generic templates should not keep type parameter names when typed bound metadata is incomplete"
        );
    assert!(
            tc.functions
                .get("identity")
                .expect("function info")
                .type_params
                .is_empty(),
            "function info and template type parameter handoff should agree when resolver metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_return_presence() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T {
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { return_type, .. } = &mut program.declarations[0] {
        *return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("identity")
        .expect("generic template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_parameter_count() {
    let mut program = parse_program(
        r#"
choose<T> = (left: T, right: T) T {
    left
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("choose")
        .expect("generic template");
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "left");
    assert_eq!(template.params[1].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
}

#[test]
fn collect_declarations_with_symbols_preserves_generic_template_param_mutability_by_position() {
    let mut program = parse_program(
        r#"
keep<T> = (mut value: T) T {
    value = value
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc.generic_functions.get("keep").expect("generic template");
    assert_eq!(template.params[0].name, "value");
    assert!(
        template.params[0].mutable,
        "resolver-restored parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_generic_template_param_names_for_mutability() {
    let mut program = parse_program(
        r#"
choose<T> = (left: T, mut right: T) T {
    right = right
    right
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params.swap(0, 1);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("choose")
        .expect("generic template");
    assert_eq!(template.params[0].name, "left");
    assert_eq!(template.params[1].name, "right");
    assert!(
        template.params[0].mutable,
        "resolver-restored first parameter should keep first AST position mutability"
    );
    assert!(
        !template.params[1].mutable,
        "resolver-restored second parameter should keep second AST position mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Box: { value: i32 }
Box.apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        type_params,
        params,
        return_type,
        ..
    } = &mut program.declarations[3]
    {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        params[1].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.apply")
        .expect("generic method template");
    assert_eq!(template.type_params, vec!["U".to_string()]);
    assert_eq!(
        tc.methods
            .get("Box.apply")
            .expect("method info")
            .type_param_bounds
            .get("U"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("U".to_string())],
        })
    );
    assert_eq!(
        template.params[1].ty,
        AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_clears_generic_method_template_type_params_when_resolver_bounds_missing(
) {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box: { value: i32 }
Box.keep<U: Json<U>> = (self: Box, value: U) U { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Value, "Box.keep", None);
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic method template");
    assert!(
            template.type_params.is_empty(),
            "resolver-backed generic method templates should not keep type parameter names when typed bound metadata is incomplete"
        );
    assert!(
            tc.methods
                .get("Box.keep")
                .expect("method info")
                .type_params
                .is_empty(),
            "method info and template type parameter handoff should agree when resolver metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_template_return_presence() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T {
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { return_type, .. } = &mut program.declarations[1] {
        *return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic method template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_template_parameter_count() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.choose<T> = (self: Box, left: T, right: T) T {
    left
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic method template");
    assert_eq!(template.params.len(), 3);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("Box".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[2].ty, AstType::Named("T".to_string()));
}

#[test]
fn collect_declarations_with_symbols_preserves_generic_method_template_param_mutability_by_position(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, mut value: T) T {
    value = value
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params[1].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic method template");
    assert_eq!(template.params[1].name, "value");
    assert!(
        template.params[1].mutable,
        "resolver-restored method parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_generic_method_template_param_names_for_mutability(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.choose<T> = (self: Box, left: T, mut right: T) T {
    right = right
    right
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params.swap(1, 2);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic method template");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert!(
            template.params[1].mutable,
            "resolver-restored first non-self method parameter should keep first AST position mutability"
        );
    assert!(
            !template.params[2].mutable,
            "resolver-restored second non-self method parameter should keep second AST position mutability"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_method_template() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::Method {
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        params[1].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should not keep AST-only generic method templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_generic_method_template_after_key_restore() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::Method {
        type_name,
        method_name,
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
        params[1].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST generic method template key after resolver key restoration"
        );
    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should clear the restored generic method template key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_name_for_body_type_refs() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T {
    same: T = value
    same
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        method_name,
        type_params,
        ..
    } = &mut program.declarations[1]
    {
        *method_name = "missing".to_string();
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic method name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_field_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Pipeline<T: Json<T>>: { callback: (i32) i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct {
        type_params,
        fields,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        fields[0].ty = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.structs.get("Pipeline").expect("struct info");
    assert_eq!(
        info.type_param_bounds.get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        info.fields[0].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.structs.contains_key("Point"));
    assert!(!tc.structs.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_field_names_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
        fields[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should use resolver-restored field names: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_struct_field_defaults_validate_from_type_metadata_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Struct { fields, .. } = &mut stale_declarations[0] {
        fields.clear();
    }
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| checker.collect_declarations(&stale_declarations));
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_resolver_struct_field_default_tasks(&tasks, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should use precollected type tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_backed_struct_field_defaults_reuse_metadata_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Struct { fields, .. } = &mut stale_declarations[0] {
        fields.clear();
    }
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| checker.collect_declarations(&stale_declarations));
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_struct_field_defaults(&program.declarations, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should reuse shared metadata tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_backed_semantic_validation_reuses_metadata_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations)
    });
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_collected_declaration_semantics(&program.declarations, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed semantic validation should reuse resolver metadata tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_struct_fields() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
        fields[0].ty = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should not keep AST-only struct fields when resolver field metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete(
) {
    let mut program = parse_program(
        r#"
Box<T>: {
    value: T = {
        same: T = 1
        same
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Box", None);
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Box"),
            "resolver-backed collection should remove struct fields when resolver field metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST struct field default refs when resolver field metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_struct_fields_after_name_restore() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    if let Declaration::Struct { name, fields, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        fields[0].ty = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST struct key after resolver name restoration"
        );
    assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should clear the restored struct key when resolver field metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_enum_payload_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Callback<T: Json<T>>: Wrap((i32) i32), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Enum {
        type_params,
        variants,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        variants[0].payload = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.enums.get("Callback").expect("enum info");
    assert_eq!(
        info.type_param_bounds.get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        info.variants[0].1,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_enum_name_metadata() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Enum { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.enums.contains_key("Option"));
    assert!(!tc.enums.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_enum_variants() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    if let Declaration::Enum { variants, .. } = &mut program.declarations[0] {
        variants[0].payload = Some(AstType::Named("Stale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should not keep AST-only enum variants when resolver variant metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_enum_variants_after_name_restore() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    if let Declaration::Enum { name, variants, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        variants[0].payload = Some(AstType::Named("Stale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.enums.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST enum key after resolver name restoration"
        );
    assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should clear the restored enum key when resolver variant metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_metadata() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(
        info.methods[0].params[1].ty,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.methods[0].return_type,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_name_metadata() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.behaviors.contains_key("Json"));
    assert!(!tc.behaviors.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_methods() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
        methods[0].params[1].ty = AstType::Named("Stale".to_string());
        methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should not keep AST-only behavior methods when resolver method metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_behavior_default_body_refs_when_methods_incomplete(
) {
    let mut program = parse_program(
        r#"
Mapper<T>: behavior {
    map: (Self, value: T) T {
        same: T = value
        same
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should remove behavior methods when resolver method metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST behavior default body refs when resolver method metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_methods_after_name_restore() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { name, methods, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        methods[0].params[1].ty = AstType::Named("Stale".to_string());
        methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST behavior key after resolver name restoration"
        );
    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should clear the restored behavior key when resolver method metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_name_metadata() {
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
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].name, "encode");
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method name metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_return_presence_metadata() {
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
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].return_type, Some(AstType::Str));
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method return metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_count() {
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
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.push(Param {
            name: "stale".to_string(),
            ty: AstType::I32,
            mutable: false,
            span: Span::dummy(),
        });
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].params.len(), 1);
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_missing_parameter_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (Self, i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(info.methods[0].params.len(), 2);
    assert_eq!(info.methods[0].params[0].name, "__arg0");
    assert_eq!(info.methods[0].params[1].name, "__arg1");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert_eq!(info.methods[0].params[1].ty, AstType::I32);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_names() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].params[0].name, "value");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_behavior_method_parameter_order() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.swap(0, 1);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(info.methods[0].params[0].name, "value");
    assert_eq!(info.methods[0].params[1].name, "input");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert_eq!(info.methods[0].params[1].ty, AstType::I32);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
    describe: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
    describe = (value: Point) str { "desc" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods.len(), 2);
    assert_eq!(info.methods[0].name, "encode");
    assert_eq!(info.methods[1].name, "describe");
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior methods should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32 { callback }
}

Point.implements(Mapper) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.map").expect("default method info");
    assert_eq!(
        info.params[1].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.return_type,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored default method");
    assert_eq!(info.params[0].0, "self");
    assert_eq!(info.return_type, AstType::Str);
    assert!(
        !tc.methods.contains_key("Point.missing"),
        "stale AST-only behavior default method name should not be synthesized"
    );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior default method name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_skips_default_when_resolver_restores_impl_method_name() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let method = tc
        .methods
        .get("Point.encode")
        .expect("restored impl method");
    assert_eq!(
        method.params[0].0, "value",
        "resolver-restored explicit impl method should not be overwritten by the behavior default"
    );
    assert!(
        !tc.methods.contains_key("Point.missing"),
        "stale AST-only impl method key should be removed"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method name should suppress default insertion: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_behavior_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[3] {
        *behavior = Some("Debug".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let method = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored behavior default");
    assert_eq!(method.params[0].0, "self");
    assert_eq!(method.return_type, AstType::Str);
    assert!(
        !tc.methods.contains_key("Point.describe"),
        "stale AST-only behavior default should not be synthesized"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl metadata should drive default synthesis: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[2] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl target should drive omitted default synthesis: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("Debug".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        !tc.methods.contains_key("Point.describe"),
        "stale AST-only behavior default should not be synthesized"
    );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl target and name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_defers_impl_checks_until_resolver_metadata_is_collected() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { callback }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_metadata_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { callback }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks() {
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
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method name metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    extra = (value: Point) str { "extra" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "encode".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| *message == "method `extra` is not declared by behavior `Json`"),
        "resolver-owned extra impl method should not be hidden by stale AST required name: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_parameter_names_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params[0].name = "stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.encode").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_impl_method_parameter_order_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.swap(0, 1);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.map").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[1].0, "input");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert_eq!(info.params[1].1, AstType::I32);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_name_metadata() {
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
    if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[2] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl target should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.impl = {
    get = (self: Point) i32 { self.x }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    assert!(
        tc.methods.contains_key("Point.get"),
        "non-behavior impl methods should still be refreshed by declaration metadata"
    );
    assert!(
        !tc.methods.contains_key("Point.encode"),
        "behavior impl method signatures should be owned by the behavior impl metadata pass"
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_method_signature()
{
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
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should not keep AST-only method metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_impl_method_signature_after_key_restore()
{
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
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        !tc.methods.contains_key("Missing.missing"),
        "resolver-backed behavior impl collection should not keep stale AST method keys"
    );
    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored method keys when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_method_signature_target_and_name_metadata(
) {
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
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.encode").expect("impl method info");
    assert!(!tc.methods.contains_key("Missing.missing"));
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert_eq!(info.return_type, AstType::Str);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl method signature should avoid stale AST diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Point.encode")
        .expect("generic behavior impl method template");
    assert!(!tc.generic_methods.contains_key("Missing.missing"));
    assert!(!tc.generic_methods.contains_key("Point.missing"));
    assert_eq!(template.type_params, vec!["T".to_string()]);
    assert_eq!(template.params.len(), 1);
    assert_eq!(template.params[0].name, "value");
    assert_eq!(template.params[0].ty, AstType::Named("Point".to_string()));
    assert_eq!(template.return_type, Some(AstType::Str));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl generic template should avoid stale AST diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        !tc.generic_methods.contains_key("Missing.missing"),
        "resolver-backed behavior impl collection should clear stale AST generic method templates"
    );
    assert!(
            !tc.generic_methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored generic method templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    assert_eq!(parents[0].behavior, "Json");
    assert_eq!(parents[0].type_args, vec![AstType::Str]);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_refs_for_test(Namespace::Behavior, "PrettyJson", None);
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = &mut program.declarations[2]
    {
        parent_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_extends.contains_key("PrettyJson"),
            "resolver-backed collection should not keep AST-only behavior parent refs when resolver parent metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args() {
    let mut program = parse_program(
        r#"
Marker<T>: behavior {
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Marker<str>)
PrettyJson.extends(Marker<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = second_parent
    {
        parent_type_args[0] = AstType::Str;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
    assert_eq!(parent_keys, vec!["Marker_str", "Marker_i32"]);
    assert!(
        tc.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("duplicate behavior inheritance")),
        "resolver-restored parent type args should avoid false duplicate diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_parent_and_type_param_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[2] {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc.behavior_extends.get("Pretty").expect("behavior parents");
    assert_eq!(parents[0].behavior, "Serializable");
    assert_eq!(parents[0].type_args, vec![AstType::Named("T".to_string())]);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent and type-parameter metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message == "type `Point` implementation of `PrettyJson` is missing required method `encode`"
            }),
            "resolver-restored parent metadata should report the inherited missing method, got {:?}",
            messages
        );
    assert!(
        messages.iter().all(|message| !message.contains("Missing")),
        "stale AST-only behavior parent names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_reports_conflict_from_restored_parent_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Debug<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = second_parent
    {
        parent_type_args[0] = AstType::Str;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message == "conflicting behavior method `encode` inherited by `PrettyJson`"
            }),
            "resolver-restored parent type args should drive inherited method coherence diagnostics, got {:?}",
            messages
        );
    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
    assert_eq!(parent_keys, vec!["Json_str", "Debug_i32"]);
}

#[test]
fn collect_declarations_with_symbols_reports_cycle_from_restored_parent_refs() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
PrettyJson: behavior {
    pretty: (Self) str
}
Debug: behavior {
    debug: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends { parent, .. } = second_parent {
        *parent = "Debug".to_string();
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("behavior inheritance cycle")),
        "resolver-restored parent refs should drive cycle diagnostics, got {:?}",
        messages
    );
    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    assert_eq!(parents[0].behavior, "Json");
}

#[test]
fn collect_declarations_with_symbols_synthesizes_defaults_from_restored_behavior_parent() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str { "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends { parent, .. } = &mut program.declarations[3] {
        *parent = "Missing".to_string();
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.methods.contains_key("Point.encode"),
        "resolver-restored parent metadata should synthesize inherited default method"
    );
    assert!(
        !tc.methods.contains_key("Point.Missing"),
        "stale AST-only parent names should not synthesize default methods"
    );
}

#[test]
fn collect_declarations_with_symbols_synthesizes_generic_defaults_from_restored_parent_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let encode = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored parent should synthesize inherited default");
    assert_eq!(
        encode.return_type,
        AstType::Str,
        "resolver-restored parent type args should drive inherited default return type"
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = &mut program.declarations[2]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string())),
        "resolver metadata should restore the validated Json<str> impl"
    );
    assert!(
        !tc.behavior_impls
            .contains(&("Point".to_string(), "Json_i32".to_string())),
        "AST-only Json<i32> impl drift should not remain after resolver collection"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = &mut program.declarations[2]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_synthesize_stale_impl_defaults_after_target_restore()
{
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
    assert!(
        !tc.methods.contains_key("Missing.encode"),
        "resolver-backed default synthesis should not keep stale AST target method keys"
    );
    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed default synthesis should not synthesize behavior defaults when resolver impl metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only impl refs after target restoration when resolver impl metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[2] {
        *behavior = Some("Missing".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl name metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string())),
        "resolver metadata should restore the validated Point implements Json<str> association"
    );
    assert!(
            !tc.behavior_impls
                .contains(&("Missing".to_string(), "AlsoMissing_i32".to_string())),
            "stale AST-only impl target and behavior metadata should not remain after resolver collection"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl target and behavior metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_impl_target_and_name() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message == "type `Point` implementation of `Json_str` is missing required method `encode`"
            }),
            "resolver-restored impl metadata should report the validated missing method, got {:?}",
            messages
        );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
        "stale AST-only impl names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_reports_overlap_from_restored_impl_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let first_impl = program
        .declarations
        .iter_mut()
        .find(|declaration| {
            matches!(
                declaration,
                Declaration::ImplBlock {
                    behavior: Some(behavior),
                    ..
                } if behavior == "Json"
            )
        })
        .expect("Json impl declaration");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = first_impl
    {
        behavior_type_args[0] = AstType::I32;
    } else {
        panic!("expected Json impl declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message
                    == "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
            }),
            "resolver-restored impl type args should drive overlap diagnostics, got {:?}",
            messages
        );
    assert!(
        messages.iter().all(|message| !message.contains("Json_i32")),
        "stale AST-only impl type args should not leak into overlap diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_impl_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_impl = program
        .declarations
        .iter_mut()
        .filter(|declaration| {
            matches!(
                declaration,
                Declaration::ImplBlock {
                    behavior: Some(behavior),
                    ..
                } if behavior == "Json"
            )
        })
        .nth(1)
        .expect("second Json impl declaration");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = second_impl
    {
        behavior_type_args[0] = AstType::Str;
    } else {
        panic!("expected second Json impl declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("duplicate implementation")),
        "resolver-restored impl type args should avoid false duplicate diagnostics, got {:?}",
        messages
    );
    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string()))
            && tc
                .behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
        "resolver-restored impl type args should keep distinct impl specializations: {:?}",
        tc.behavior_impls
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        behavior_type_args, ..
    } = &mut program.declarations[3]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored requires metadata should avoid stale AST requires diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires { type_name, .. } = &mut program.declarations[3] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target and behavior metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_required_target_and_name() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| *message
                == "type `Point` does not implement required behavior `Json_str`"),
        "resolver-restored requires metadata should report the validated missing impl, got {:?}",
        messages
    );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
        "stale AST-only requires names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_uses_restored_requires_ref_for_inherited_impl() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let requires = program
        .declarations
        .iter_mut()
        .find(|declaration| matches!(declaration, Declaration::Requires { .. }))
        .expect("requires declaration");
    if let Declaration::Requires {
        behavior,
        behavior_type_args,
        ..
    } = requires
    {
        *behavior = "Missing".to_string();
        behavior_type_args[0] = AstType::I32;
    } else {
        panic!("expected requires declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored requires ref should be satisfied by inherited child impl: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_distinct_restored_requires_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_requires = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::Requires { .. }))
        .nth(1)
        .expect("second requires declaration");
    if let Declaration::Requires {
        behavior_type_args, ..
    } = second_requires
    {
        behavior_type_args[0] = AstType::Str;
    } else {
        panic!("expected requires declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("does not implement required behavior")),
        "resolver-restored requires type args should keep distinct satisfied specializations: {:?}",
        tc.diagnostics
    );
    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string()))
            && tc
                .behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
        "resolver-restored impl refs should keep both required specializations available: {:?}",
        tc.behavior_impls
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_required_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::Requires {
        behavior_type_args, ..
    } = &mut program.declarations[3]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_requires_after_target_restore() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs after target restoration when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires { behavior, .. } = &mut program.declarations[3] {
        *behavior = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires name metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn check_program_with_symbols_requires_resolver_declarations() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let empty_symbols = SymbolTable::default();
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &empty_symbols)
        .expect_err("missing resolver symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing value symbol 'main'")),
        "expected missing resolver symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_declarations() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
main = () i32 { 0 }
extra = () i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver declarations should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra value symbol 'extra'")),
        "expected extra resolver symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_imports_when_ast_imports_are_present() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
{ io, math } = std
main = () i32 { 0 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver imports should fail when AST imports are present");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra import symbol 'math'")),
        "expected extra resolver import diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_modules_when_ast_imports_are_present() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
{ io } = std
{ helper } = other
main = () i32 { 0 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver modules should fail when AST imports are present");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra module symbol 'other'")),
        "expected extra resolver module diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_method_receiver_type() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point.label = () str { "point" }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Type, "Point");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing receiver type resolver symbol should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing type symbol 'Point'")),
        "expected missing method receiver type symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_method_signature() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "Box.get",
        Some(vec!["Box<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver method signature mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Box.get' has parameter types '(Box<i32>)', expected '(Box<T>)'"
        )),
        "expected resolver method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_method_function_type_signature() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.map<T> = (self: Box<T>, callback: (T) T) (T) T {
    callback
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "Box.map",
        Some(vec!["Box<T>".to_string(), "T".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver method function type mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Box.map' has parameter types '(Box<T>, T)', expected '(Box<T>, (T) T)'"
            )),
            "expected resolver method function type diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_uses_resolver_import_bindings() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 {
    io.println("ok")
    0
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));

    let mut tc = TypeChecker::new();
    tc.check_program_with_symbols(&program, &symbols)
        .expect("resolver import symbols should seed typechecker imports");

    assert!(tc.is_root_std_import("io"));
}

#[test]
fn check_program_with_symbols_validates_stripped_resolver_import_sources() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Import, "io", None);
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver imports without sources should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver import symbol 'io' has source 'unknown', expected a module source"
        )),
        "expected stripped resolver import source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_stripped_resolver_import_visibility() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Import, "io", true);
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver import visibility should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has visibility public, expected private")),
        "expected stripped resolver import visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_stripped_resolver_import_modules() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Module, "std");
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver imports should require source module symbols");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing module symbol 'std'")),
        "expected stripped resolver import module diagnostic, got {err:?}"
    );
}

#[test]
fn check_module_graph_entry_uses_graph_import_bindings() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n")
        .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 { add(1, 2) }\n",
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");
    let entry = graph.module(graph.entry).expect("entry module");
    assert!(
        !entry
            .program
            .declarations
            .iter()
            .any(|decl| decl.name() == Some("add")),
        "graph entry should not merge imported declarations"
    );

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed imported signatures");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "main"));
    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "add"));
}

#[test]
fn check_module_graph_entry_seeds_imported_function_type_signatures() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let callbacks_path = tmp.path().join("callbacks.zen");
    std::fs::write(
        &callbacks_path,
        "pub apply = (callback: (i32) i32, value: i32) i32 { value }\n",
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ apply } = callbacks

main = () i32 {
    callback = (value: i32) i32 { value }
    apply(callback, 1)
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    tc.check_module_graph_entry(&graph)
        .expect("graph import bindings should seed function-typed signatures");
}

#[test]
fn check_module_graph_entry_specializes_imported_generic_functions() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let identity_path = tmp.path().join("identity.zen");
    std::fs::write(&identity_path, "pub id<T> = (value: T) T { value }\n")
        .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        "{ id } = identity\n\nmain = () i32 { id<i32>(1) }\n",
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed generic templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "id_i32"));
}

#[test]
fn check_module_graph_entry_specializes_imported_generic_enums() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let option_path = tmp.path().join("option.zen");
    std::fs::write(
        &option_path,
        r#"pub Option<T>:
    None,
    Some(T)

pub Result<T, E>:
    Ok(T),
    Err(E)
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Option, Result } = option

main = () i32 {
    maybe = Option<i32>.Some(7)
    result = Result<i32, str>.Ok(9)
    0
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed generic enum templates");

    assert!(typed.types.iter().any(|ty| ty.name == "Option_i32"));
    assert!(typed.types.iter().any(|ty| ty.name == "Result_i32_str"));
}

#[test]
fn check_module_graph_entry_seeds_public_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

pub Point.value = (self: Point) i32 {
    self.x
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    tc.check_module_graph_entry(&graph)
        .expect("imported public type should seed its public methods");
}

#[test]
fn check_module_graph_entry_does_not_seed_private_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

Point.value = (self: Point) i32 {
    self.x
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let err = TypeChecker::new()
        .check_module_graph_entry(&graph)
        .expect_err("private imported methods should not be seeded");

    assert!(
        err.iter()
            .any(|d| d.message.contains("type `Point` has no method `value`")),
        "expected private imported method diagnostic, got {err:?}"
    );
}

#[test]
fn check_module_graph_entry_specializes_public_generic_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

pub Point.keep<T> = (self: Point, value: T) T {
    value
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.keep<i32>(1)
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("imported public type should seed public generic method templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "Point.keep_i32"));
}

#[test]
fn check_program_with_symbols_validates_resolver_import_sources() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Import, "io", Some("other".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import source mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has source 'other', expected 'std'")),
        "expected resolver import source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_import_visibility() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Import, "io", true);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has visibility public, expected private")),
        "expected resolver import visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_import_absent_declaration_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Import, "io", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has parameter count metadata, expected none")),
        "expected resolver import parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has return type metadata, expected none")),
        "expected resolver import return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_import_absent_type_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Import, "io", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Import, "io", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Import, "io", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_type_parameter_names_for_test(Namespace::Import, "io", Some(vec!["T".to_string()]));
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Import,
        "io",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Import,
        "io",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Import,
        "io",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Import, "io", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Import, "io", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Import,
        "io",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Import, "io", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Import,
        "io",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import type metadata should fail");

    for expected in [
        "resolver import symbol 'io' has parameter names metadata, expected none",
        "resolver import symbol 'io' has parameter types metadata, expected none",
        "resolver import symbol 'io' has typed parameter types metadata, expected none",
        "resolver import symbol 'io' has typed return type metadata, expected none",
        "resolver import symbol 'io' has type parameter count metadata, expected none",
        "resolver import symbol 'io' has type parameter names metadata, expected none",
        "resolver import symbol 'io' has type parameter bounds metadata, expected none",
        "resolver import symbol 'io' has typed type parameter bound refs metadata, expected none",
        "resolver import symbol 'io' has field count metadata, expected none",
        "resolver import symbol 'io' has field types metadata, expected none",
        "resolver import symbol 'io' has typed field types metadata, expected none",
        "resolver import symbol 'io' has variant names metadata, expected none",
        "resolver import symbol 'io' has variant owner metadata, expected none",
        "resolver import symbol 'io' has variant payload count metadata, expected none",
        "resolver import symbol 'io' has variant payload type metadata, expected none",
        "resolver import symbol 'io' has typed variant payload type metadata, expected none",
        "resolver import symbol 'io' has behavior methods metadata, expected none",
        "resolver import symbol 'io' has typed behavior methods metadata, expected none",
        "resolver import symbol 'io' has behavior parents metadata, expected none",
        "resolver import symbol 'io' has typed behavior parents metadata, expected none",
        "resolver import symbol 'io' has behavior impls metadata, expected none",
        "resolver import symbol 'io' has typed behavior impls metadata, expected none",
        "resolver import symbol 'io' has behavior requires metadata, expected none",
        "resolver import symbol 'io' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver import metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_import_and_module_absent_mutability() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_mutability_for_test(Namespace::Import, "io", Some(true));
    symbols.set_mutability_for_test(Namespace::Module, "std", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import/module mutability metadata should fail");

    for expected in [
        "resolver import symbol 'io' has mutability metadata, expected none",
        "resolver module symbol 'std' has mutability metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver import/module mutability diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_module_symbols() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Module, "std", true);
    symbols.set_import_source_for_test(Namespace::Module, "std", Some("other".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has visibility public, expected private")),
        "expected resolver module visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has source 'other', expected none")),
        "expected resolver module source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_module_absent_declaration_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Module, "std", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has parameter count metadata, expected none")),
        "expected resolver module parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has return type metadata, expected none")),
        "expected resolver module return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_module_absent_type_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Module, "std", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Module, "std", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Module, "std", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_type_parameter_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["T".to_string()]),
    );
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Module,
        "std",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Module,
        "std",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Module,
        "std",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Module, "std", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Module, "std", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Module,
        "std",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Module, "std", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Module,
        "std",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module type metadata should fail");

    for expected in [
        "resolver module symbol 'std' has parameter names metadata, expected none",
        "resolver module symbol 'std' has parameter types metadata, expected none",
        "resolver module symbol 'std' has typed parameter types metadata, expected none",
        "resolver module symbol 'std' has typed return type metadata, expected none",
        "resolver module symbol 'std' has type parameter count metadata, expected none",
        "resolver module symbol 'std' has type parameter names metadata, expected none",
        "resolver module symbol 'std' has type parameter bounds metadata, expected none",
        "resolver module symbol 'std' has typed type parameter bound refs metadata, expected none",
        "resolver module symbol 'std' has field count metadata, expected none",
        "resolver module symbol 'std' has field types metadata, expected none",
        "resolver module symbol 'std' has typed field types metadata, expected none",
        "resolver module symbol 'std' has variant names metadata, expected none",
        "resolver module symbol 'std' has variant owner metadata, expected none",
        "resolver module symbol 'std' has variant payload count metadata, expected none",
        "resolver module symbol 'std' has variant payload type metadata, expected none",
        "resolver module symbol 'std' has typed variant payload type metadata, expected none",
        "resolver module symbol 'std' has behavior methods metadata, expected none",
        "resolver module symbol 'std' has typed behavior methods metadata, expected none",
        "resolver module symbol 'std' has behavior parents metadata, expected none",
        "resolver module symbol 'std' has typed behavior parents metadata, expected none",
        "resolver module symbol 'std' has behavior impls metadata, expected none",
        "resolver module symbol 'std' has typed behavior impls metadata, expected none",
        "resolver module symbol 'std' has behavior requires metadata, expected none",
        "resolver module symbol 'std' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver module metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_requires_resolver_impl_methods() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { "point" }
}
"#,
    );
    let symbols = SymbolTable::default();
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing impl method resolver symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing value symbol 'Point.stringify'")),
        "expected missing impl method symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_impl_method_signature() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(
        Namespace::Value,
        "Point.stringify",
        Some("i32".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver impl method signature mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Point.stringify' has return type 'i32', expected 'str'"
        )),
        "expected resolver impl method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_impl_function_type_signature() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}

Point: { x: i32 }

Point.implements(Mapper) {
    map = (value: Point, callback: (i32) i32) (i32) i32 {
        callback
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "Point.map", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver impl method function type mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Point.map' has return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver impl method function type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_impl_method_body_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str {
        label = "point"
        label
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "label");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver impl method body local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'label'")),
        "expected missing resolver impl method body local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_enum_variants() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Variant, "Some");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver enum variant symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing variant symbol 'Some'")),
        "expected missing enum variant symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_arity() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Value, "add", Some(1));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function arity mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'add' has parameter count 1, expected 2")),
        "expected resolver function arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_parameter_types() {
    let program = parse_program(
        r#"
add = (a: i32, b: f64) f64 { b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "add",
        Some(vec!["i32".to_string(), "i32".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function parameter type mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'add' has parameter types '(i32, i32)', expected '(i32, f64)'"
        )),
        "expected resolver function parameter type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_metadata() {
    let program = parse_program(
        r#"
apply = (callback: (i32) i32, value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "apply",
        Some(vec!["i32".to_string(), "i32".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function type parameter metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has parameter types '(i32, i32)', expected '((i32) i32, i32)'"
            )),
            "expected resolver function type parameter metadata diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_parameter_names() {
    let program = parse_program(
        r#"
add = (a: i32, b: f64) f64 { b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(
        Namespace::Value,
        "add",
        Some(vec!["a".to_string(), "other".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function parameter name mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'add' has parameter names '(a, other)', expected '(a, b)'"
        )),
        "expected resolver function parameter name diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_parameter_locals() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "a");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver parameter local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'a'")),
        "expected missing resolver parameter local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_parameter_local_mutability() {
    let program = parse_program(
        r#"
add = (mut a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("a", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver parameter local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has mutability immutable, expected mutable")),
        "expected resolver parameter local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_visibility_and_source() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Local, "a", true);
    symbols.set_import_source_for_test(Namespace::Local, "a", Some("std".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local visibility/source mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has visibility public, expected private")),
        "expected resolver local visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has source 'std', expected none")),
        "expected resolver local source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_absent_declaration_metadata() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has parameter count metadata, expected none")),
        "expected resolver local parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has return type metadata, expected none")),
        "expected resolver local return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_absent_type_metadata() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Local, "a", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(Namespace::Local, "a", Some(vec!["i32".to_string()]));
    symbols.set_parameter_types_for_test(Namespace::Local, "a", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Local, "a", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_type_parameter_names_for_test(Namespace::Local, "a", Some(vec!["T".to_string()]));
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Local,
        "a",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Local,
        "a",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Local,
        "a",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Local, "a", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Local, "a", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_variant_payload_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
    symbols.set_variant_payload_type_for_test(Namespace::Local, "a", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Local,
        "a",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Local,
        "a",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(Namespace::Local, "a", Some(vec!["Json".to_string()]));
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Local,
        "a",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local type metadata should fail");

    for expected in [
        "resolver local symbol 'a' has parameter names metadata, expected none",
        "resolver local symbol 'a' has parameter types metadata, expected none",
        "resolver local symbol 'a' has typed parameter types metadata, expected none",
        "resolver local symbol 'a' has typed return type metadata, expected none",
        "resolver local symbol 'a' has type parameter count metadata, expected none",
        "resolver local symbol 'a' has type parameter names metadata, expected none",
        "resolver local symbol 'a' has type parameter bounds metadata, expected none",
        "resolver local symbol 'a' has typed type parameter bound refs metadata, expected none",
        "resolver local symbol 'a' has field count metadata, expected none",
        "resolver local symbol 'a' has field types metadata, expected none",
        "resolver local symbol 'a' has typed field types metadata, expected none",
        "resolver local symbol 'a' has variant names metadata, expected none",
        "resolver local symbol 'a' has variant owner metadata, expected none",
        "resolver local symbol 'a' has variant payload count metadata, expected none",
        "resolver local symbol 'a' has variant payload type metadata, expected none",
        "resolver local symbol 'a' has typed variant payload type metadata, expected none",
        "resolver local symbol 'a' has behavior methods metadata, expected none",
        "resolver local symbol 'a' has typed behavior methods metadata, expected none",
        "resolver local symbol 'a' has behavior parents metadata, expected none",
        "resolver local symbol 'a' has typed behavior parents metadata, expected none",
        "resolver local symbol 'a' has behavior impls metadata, expected none",
        "resolver local symbol 'a' has typed behavior impls metadata, expected none",
        "resolver local symbol 'a' has behavior requires metadata, expected none",
        "resolver local symbol 'a' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver local metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_requires_resolver_var_decl_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    value = 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver var local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver var local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_var_decl_local_mutability() {
    let program = parse_program(
        r#"
main = () i32 {
    value ::= 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("value", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver var local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability immutable, expected mutable")),
        "expected resolver var local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    0
}
"#,
    );
    let symbols_program = parse_program(
        r#"
main = () i32 {
    value = 1
    0
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra local symbol 'value'")),
        "expected extra resolver local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_mutability_by_scope() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let inner_scope = symbols
        .symbols()
        .iter()
        .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
        .map(|symbol| symbol.scope_id)
        .max()
        .expect("inner value local");
    symbols.set_local_mutability_in_scope_for_test("value", inner_scope, Some(true));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver scoped local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability mutable, expected immutable")),
        "expected scoped resolver local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_pattern_locals() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    value ?
        | Some(inner) { inner }
        | None { 0 }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "inner");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver pattern local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'inner'")),
        "expected missing resolver pattern local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_top_level_expr_locals() {
    let program = parse_program(
        r#"
value := 1
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver top-level expr local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver top-level expr local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_closure_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (input: i32) i32 {
        inner = input
        inner
    }
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "inner");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver closure local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'inner'")),
        "expected missing resolver closure local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_closure_parameter_mutability() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (mut input: i32) i32 {
        input = input + 1
        input
    }
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("input", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver closure parameter mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'input' has mutability immutable, expected mutable")),
        "expected resolver closure parameter mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_struct_field_default_locals() {
    let program = parse_program(
        r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver struct field default local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver struct field default local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_behavior_default_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str {
        value = "{}"
        value
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver behavior default local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver behavior default local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_visibility() {
    let program = parse_program(
        r#"
pub exported = () i32 { 1 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Value, "exported", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'exported' has visibility private, expected public")),
        "expected resolver function visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_return_type() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "main", Some("bool".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function return mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'main' has return type 'bool', expected 'i32'")),
        "expected resolver function return diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_return_metadata() {
    let program = parse_program(
        r#"
factory = () (i32) i32 {
    (value: i32) i32 { value }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "factory", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function type return metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'factory' has return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver function type return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_typed_signature_metadata() {
    let program = parse_program(
        r#"
apply = (callback: (i32) i32) (i32) i32 {
    callback
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_types_for_test(Namespace::Value, "apply", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Value, "apply", Some(AstType::I32));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed function signature metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
            )),
            "expected resolver typed parameter diagnostic, got {err:?}"
        );
    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver typed return diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_counts() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_count_for_test(Namespace::Value, "identity", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic arity mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'identity' has type parameter count 0, expected 1")),
        "expected resolver function generic arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_names() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_names_for_test(
        Namespace::Value,
        "identity",
        Some(vec!["U".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic parameter name mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
        )),
        "expected resolver function generic parameter name diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
encode<T: Json> = (value: T) str { "encoded" }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Value,
        "encode",
        Some(vec![("T".to_string(), "Other".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'encode' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver function generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs() {
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
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Value,
        "identity",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: vec![AstType::Str],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic bound ref mismatch should fail");

    let expected = "resolver value symbol 'identity' has type parameter bound refs '(T: Json<str>)', expected '(T: Json<T>)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver function generic bound ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_absent_declaration_metadata() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Value, "main", Some("std".to_string()));
    symbols.set_field_count_for_test(Namespace::Value, "main", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Value,
        "main",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Value,
        "main",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Value, "main", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Value,
        "main",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Value, "main", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Value,
        "main",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::Named("Self".to_string())],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function declaration metadata should fail");

    for expected in [
        "resolver value symbol 'main' has source 'std', expected none",
        "resolver value symbol 'main' has field count metadata, expected none",
        "resolver value symbol 'main' has field types metadata, expected none",
        "resolver value symbol 'main' has typed field types metadata, expected none",
        "resolver value symbol 'main' has variant names metadata, expected none",
        "resolver value symbol 'main' has variant payload type metadata, expected none",
        "resolver value symbol 'main' has typed variant payload type metadata, expected none",
        "resolver value symbol 'main' has behavior methods metadata, expected none",
        "resolver value symbol 'main' has typed behavior methods metadata, expected none",
        "resolver value symbol 'main' has behavior parents metadata, expected none",
        "resolver value symbol 'main' has typed behavior parents metadata, expected none",
        "resolver value symbol 'main' has behavior impls metadata, expected none",
        "resolver value symbol 'main' has typed behavior impls metadata, expected none",
        "resolver value symbol 'main' has behavior requires metadata, expected none",
        "resolver value symbol 'main' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver function declaration metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_counts() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_count_for_test(Namespace::Type, "Box", Some(0));
    symbols.set_type_parameter_count_for_test(Namespace::Behavior, "Serializable", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic arity mismatches should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has type parameter count 0, expected 1")),
        "expected resolver type generic arity diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver behavior symbol 'Serializable' has type parameter count 0, expected 1"
        )),
        "expected resolver behavior generic arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_names() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_names_for_test(Namespace::Type, "Box", Some(vec!["U".to_string()]));
    symbols.set_type_parameter_names_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec!["U".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic parameter name mismatches should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has type parameter names '(U)', expected '(T)'")),
        "expected resolver type generic parameter name diagnostic, got {err:?}"
    );
    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver behavior generic parameter name diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_visibility() {
    let program = parse_program(
        r#"
pub Box<T>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Type, "Box", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has visibility private, expected public")),
        "expected resolver type visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_visibility() {
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
    symbols.set_public_for_test(Namespace::Behavior, "Json", true);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver behavior symbol 'Json' has visibility public, expected private")),
        "expected resolver behavior visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
Box<T: Json>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Type,
        "Box",
        Some(vec![("T".to_string(), "Other".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver type generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec![("T".to_string(), "Json<i32>".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter bounds '(T: Json<i32>)', expected '(T: Json<T>)'"
            )),
            "expected resolver behavior generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_like_absent_value_metadata() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Type, "Box", Some("std".to_string()));
    symbols.set_parameter_count_for_test(Namespace::Type, "Box", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Type, "Box", Some("i32".to_string()));
    symbols.set_return_type_for_test(Namespace::Type, "Box", Some(AstType::I32));
    symbols.set_import_source_for_test(Namespace::Behavior, "Json", Some("std".to_string()));
    symbols.set_parameter_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["value".to_string()]),
    );
    symbols.set_parameter_type_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Self".to_string()]),
    );
    symbols.set_parameter_types_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![AstType::SelfType]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type-like value metadata should fail");

    for expected in [
        "resolver type symbol 'Box' has source 'std', expected none",
        "resolver type symbol 'Box' has parameter count metadata, expected none",
        "resolver type symbol 'Box' has return type metadata, expected none",
        "resolver type symbol 'Box' has typed return type metadata, expected none",
        "resolver behavior symbol 'Json' has source 'std', expected none",
        "resolver behavior symbol 'Json' has parameter names metadata, expected none",
        "resolver behavior symbol 'Json' has parameter types metadata, expected none",
        "resolver behavior symbol 'Json' has typed parameter types metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver type-like value metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_method_signatures() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self, i32) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string(), "bool".to_string()],
            "str".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior method signature mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has methods '(encode(Self, bool) str)', expected '(encode(Self, i32) str)'"
            )),
            "expected resolver behavior method signature diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![(
            "map".to_string(),
            vec!["Self".to_string(), "i32".to_string()],
            "(i32) i32".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior function type method signature mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, i32) (i32) i32)', expected '(map(Self, (i32) i32) (i32) i32)'"
            )),
            "expected resolver behavior function type method signature diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_method_types() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "map".to_string(),
            parameter_names: vec!["__arg0".to_string(), "__arg1".to_string()],
            parameter_types: vec![AstType::SelfType, AstType::I32],
            return_type: AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            },
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed behavior method metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has typed methods '(map(__arg0: Self, __arg1: i32) (i32) i32)', expected '(map(__arg0: Self, __arg1: (i32) i32) (i32) i32)'"
            )),
            "expected resolver typed behavior method diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_method_signatures() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior method signature mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Json' has methods '(encode(Self) str)', expected '(encode(Self) T)'"
            )),
            "expected resolver generic behavior method signature diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures()
{
    let program = parse_program(
        r#"
Mapper<T>: behavior {
    map: (Self, (T) T) (T) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![(
            "map".to_string(),
            vec!["Self".to_string(), "T".to_string()],
            "(T) T".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior function type method mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, T) (T) T)', expected '(map(Self, (T) T) (T) T)'"
            )),
            "expected resolver generic behavior function type method diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_absent_type_metadata() {
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
    symbols.set_field_count_for_test(Namespace::Behavior, "Json", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Behavior, "Json", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Behavior,
        "Json",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Behavior, "Json", Some(AstType::I32));
    symbols.set_behavior_impl_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Debug".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Debug".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior type metadata should fail");

    for expected in [
        "resolver behavior symbol 'Json' has field count metadata, expected none",
        "resolver behavior symbol 'Json' has field types metadata, expected none",
        "resolver behavior symbol 'Json' has typed field types metadata, expected none",
        "resolver behavior symbol 'Json' has variant names metadata, expected none",
        "resolver behavior symbol 'Json' has variant payload type metadata, expected none",
        "resolver behavior symbol 'Json' has typed variant payload type metadata, expected none",
        "resolver behavior symbol 'Json' has behavior impls metadata, expected none",
        "resolver behavior symbol 'Json' has typed behavior impls metadata, expected none",
        "resolver behavior symbol 'Json' has behavior requires metadata, expected none",
        "resolver behavior symbol 'Json' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver behavior type metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_parent_names() {
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
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_names_for_test(Namespace::Behavior, "PrettyJson", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior parent metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver behavior symbol 'PrettyJson' has parents 'none', expected to include 'Json'"
        )),
        "expected resolver behavior parent metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_parent_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_names_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior parent metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'PrettyJson' has parents 'Json<i32>', expected to include 'Json<str>'"
            )),
            "expected resolver generic behavior parent metadata diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_parent_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior parent ref mismatch should fail");

    let expected =
            "resolver behavior symbol 'PrettyJson' has parent refs 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior parent ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_accepts_resolver_behavior_parent_child_type_param_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    tc.check_program_with_symbols(&program, &symbols)
        .expect("resolver parent type arg using child type parameter should validate");
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_names_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior parent metadata should fail");

    let expected =
        "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior parent metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior parent ref metadata should fail");

    let expected =
        "resolver behavior symbol 'PrettyJson' has parent refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior parent ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_impl_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_names_for_test(Namespace::Type, "Point", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior impl metadata mismatch should fail");

    let expected =
        "resolver type symbol 'Point' has behavior impls 'none', expected to include 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver behavior impl metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_impl_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior impl metadata mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior impls 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior impl metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_impl_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior impl ref mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior impl refs 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior impl ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_required_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(Namespace::Type, "Point", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior requires metadata mismatch should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires 'none', expected to include 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_required_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior requires metadata mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior requires 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_required_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior requires ref mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior requires refs 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior requires ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior impl metadata should fail");

    let expected = "resolver type symbol 'Point' has behavior impls 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior impl metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior impl ref metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior impl refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior impl ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_required_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior requires metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_required_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior requires ref metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior requires ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_counts() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_count_for_test(Namespace::Type, "Point", Some(1));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Point' has field count 1, expected 2")),
        "expected resolver struct field count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32, y: f64 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field type mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Point' has fields '(x: i32, y: i32)', expected '(x: i32, y: f64)'"
            )),
            "expected resolver struct field type diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_function_type_fields() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct function type field mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver struct function type field diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_typed_field_metadata() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed struct field metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver typed struct field diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_struct_field_types() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Box",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic struct field mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver type symbol 'Box' has fields '(value: i32)', expected '(value: T)'"
        )),
        "expected resolver generic struct field diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_and_enum_absent_kind_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Point", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Type,
        "Point",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Type, "Point", Some(AstType::I32));
    symbols.set_field_count_for_test(Namespace::Type, "Option", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct/enum kind metadata should fail");

    for expected in [
        "resolver type symbol 'Point' has variant names metadata, expected none",
        "resolver type symbol 'Point' has variant payload type metadata, expected none",
        "resolver type symbol 'Point' has typed variant payload type metadata, expected none",
        "resolver type symbol 'Option' has field count metadata, expected none",
        "resolver type symbol 'Option' has field types metadata, expected none",
        "resolver type symbol 'Option' has typed field types metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver struct/enum kind metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_counts() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_count_for_test(Namespace::Variant, "Some", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload count 0, expected 1")),
        "expected resolver enum variant payload count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_visibility() {
    let program = parse_program(
        r#"
pub Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Variant, "Some", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has visibility private, expected public")),
        "expected resolver enum variant visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_types() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Some",
        Some("bool".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload type mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload type 'bool', expected 'i32'")),
        "expected resolver enum variant payload type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("i32".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_typed_payload_metadata() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_for_test(Namespace::Variant, "Wrap", Some(AstType::I32));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed enum payload metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver typed enum payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback<T>: Wrap((T) T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Wrap' has payload type 'T', expected '(T) T'")),
        "expected resolver generic enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_payload_types() {
    let program = parse_program(
        r#"
Result<T, E>: Ok(T), Err(E)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Err",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Err' has payload type 'T', expected 'E'")),
        "expected resolver generic enum payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_variant_absent_other_metadata() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Variant, "Some", Some("std".to_string()));
    symbols.set_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_parameter_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["value".to_string()]),
    );
    symbols.set_parameter_type_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Variant, "Some", Some(vec![AstType::I32]));
    symbols.set_return_type_name_for_test(Namespace::Variant, "Some", Some("i32".to_string()));
    symbols.set_return_type_for_test(Namespace::Variant, "Some", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_type_parameter_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["T".to_string()]),
    );
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Variant, "Some", Some(vec!["Other".to_string()]));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver variant non-variant metadata should fail");

    for expected in [
            "resolver variant symbol 'Some' has source 'std', expected none",
            "resolver variant symbol 'Some' has parameter count metadata, expected none",
            "resolver variant symbol 'Some' has parameter names metadata, expected none",
            "resolver variant symbol 'Some' has parameter types metadata, expected none",
            "resolver variant symbol 'Some' has typed parameter types metadata, expected none",
            "resolver variant symbol 'Some' has return type metadata, expected none",
            "resolver variant symbol 'Some' has typed return type metadata, expected none",
            "resolver variant symbol 'Some' has type parameter count metadata, expected none",
            "resolver variant symbol 'Some' has type parameter names metadata, expected none",
            "resolver variant symbol 'Some' has type parameter bounds metadata, expected none",
            "resolver variant symbol 'Some' has typed type parameter bound refs metadata, expected none",
            "resolver variant symbol 'Some' has field count metadata, expected none",
            "resolver variant symbol 'Some' has field types metadata, expected none",
            "resolver variant symbol 'Some' has typed field types metadata, expected none",
            "resolver variant symbol 'Some' has variant names metadata, expected none",
            "resolver variant symbol 'Some' has behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has behavior requires metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver variant metadata diagnostic '{expected}', got {err:?}"
            );
        }
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", Some(vec!["Some".to_string()]));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant names mismatch should fail");

    let expected = "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant names diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_owner_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_owner_name_for_test(Namespace::Variant, "Some", Some("Result".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant owner mismatch should fail");

    let expected = "resolver variant symbol 'Some' has owner 'Result', expected 'Option'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant owner diagnostic, got {err:?}"
    );
}

#[test]
fn binary_op_types() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.check_binary_op(BinaryOp::Add, &Type::I32, &Type::I32, &Span::dummy())
            .unwrap(),
        Type::I32
    );
    assert_eq!(
        tc.check_binary_op(BinaryOp::Eq, &Type::I32, &Type::I32, &Span::dummy())
            .unwrap(),
        Type::Bool
    );
    assert_eq!(
        tc.check_binary_op(BinaryOp::And, &Type::Bool, &Type::Bool, &Span::dummy())
            .unwrap(),
        Type::Bool
    );
}

#[test]
fn binary_op_type_mismatch() {
    let tc = TypeChecker::new();
    // Arithmetic on non-numeric type
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::I32, &Type::Str, &Span::dummy())
        .is_err());
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::Bool, &Type::I32, &Span::dummy())
        .is_err());
    // Logical op on non-bool
    assert!(tc
        .check_binary_op(BinaryOp::And, &Type::I32, &Type::Bool, &Span::dummy())
        .is_err());
    // Unknown is permissive (error recovery)
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::Unknown, &Type::Str, &Span::dummy())
        .is_ok());
}

#[test]
fn binary_op_mixed_numeric_width_requires_cast() {
    let tc = TypeChecker::new();
    let err = tc
        .check_binary_op(BinaryOp::Add, &Type::I32, &Type::I64, &Span::dummy())
        .expect_err("mixed integer arithmetic should fail");
    assert!(
        err.message
            .contains("arithmetic operands must have the same type"),
        "expected mixed numeric diagnostic, got {err:?}"
    );

    let err = tc
        .check_binary_op(BinaryOp::Mul, &Type::F32, &Type::F64, &Span::dummy())
        .expect_err("mixed float arithmetic should fail");
    assert!(
        err.message
            .contains("arithmetic operands must have the same type"),
        "expected mixed numeric diagnostic, got {err:?}"
    );
}

#[test]
fn unknown_function_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "main".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: vec![ast::Statement::Expression {
                    expr: Expression::FunctionCall {
                        name: "nonexistent".into(),
                        module: None,
                        type_args: Vec::new(),
                        args: Vec::new(),
                        span: Span::dummy(),
                    },
                    span: Span::dummy(),
                }],
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    let result = tc.check_program(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("undefined function")));
}

#[test]
fn return_type_mismatch_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "foo".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::I32),
            body: Expression::Block {
                statements: Vec::new(),
                expr: Some(Box::new(Expression::Return {
                    value: Some(Box::new(Expression::StringLiteral {
                        value: "hello".into(),
                        span: Span::dummy(),
                    })),
                    span: Span::dummy(),
                })),
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    let result = tc.check_program(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("return type mismatch")));
}

#[test]
fn function_call_wrong_arity_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "add".into(),
                type_params: Vec::new(),
                params: vec![
                    ast::Param {
                        name: "a".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    },
                    ast::Param {
                        name: "b".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    },
                ],
                return_type: Some(AstType::I32),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: Some(Box::new(Expression::Return {
                        value: Some(Box::new(Expression::Identifier {
                            name: "a".into(),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    })),
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "add".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: vec![Expression::IntLiteral {
                                value: 1,
                                span: Span::dummy(),
                            }],
                            span: Span::dummy(),
                        },
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("wrong arity should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("function `add` expects 2 arguments, found 1")),
        "expected arity diagnostic, got {errors:?}"
    );
}

#[test]
fn function_call_argument_type_mismatch_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "takes_i32".into(),
                type_params: Vec::new(),
                params: vec![ast::Param {
                    name: "value".into(),
                    ty: AstType::I32,
                    mutable: false,
                    span: Span::dummy(),
                }],
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "takes_i32".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: vec![Expression::StringLiteral {
                                value: "bad".into(),
                                span: Span::dummy(),
                            }],
                            span: Span::dummy(),
                        },
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("argument type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("argument 1 for `takes_i32` expects `i32`, found `str`")),
        "expected argument type diagnostic, got {errors:?}"
    );
}

#[test]
fn struct_literal_missing_field_is_error() {
    use crate::ast::declarations::StructField;
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Point".into(),
                type_params: Vec::new(),
                fields: vec![
                    StructField {
                        name: "x".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    },
                    StructField {
                        name: "y".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    },
                ],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "p".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Point".into(),
                            type_args: Vec::new(),
                            fields: vec![(
                                "x".into(),
                                Expression::IntLiteral {
                                    value: 1,
                                    span: Span::dummy(),
                                },
                            )],
                            span: Span::dummy(),
                        },
                        mutable: false,
                        constant: false,
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("missing struct field should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("missing field `y` for struct `Point`")),
        "expected missing field diagnostic, got {errors:?}"
    );
}

#[test]
fn struct_literal_uses_default_for_omitted_field() {
    use crate::ast::declarations::StructField;
    use crate::ast::typed::{TypedExprKind, TypedStatementKind};
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Point".into(),
                type_params: Vec::new(),
                fields: vec![
                    StructField {
                        name: "x".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    },
                    StructField {
                        name: "y".into(),
                        ty: AstType::I32,
                        default: Some(Expression::IntLiteral {
                            value: 2,
                            span: Span::dummy(),
                        }),
                        mutable: false,
                        span: Span::dummy(),
                    },
                ],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "p".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Point".into(),
                            type_args: Vec::new(),
                            fields: vec![(
                                "x".into(),
                                Expression::IntLiteral {
                                    value: 1,
                                    span: Span::dummy(),
                                },
                            )],
                            span: Span::dummy(),
                        },
                        mutable: false,
                        constant: false,
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let typed = tc
        .check_program(&program)
        .expect("defaulted struct field may be omitted");
    let TypedStatementKind::VarDecl { value, .. } = &typed.functions[0].body.statements[0].kind
    else {
        panic!("expected var decl");
    };
    let TypedExprKind::StructLiteral { fields, .. } = &value.kind else {
        panic!("expected struct literal");
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[1].0, "y");
    assert!(matches!(fields[1].1.kind, TypedExprKind::IntLiteral(2)));
}

#[test]
fn generic_struct_literal_uses_substituted_default_for_omitted_field() {
    use crate::ast::declarations::{StructField, TypeParam};
    use crate::ast::typed::{TypedExprKind, TypedStatementKind};
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Box".into(),
                type_params: vec![TypeParam {
                    name: "T".into(),
                    constraint: None,
                    constraint_type_args: Vec::new(),
                    span: Span::dummy(),
                }],
                fields: vec![StructField {
                    name: "value".into(),
                    ty: AstType::Named("T".into()),
                    default: Some(Expression::Block {
                        statements: vec![Statement::VarDecl {
                            name: "same".into(),
                            ty: Some(AstType::Named("T".into())),
                            value: Expression::StringLiteral {
                                value: "fallback".into(),
                                span: Span::dummy(),
                            },
                            mutable: false,
                            constant: false,
                            span: Span::dummy(),
                        }],
                        expr: Some(Box::new(Expression::Identifier {
                            name: "same".into(),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    }),
                    mutable: false,
                    span: Span::dummy(),
                }],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "box".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Box".into(),
                            type_args: vec![AstType::Str],
                            fields: Vec::new(),
                            span: Span::dummy(),
                        },
                        mutable: false,
                        constant: false,
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let typed = tc
        .check_program(&program)
        .expect("generic defaulted struct field may be omitted");
    let TypedStatementKind::VarDecl { value, .. } = &typed.functions[0].body.statements[0].kind
    else {
        panic!("expected var decl");
    };
    let TypedExprKind::StructLiteral { fields, .. } = &value.kind else {
        panic!("expected struct literal");
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "value");
    assert_eq!(fields[0].1.ty, Type::Str);
}

#[test]
fn struct_literal_field_type_mismatch_is_error() {
    use crate::ast::declarations::StructField;
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Point".into(),
                type_params: Vec::new(),
                fields: vec![StructField {
                    name: "x".into(),
                    ty: AstType::I32,
                    default: None,
                    mutable: false,
                    span: Span::dummy(),
                }],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "p".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Point".into(),
                            type_args: Vec::new(),
                            fields: vec![(
                                "x".into(),
                                Expression::StringLiteral {
                                    value: "bad".into(),
                                    span: Span::dummy(),
                                },
                            )],
                            span: Span::dummy(),
                        },
                        mutable: false,
                        constant: false,
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("struct field type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("field `x` for struct `Point` expects `i32`, found `str`")),
        "expected field type diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_variant_unknown_variant_is_error() {
    let program = parse_program(
        r#"
Status: Ok, Err

main = () void {
    value = Status.Pending
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown enum variant should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
        "expected unknown variant diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_variant_payload_type_mismatch_is_error() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () void {
    value = Maybe.Some("bad")
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum payload type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("payload for enum variant `Maybe.Some` expects `i32`, found `str`")),
        "expected payload type diagnostic, got {errors:?}"
    );
}

#[test]
fn assignment_to_immutable_binding_is_error() {
    let program = parse_program(
        r#"
main = () void {
    x = 1
    x = 2
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("immutable assignment should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot assign to immutable variable `x`")),
        "expected immutable assignment diagnostic, got {errors:?}"
    );
}

#[test]
fn assignment_to_mutable_closure_parameter_is_allowed() {
    let program = parse_program(
        r#"
main = () void {
    mapper = (mut input: i32) i32 {
        input = input + 1
        input
    }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("mutable closure parameter assignment should pass");
}

#[test]
fn assignment_type_mismatch_is_error() {
    let program = parse_program(
        r#"
main = () void {
    x ::= 1
    x = "bad"
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("assignment type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("assignment to `x` expects `i32`, found `str`")),
        "expected assignment type diagnostic, got {errors:?}"
    );
}

#[test]
fn invalid_field_access_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () void {
    p = Point { x: 1 }
    y = p.y
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("invalid field access should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("type `Point` has no field `y`")),
        "expected invalid field diagnostic, got {errors:?}"
    );
}

#[test]
fn implicit_integer_width_conversion_is_error() {
    let program = parse_program(
        r#"
take_i64 = (value: i64) void {}

main = () void {
    x: i32 = 1
    take_i64(x)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("implicit integer conversion should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("argument 1 for `take_i64` expects `i64`, found `i32`")),
        "expected integer conversion diagnostic, got {errors:?}"
    );
}

#[test]
fn implicit_float_width_conversion_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "take_f32".into(),
            type_params: Vec::new(),
            params: vec![ast::Param {
                name: "value".into(),
                ty: AstType::F32,
                mutable: false,
                span: Span::dummy(),
            }],
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: Vec::new(),
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    tc.collect_declarations(&program.declarations);

    let expected = tc.functions["take_f32"].params[0].1.clone();
    assert!(!tc.types_compatible(&tc.resolve_type(&expected), &Type::F64));
}

#[test]
fn unknown_root_std_module_call_is_error() {
    let program = parse_program(
        r#"
{ io } = std

main = () void {
    io.nope("bad")
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown std module function should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("undefined module function `io.nope`")),
        "expected undefined module function diagnostic, got {errors:?}"
    );
}

#[test]
fn known_root_std_runtime_standins_remain_allowed() {
    let program = parse_program(
        r#"
{ io } = std

main = () void {
    io.print("hello")
    io.println("world")
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("temporary root std io stand-ins should typecheck");
}

#[test]
fn non_void_function_without_return_is_error() {
    let program = parse_program(
        r#"
missing = () i32 {
    x = 1
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-void fallthrough should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("function `missing` must return `i32` on all non-error paths")),
        "expected missing return diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_missing_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green, Blue

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-exhaustive enum match should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-exhaustive match on `Color`: missing `Blue`")),
        "expected non-exhaustive enum diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_duplicate_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Red { "again" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate enum match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate match arm for `Color.Red`")),
        "expected duplicate enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_unknown_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Blue { "blue" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown enum match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("enum `Color` has no variant `Blue`")),
        "expected unknown enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_payload_shape_is_checked() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

describe = (m: Maybe) StaticString {
    m ?
        | Some { "some" }
        | None(value) { "none" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum match payload shape should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("match arm `Maybe.Some` requires a payload")),
        "expected missing payload diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("match arm `Maybe.None` does not accept a payload")),
        "expected forbidden payload diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_wildcard_after_all_variants_is_redundant() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
        | _ { "fallback" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("redundant enum wildcard arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("redundant wildcard match arm")),
        "expected redundant wildcard diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_variant_after_wildcard_is_redundant() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | _ { "fallback" }
        | Red { "red" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum variant after wildcard should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("redundant match arm for `Color.Red`")),
        "expected redundant enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn bool_match_missing_arm_is_error_for_value_match() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-exhaustive boolean value match should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-exhaustive bool match: missing `false`")),
        "expected non-exhaustive bool diagnostic, got {errors:?}"
    );
}

#[test]
fn bool_match_duplicate_arm_is_error() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
        | true { "again" }
        | false { "no" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate boolean match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate match arm for `true`")),
        "expected duplicate bool arm diagnostic, got {errors:?}"
    );
}

#[test]
fn match_arm_return_does_not_force_never_result_type() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "early" }
        | false { "late" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_program(&program)
        .expect("returning arm should not force match type to never");
    let body = &typed.functions[0].body;
    assert_eq!(body.ty, Type::Str);
}

#[test]
fn types_compatible_basics() {
    let tc = TypeChecker::new();
    // Same types
    assert!(tc.types_compatible(&Type::I32, &Type::I32));
    // Numeric conversions require explicit casts except literal coercion.
    assert!(!tc.types_compatible(&Type::I64, &Type::I32));
    assert!(!tc.types_compatible(&Type::F32, &Type::F64));
    // Unknown is permissive
    assert!(tc.types_compatible(&Type::I32, &Type::Unknown));
    // Named types are nominal and do not match unrelated concrete types.
    assert!(tc.types_compatible(&Type::Named("UserId".into()), &Type::Named("UserId".into())));
    assert!(!tc.types_compatible(
        &Type::Named("UserId".into()),
        &Type::Named("OrderId".into())
    ));
    assert!(!tc.types_compatible(&Type::Str, &Type::Named("StaticString".into())));
    // Clear mismatch
    assert!(!tc.types_compatible(&Type::I32, &Type::Str));
    assert!(!tc.types_compatible(&Type::Bool, &Type::I32));
}

#[test]
fn literal_coercion_in_var_decl() {
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "main".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: vec![Statement::VarDecl {
                    name: "x".into(),
                    ty: Some(AstType::I64),
                    value: Expression::IntLiteral {
                        value: 42,
                        span: Span::dummy(),
                    },
                    mutable: false,
                    constant: false,
                    span: Span::dummy(),
                }],
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    let result = tc.check_program(&program).unwrap();
    // The variable should have type I64 (coerced from I32 literal)
    let body = &result.functions[0].body;
    match &body.statements[0].kind {
        TypedStatementKind::VarDecl { ty, .. } => assert_eq!(*ty, Type::I64),
        _ => panic!("expected VarDecl"),
    }
}

#[test]
fn resolve_string_type() {
    let tc = TypeChecker::new();
    // "String" as a named type should resolve to Type::String
    assert_eq!(
        tc.resolve_type(&AstType::Named("String".into())),
        Type::String
    );
}

#[test]
fn resolve_slice_type() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.resolve_type(&AstType::Slice(Box::new(AstType::I32))),
        Type::Slice(Box::new(Type::I32))
    );
}

#[test]
fn infer_type_args_basic() {
    let tc = TypeChecker::new();
    // Generic function: identity<T>(x: T) -> T
    let type_params = vec!["T".to_string()];
    let params = vec![("x".to_string(), AstType::Named("T".into()))];
    let arg_types = vec![Type::I32];
    let subs = tc.infer_type_args(&type_params, &params, &arg_types);
    assert_eq!(subs.get("T"), Some(&Type::I32));
}

#[test]
fn substitute_type_basic() {
    let tc = TypeChecker::new();
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), Type::I32);
    // T → I32
    assert_eq!(
        tc.substitute_type(&AstType::Named("T".into()), &subs),
        Type::I32
    );
    // Ptr<T> → Ptr<I32>
    assert_eq!(
        tc.substitute_type(&AstType::Ptr(Box::new(AstType::Named("T".into()))), &subs),
        Type::Ptr(Box::new(Type::I32))
    );
    // Non-generic type unchanged
    assert_eq!(tc.substitute_type(&AstType::Bool, &subs), Type::Bool);
}

#[test]
fn substitute_type_covers_all_composite_type_shapes() {
    let tc = TypeChecker::new();
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), Type::I32);

    assert_eq!(
        tc.substitute_type(
            &AstType::RawPtr(Box::new(AstType::Named("T".into()))),
            &subs
        ),
        Type::RawPtr(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::MutPtr(Box::new(AstType::Named("T".into()))),
            &subs
        ),
        Type::MutPtr(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(&AstType::Slice(Box::new(AstType::Named("T".into()))), &subs),
        Type::Slice(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::Array {
                elem: Box::new(AstType::Named("T".into())),
                size: Some(3),
            },
            &subs,
        ),
        Type::Array {
            elem: Box::new(Type::I32),
            size: Some(3),
        }
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::Function {
                params: vec![AstType::Named("T".into())],
                ret: Box::new(AstType::Named("T".into())),
            },
            &subs,
        ),
        Type::Function {
            params: vec![Type::I32],
            ret: Box::new(Type::I32),
        }
    );
}

#[test]
fn substitute_type_preserves_function_type_arguments_in_nested_generics() {
    let mut tc = TypeChecker::new();
    tc.structs.insert(
        "Box".to_string(),
        StructInfo {
            name: "Box".to_string(),
            fields: vec![("value".to_string(), AstType::Named("T".to_string()))],
            field_defaults: HashMap::new(),
            type_params: vec!["T".to_string()],
            type_param_bounds: HashMap::new(),
        },
    );
    let function_type = Type::Function {
        params: vec![Type::I32],
        ret: Box::new(Type::I32),
    };
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), function_type.clone());

    assert_eq!(
        tc.substitute_type(
            &AstType::Generic {
                name: "Box".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            },
            &subs,
        ),
        Type::Struct {
            name: "Box_fn_i32_ret_i32".to_string(),
            fields: vec![("value".to_string(), function_type)],
        }
    );
}

#[test]
fn generic_function_collection() {
    use crate::ast::Expression;
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Function {
        name: "identity".into(),
        type_params: vec![crate::ast::declarations::TypeParam {
            name: "T".into(),
            constraint: None,
            constraint_type_args: Vec::new(),
            span: Span::dummy(),
        }],
        params: vec![crate::ast::Param {
            name: "x".into(),
            ty: AstType::Named("T".into()),
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Named("T".into())),
        body: Expression::Block {
            statements: Vec::new(),
            expr: None,
            span: Span::dummy(),
        },
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    let info = tc.functions.get("identity").unwrap();
    assert_eq!(info.type_params, vec!["T".to_string()]);
}

#[test]
fn generic_method_collection() {
    use crate::ast::Expression;
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Method {
        type_name: "Box".into(),
        method_name: "get".into(),
        type_params: vec![crate::ast::declarations::TypeParam {
            name: "T".into(),
            constraint: None,
            constraint_type_args: Vec::new(),
            span: Span::dummy(),
        }],
        params: vec![crate::ast::Param {
            name: "value".into(),
            ty: AstType::Named("T".into()),
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Named("T".into())),
        body: Expression::Block {
            statements: Vec::new(),
            expr: None,
            span: Span::dummy(),
        },
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    let info = tc.methods.get("Box.get").unwrap();
    assert_eq!(info.type_params, vec!["T".to_string()]);
    assert!(tc.generic_methods.contains_key("Box.get"));
}

#[test]
fn type_impl_method_collection() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 {
        self.x
    }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.collect_declarations(&program.declarations);
    let info = tc.methods.get("Point.get").unwrap();
    assert_eq!(info.params.len(), 1);
    assert_eq!(info.return_type, AstType::I32);
}

#[test]
fn behavior_declaration_collection() {
    let program = parse_program(
        r#"
Serializable: behavior {
    to_json: (Self) String
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.collect_declarations(&program.declarations);
    let info = tc.behaviors.get("Serializable").unwrap();
    assert_eq!(info.name, "Serializable");
    assert_eq!(info.methods.len(), 1);
    assert_eq!(info.methods[0].name, "to_json");
}

#[test]
fn behavior_impl_with_required_method_passes() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("valid behavior impl should typecheck");
}

#[test]
fn behavior_impl_missing_required_method_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("missing behavior method should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `Json` is missing required method `to_json`"
        )),
        "expected missing behavior method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_can_omit_default_method() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str { "{}" }
}

Point.implements(Json) {
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("behavior impl may omit a method with a default body");
}

#[test]
fn behavior_impl_duplicate_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate behavior impl should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("duplicate implementation of behavior `Json` for type `Point`")),
        "expected duplicate behavior impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_without_type_args_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior impl arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_with_type_args_passes_requires() {
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

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior impl should satisfy matching generic requires");
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_failure_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior type argument bound should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json<Point>` required by `T`")),
        "expected generic behavior type argument bound diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Json<Point>) {
    encode = (value: Point) Point { value }
}

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior type argument bound should pass when satisfied");
}

#[test]
fn behavior_requires_generic_behavior_type_arg_arity_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<i32, str>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior requires arity mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 2")),
        "expected generic behavior requires arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_substitutes_method_signature() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) i32 { 1 }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl return mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("method `encode` for behavior `Json_str` expects return `str`, found `i32`")),
        "expected substituted behavior method return diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_overlapping_inherited_behavior_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

Point.implements(PrettyJson) {
    to_json = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("overlapping inherited behavior impl should fail");
    assert!(
        errors.iter().any(|d| {
            d.message.contains(
                "overlapping implementations of behaviors `Json` and `PrettyJson` for type `Point`",
            )
        }),
        "expected overlapping behavior impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_passes_when_impl_exists() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

Point.requires(Json)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("requires should pass when behavior impl exists");
}

#[test]
fn behavior_requires_rejects_missing_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.requires(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("requires should fail without behavior impl");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement required behavior `Json`")),
        "expected requires missing impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_generic_behavior_without_type_args_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.requires(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior requires without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior requires arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_requires_parent_methods() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("extended behavior should require parent methods");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `PrettyJson` is missing required method `to_json`"
        )),
        "expected inherited missing method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_impl_satisfies_parent_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

Point.requires(Json)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("implementation of child behavior should satisfy parent requires");
}

#[test]
fn behavior_extends_generic_parent_requires_substituted_methods() {
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
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic parent method should be required with substituted signature");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `PrettyJson` is missing required method `encode`"
        )),
        "expected inherited generic parent missing method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_generic_parent_satisfies_specialized_requires() {
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

    TypeChecker::new()
        .check_program(&program)
        .expect("child behavior impl should satisfy specialized generic parent requires");
}

#[test]
fn behavior_extends_generic_parent_accepts_child_type_parameter_arg() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior parent should accept child type parameter args");
}

#[test]
fn behavior_impl_generic_parent_overlap_is_error() {
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

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("specialized parent and child behavior impls should overlap");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
        )),
        "expected specialized behavior impl overlap diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_distinct_generic_specializations_do_not_overlap() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("distinct behavior specializations should not overlap");
}

#[test]
fn behavior_extends_cycle_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("cyclic behavior inheritance should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("behavior inheritance cycle")),
        "expected behavior inheritance cycle diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_duplicate_parent_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("duplicate behavior inheritance edge should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("duplicate behavior inheritance `PrettyJson.extends(Json)`")
        }),
        "expected duplicate behavior inheritance diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_duplicate_generic_parent_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Json<str>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("duplicate specialized behavior inheritance edge should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("duplicate behavior inheritance `PrettyJson.extends(Json<str>)`")
        }),
        "expected duplicate generic behavior inheritance diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_generic_parent_without_type_args_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior extends parent without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior extends parent arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_conflicting_method_signature_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    to_json: (Self) i32
}

PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("conflicting inherited behavior method should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("conflicting behavior method `to_json` inherited by `PrettyJson`")
        }),
        "expected conflicting inherited behavior method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_signature_mismatch_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: i32) i32 { value }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("behavior impl signature mismatch should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("parameter 1 for method `to_json`")),
        "expected behavior parameter mismatch diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("expects return `str`, found `i32`")),
        "expected behavior return mismatch diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_explicit_type_arg_arity_is_error() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T {
    value
}

main = () i32 {
    identity<i32, str>(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("wrong generic type-argument arity should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic function `identity` expects 1 type arguments, found 2")),
        "expected generic arity diagnostic, got {errors:?}"
    );
}

#[test]
fn nongeneric_function_explicit_type_args_are_error() {
    let program = parse_program(
        r#"
id = (value: i32) i32 {
    value
}

main = () i32 {
    id<i32>(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-generic function type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic function `id` does not accept type arguments")),
        "expected non-generic type-argument diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_failure_is_error() {
    let program = parse_program(
        r#"
make_default<T> = () T {
    0
}

main = () i32 {
    make_default()
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("uninferred generic type argument should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot infer type argument `T` for generic function `make_default`")),
        "expected generic inference diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_bound_references_unknown_behavior_is_error() {
    let program = parse_program(
        r#"
show<T: Display> = (value: T) T {
    value
}

main = () i32 {
    show(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown generic behavior bounds should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "generic bound `Display` on type parameter `T` references undefined behavior"
        )),
        "expected generic bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_bound_rejects_unspecialized_generic_behavior() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    "encoded"
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("generic behavior bound without type arguments should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")
        }),
        "expected generic behavior bound arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_behavior_bound_with_type_args_accepts_matching_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<Point>) {
    encode = (value: Point) Point { value }
}

identity<T: Json<T>> = (value: T) T {
    value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    same.x
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior bound type argument should substitute at call site");
}

#[test]
fn generic_behavior_bound_with_type_args_rejects_mismatched_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

identity<T: Json<T>> = (value: T) T {
    value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    same.x
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior bound should require matching behavior type args");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json<Point>` required by `T`")),
        "expected generic behavior bound type argument diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_generic_bound_accepts_later_behavior_declaration() {
    let program = parse_program(
        r#"
Serializable<T: Json>: behavior {
    encode: (Self) str
}

Json: behavior {
    to_json: (Self) str
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("behavior generic bounds should be independent of declaration order");
}

#[test]
fn behavior_generic_bound_unknown_behavior_reports_once() {
    let program = parse_program(
        r#"
Serializable<T: Missing>: behavior {
    encode: (Self) str
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown behavior generic bound should fail");
    let count = errors
        .iter()
        .filter(|d| {
            d.message.contains(
                "generic bound `Missing` on type parameter `T` references undefined behavior",
            )
        })
        .count();
    assert_eq!(
        count, 1,
        "expected one behavior generic bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_behavior_bound_accepts_type_with_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

encode<T: Json> = (value: T) str {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("type with behavior impl should satisfy generic bound");
}

#[test]
fn generic_behavior_bound_accepts_inherited_behavior_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

encode<T: Json> = (value: T) str {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("child behavior impl should satisfy inherited generic bound");
}

#[test]
fn generic_behavior_bound_rejects_type_without_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("type without behavior impl should not satisfy generic bound");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json`")),
        "expected missing generic bound impl diagnostic, got {errors:?}"
    );
}

#[test]
fn func_info_non_generic_has_empty_type_params() {
    use crate::ast::Expression;
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Function {
        name: "add".into(),
        type_params: Vec::new(),
        params: vec![
            crate::ast::Param {
                name: "a".into(),
                ty: AstType::I32,
                mutable: false,
                span: Span::dummy(),
            },
            crate::ast::Param {
                name: "b".into(),
                ty: AstType::I32,
                mutable: false,
                span: Span::dummy(),
            },
        ],
        return_type: Some(AstType::I32),
        body: Expression::Block {
            statements: Vec::new(),
            expr: None,
            span: Span::dummy(),
        },
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    let info = tc.functions.get("add").unwrap();
    assert!(info.type_params.is_empty());
}
