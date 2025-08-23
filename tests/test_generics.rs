use zen::ast::{AstType, TypeParameter, Function, StructDefinition, StructField, EnumDefinition, EnumVariant, Statement, Expression};
use zen::type_system::{TypeEnvironment, TypeInstantiator, TypeSubstitution, Monomorphizer};

#[test]
fn test_type_substitution() {
    let mut substitution = TypeSubstitution::new();
    substitution.add("T".to_string(), AstType::I32);
    substitution.add("U".to_string(), AstType::String);
    
    let generic_type = AstType::Generic {
        name: "T".to_string(),
        type_args: vec![],
    };
    let result = substitution.apply(&generic_type);
    assert_eq!(result, AstType::I32);
    
    let nested_generic = AstType::Array(Box::new(AstType::Generic {
        name: "U".to_string(),
        type_args: vec![],
    }));
    let result = substitution.apply(&nested_generic);
    assert_eq!(result, AstType::Array(Box::new(AstType::String)));
}

#[test]
fn test_generic_function_instantiation() {
    let mut env = TypeEnvironment::new();
    
    let generic_func = Function {
        name: "identity".to_string(),
        type_params: vec![TypeParameter {
            name: "T".to_string(),
            constraints: vec![],
        }],
        args: vec![("value".to_string(), AstType::Generic {
            name: "T".to_string(),
            type_args: vec![],
        })],
        return_type: AstType::Generic {
            name: "T".to_string(),
            type_args: vec![],
        },
        body: vec![Statement::Return(Expression::Identifier("value".to_string()))],
        is_async: false,
    };
    
    env.register_generic_function(generic_func.clone());
    
    let mut instantiator = TypeInstantiator::new(&mut env);
    let instantiated = instantiator.instantiate_function(&generic_func, vec![AstType::I32]).unwrap();
    
    assert_eq!(instantiated.name, "identity_i32");
    assert_eq!(instantiated.type_params.len(), 0);
    assert_eq!(instantiated.args[0].1, AstType::I32);
    assert_eq!(instantiated.return_type, AstType::I32);
}

#[test]
fn test_generic_struct_instantiation() {
    let mut env = TypeEnvironment::new();
    
    let generic_struct = StructDefinition {
        name: "Box".to_string(),
        type_params: vec![TypeParameter {
            name: "T".to_string(),
            constraints: vec![],
        }],
        fields: vec![StructField {
            name: "value".to_string(),
            type_: AstType::Generic {
                name: "T".to_string(),
                type_args: vec![],
            },
            is_mutable: false,
            default_value: None,
        }],
        methods: vec![],
    };
    
    env.register_generic_struct(generic_struct.clone());
    
    let mut instantiator = TypeInstantiator::new(&mut env);
    let instantiated = instantiator.instantiate_struct(&generic_struct, vec![AstType::String]).unwrap();
    
    assert_eq!(instantiated.name, "Box_string");
    assert_eq!(instantiated.type_params.len(), 0);
    assert_eq!(instantiated.fields[0].type_, AstType::String);
}

#[test]
fn test_nested_generic_types() {
    let mut substitution = TypeSubstitution::new();
    substitution.add("T".to_string(), AstType::I32);
    
    let nested_type = AstType::Generic {
        name: "Option".to_string(),
        type_args: vec![AstType::Generic {
            name: "T".to_string(),
            type_args: vec![],
        }],
    };
    
    let result = substitution.apply(&nested_type);
    match result {
        AstType::Generic { name, type_args } => {
            assert_eq!(name, "Option");
            assert_eq!(type_args.len(), 1);
            assert_eq!(type_args[0], AstType::I32);
        }
        _ => panic!("Expected Generic type"),
    }
}

#[test]
fn test_multiple_type_parameters() {
    let mut env = TypeEnvironment::new();
    
    let generic_func = Function {
        name: "pair".to_string(),
        type_params: vec![
            TypeParameter {
                name: "T".to_string(),
                constraints: vec![],
            },
            TypeParameter {
                name: "U".to_string(),
                constraints: vec![],
            },
        ],
        args: vec![
            ("first".to_string(), AstType::Generic {
                name: "T".to_string(),
                type_args: vec![],
            }),
            ("second".to_string(), AstType::Generic {
                name: "U".to_string(),
                type_args: vec![],
            }),
        ],
        return_type: AstType::Void,
        body: vec![],
        is_async: false,
    };
    
    env.register_generic_function(generic_func.clone());
    
    let mut instantiator = TypeInstantiator::new(&mut env);
    let instantiated = instantiator.instantiate_function(
        &generic_func,
        vec![AstType::I32, AstType::String]
    ).unwrap();
    
    assert_eq!(instantiated.name, "pair_i32_string");
    assert_eq!(instantiated.args[0].1, AstType::I32);
    assert_eq!(instantiated.args[1].1, AstType::String);
}

#[test]
fn test_generic_enum_instantiation() {
    let mut env = TypeEnvironment::new();
    
    let generic_enum = EnumDefinition {
        name: "Result".to_string(),
        type_params: vec![
            TypeParameter {
                name: "T".to_string(),
                constraints: vec![],
            },
            TypeParameter {
                name: "E".to_string(),
                constraints: vec![],
            },
        ],
        variants: vec![
            EnumVariant {
                name: "Ok".to_string(),
                payload: Some(AstType::Generic {
                    name: "T".to_string(),
                    type_args: vec![],
                }),
            },
            EnumVariant {
                name: "Err".to_string(),
                payload: Some(AstType::Generic {
                    name: "E".to_string(),
                    type_args: vec![],
                }),
            },
        ],
        methods: vec![],
    };
    
    env.register_generic_enum(generic_enum.clone());
    
    let mut instantiator = TypeInstantiator::new(&mut env);
    let instantiated = instantiator.instantiate_enum(
        &generic_enum,
        vec![AstType::I32, AstType::String]
    ).unwrap();
    
    assert_eq!(instantiated.name, "Result_i32_string");
    assert_eq!(instantiated.variants[0].payload, Some(AstType::I32));
    assert_eq!(instantiated.variants[1].payload, Some(AstType::String));
}

#[test]
fn test_type_environment_scope() {
    let mut env = TypeEnvironment::new();
    
    env.push_scope(vec![TypeParameter {
        name: "T".to_string(),
        constraints: vec![],
    }]);
    
    env.add_substitution("T".to_string(), AstType::I32);
    
    let generic_type = AstType::Generic {
        name: "T".to_string(),
        type_args: vec![],
    };
    
    let resolved = env.resolve_type(&generic_type);
    assert_eq!(resolved, AstType::I32);
    
    env.pop_scope();
    
    let resolved_after_pop = env.resolve_type(&generic_type);
    assert_eq!(resolved_after_pop, generic_type);
}

#[test]
fn test_validate_type_args() {
    let env = TypeEnvironment::new();
    
    let params = vec![
        TypeParameter {
            name: "T".to_string(),
            constraints: vec![],
        },
        TypeParameter {
            name: "U".to_string(),
            constraints: vec![],
        },
    ];
    
    let result = env.validate_type_args(&params, &vec![AstType::I32, AstType::String]);
    assert!(result.is_ok());
    
    let result = env.validate_type_args(&params, &vec![AstType::I32]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Type argument count mismatch"));
}

#[test]
fn test_is_generic_type() {
    use zen::type_system::is_generic_type;
    
    assert!(is_generic_type(&AstType::Generic {
        name: "T".to_string(),
        type_args: vec![],
    }));
    
    assert!(is_generic_type(&AstType::Array(Box::new(AstType::Generic {
        name: "T".to_string(),
        type_args: vec![],
    }))));
    
    assert!(!is_generic_type(&AstType::I32));
    assert!(!is_generic_type(&AstType::String));
}

#[test]
fn test_extract_type_parameters() {
    use zen::type_system::extract_type_parameters;
    
    let type_with_params = AstType::Function {
        args: vec![
            AstType::Generic {
                name: "T".to_string(),
                type_args: vec![],
            },
            AstType::Generic {
                name: "U".to_string(),
                type_args: vec![],
            },
        ],
        return_type: Box::new(AstType::Generic {
            name: "T".to_string(),
            type_args: vec![],
        }),
    };
    
    let params = extract_type_parameters(&type_with_params);
    assert_eq!(params.len(), 2);
    assert!(params.contains(&"T".to_string()));
    assert!(params.contains(&"U".to_string()));
}