extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, VariableDeclarationType};
use test_utils::TestContext;
use test_utils::test_context;
use inkwell::context::Context;

// Test nested struct field access
#[test]
fn test_nested_struct_field_access() {
    test_context!(|test_context: &mut TestContext| {
        // Define inner struct
        let inner_struct = ast::Declaration::Struct(ast::StructDefinition {
            name: "Inner".to_string(),
            type_params: vec![],
            fields: vec![
                ast::StructField {
                    name: "value".to_string(),
                    type_: ast::AstType::I32,
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        });
        
        // Define outer struct
        let outer_struct = ast::Declaration::Struct(ast::StructDefinition {
            name: "Outer".to_string(),
            type_params: vec![],
            fields: vec![
                ast::StructField {
                    name: "inner".to_string(),
                    type_: ast::AstType::Struct {
                        name: "Inner".to_string(),
                        fields: vec![("value".to_string(), AstType::I32)],
                    },
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        });
        
        // Main function
        let func = ast::Function {
            type_params: vec![],
            is_async: false,
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I32,
            body: vec![
                // Create inner struct
                Statement::VariableDeclaration {
                    name: "inner".to_string(),
                    type_: Some(AstType::Struct {
                        name: "Inner".to_string(),
                        fields: vec![("value".to_string(), AstType::I32)],
                    }),
                    initializer: Some(Expression::StructLiteral {
                        name: "Inner".to_string(),
                        fields: vec![
                            ("value".to_string(), Expression::Integer32(42)),
                        ],
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                // Create outer struct
                Statement::VariableDeclaration {
                    name: "outer".to_string(),
                    type_: Some(AstType::Struct {
                        name: "Outer".to_string(),
                        fields: vec![
                            ("inner".to_string(), AstType::Struct {
                                name: "Inner".to_string(),
                                fields: vec![("value".to_string(), AstType::I32)],
                            }),
                        ],
                    }),
                    initializer: Some(Expression::StructLiteral {
                        name: "Outer".to_string(),
                        fields: vec![
                            ("inner".to_string(), Expression::Identifier("inner".to_string())),
                        ],
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                // Access nested field: outer.inner.value
                Statement::Return(
                    Expression::StructField {
                        struct_: Box::new(
                            Expression::StructField {
                                struct_: Box::new(Expression::Identifier("outer".to_string())),
                                field: "inner".to_string(),
                            }
                        ),
                        field: "value".to_string(),
                    }
                ),
            ],
        };
        
        let program = ast::Program {
            declarations: vec![inner_struct, outer_struct, ast::Declaration::Function(func)],
        };
        
        let result = test_context.compile(&program);
        if let Err(e) = &result {
            panic!("Compilation failed: {:?}", e);
        }
        assert!(result.is_ok());
    });
}

// Test struct field access from function return
#[test]
fn test_struct_field_from_function_return() {
    test_context!(|test_context: &mut TestContext| {
        // Define Point struct
        let point_struct = ast::Declaration::Struct(ast::StructDefinition {
            name: "Point".to_string(),
            type_params: vec![],
            fields: vec![
                ast::StructField {
                    name: "x".to_string(),
                    type_: ast::AstType::I32,
                    is_mutable: false,
                    default_value: None,
                },
                ast::StructField {
                    name: "y".to_string(),
                    type_: ast::AstType::I32,
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        });
        
        // Helper function that returns a Point
        let make_point = ast::Function {
            type_params: vec![],
            is_async: false,
            name: "makePoint".to_string(),
            args: vec![],
            return_type: AstType::Struct {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), AstType::I32),
                    ("y".to_string(), AstType::I32),
                ],
            },
            body: vec![
                Statement::Return(Expression::StructLiteral {
                    name: "Point".to_string(),
                    fields: vec![
                        ("x".to_string(), Expression::Integer32(100)),
                        ("y".to_string(), Expression::Integer32(200)),
                    ],
                }),
            ],
        };
        
        // Main function that accesses field from function return
        let func = ast::Function {
            type_params: vec![],
            is_async: false,
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I32,
            body: vec![
                // Access x field directly from function return
                Statement::Return(
                    Expression::StructField {
                        struct_: Box::new(Expression::FunctionCall {
                            name: "makePoint".to_string(),
                            args: vec![],
                        }),
                        field: "x".to_string(),
                    }
                ),
            ],
        };
        
        let program = ast::Program {
            declarations: vec![
                point_struct,
                ast::Declaration::Function(make_point),
                ast::Declaration::Function(func),
            ],
        };
        
        let result = test_context.compile(&program);
        if let Err(e) = &result {
            panic!("Compilation failed: {:?}", e);
        }
        assert!(result.is_ok());
    });
}

// Test struct field access from struct literal
#[test]
fn test_struct_field_from_literal() {
    test_context!(|test_context: &mut TestContext| {
        // Define Point struct
        let point_struct = ast::Declaration::Struct(ast::StructDefinition {
            name: "Point".to_string(),
            type_params: vec![],
            fields: vec![
                ast::StructField {
                    name: "x".to_string(),
                    type_: ast::AstType::I32,
                    is_mutable: false,
                    default_value: None,
                },
                ast::StructField {
                    name: "y".to_string(),
                    type_: ast::AstType::I32,
                    is_mutable: false,
                    default_value: None,
                },
            ],
            methods: vec![],
        });
        
        // Main function
        let func = ast::Function {
            type_params: vec![],
            is_async: false,
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I32,
            body: vec![
                // Access field directly from struct literal
                Statement::Return(
                    Expression::StructField {
                        struct_: Box::new(Expression::StructLiteral {
                            name: "Point".to_string(),
                            fields: vec![
                                ("x".to_string(), Expression::Integer32(42)),
                                ("y".to_string(), Expression::Integer32(84)),
                            ],
                        }),
                        field: "x".to_string(),
                    }
                ),
            ],
        };
        
        let program = ast::Program {
            declarations: vec![point_struct, ast::Declaration::Function(func)],
        };
        
        let result = test_context.compile(&program);
        if let Err(e) = &result {
            panic!("Compilation failed: {:?}", e);
        }
        assert!(result.is_ok());
    });
}