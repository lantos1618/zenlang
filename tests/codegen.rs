extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, ConditionalArm, Pattern, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

// Helper function to compile and execute a program
fn compile_and_run<'ctx>(test_context: &mut TestContext<'ctx>, program: &ast::Program) -> i64 {
    test_context.compile(program).unwrap();
    let execution_engine = test_context.module().unwrap().create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let jit_function: JitFunction<unsafe extern "C" fn() -> i64> = unsafe { execution_engine.get_function("main").unwrap() };
    let result = unsafe { jit_function.call() };
    drop(execution_engine); // Explicitly drop before returning
    result
}

#[test]
fn test_simple_return() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_simple_program(42);
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_simple_return_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_simple_program(42);
        test_context.compile(&program).unwrap();
        let ir = test_context.get_ir().unwrap();
        assert!(ir.contains("ret i64 42"), "IR should contain 'ret i64 42'\nIR was:\n{}", ir);
    });
}

#[test]
fn test_binary_operations_execution() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_binary_op_program(40, BinaryOperator::Add, 2);
        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_binary_operations_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_binary_op_program(40, BinaryOperator::Add, 2);
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_variable_declaration_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_variable_program("x", 42);
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_string_literal() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { 
                is_async: false,
                name: "get_string".to_string(),
                args: vec![],
                return_type: AstType::String,
                body: vec![Statement::Return(Expression::String("Hello, World!".to_string()))],
            },
            ast::Function { 
                is_async: false,
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration { 
                        name: "str".to_string(),
                        type_: Some(AstType::String),
                        initializer: Some(Expression::FunctionCall {
                            name: "get_string".to_string(),
                            args: vec![],
                        }),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::Integer64(0)), // Return 0 to indicate success
                ],
            },
        ]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 0);
    });
}

#[test]
fn test_conditional_expression() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::Return(Expression::Conditional {
                    scrutinee: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Integer64(1)),
                        op: BinaryOperator::Equals,
                        right: Box::new(Expression::Integer64(1)),
                    }),
                    arms: vec![
                        ConditionalArm { 
                            pattern: Pattern::Literal(Expression::Integer64(1)), 
                            guard: None, 
                            body: Expression::Integer64(42) 
                        },
                        ConditionalArm { 
                            pattern: Pattern::Literal(Expression::Integer64(0)), 
                            guard: None, 
                            body: Expression::Integer64(0) 
                        },
                    ],
                }),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_variable_declaration() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_variable_program("x", 42);
        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_function_call() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { 
                is_async: false,
                name: "add".to_string(),
                args: vec![
                    ("a".to_string(), AstType::I64),
                    ("b".to_string(), AstType::I64),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            },
            ast::Function { 
                is_async: false,
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: "add".to_string(),
                    args: vec![
                        Expression::Integer64(40),
                        Expression::Integer64(2),
                    ],
                })],
            },
        ]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_undefined_variable() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_undefined".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Identifier("x".to_string()))],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
        match result {
            Err(CompileError::UndeclaredVariable(name, _)) => assert_eq!(name, "x"),
            _ => panic!("Expected UndeclaredVariable error"),
        }
    });
}

#[test]
fn test_type_mismatch() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_type_mismatch".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Integer64(1)),
                op: ast::BinaryOperator::Add,
                right: Box::new(Expression::Float64(2.0)),
            })],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
        match result {
            Err(CompileError::TypeMismatch { expected, found, .. }) => {
                assert_eq!(expected, "int or float");
                assert!(found.contains("mixed types"));
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    });
}

#[test]
fn test_undefined_function() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_undefined_func".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::FunctionCall {
                name: "nonexistent".to_string(),
                args: vec![],
            })],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
        match result {
            Err(CompileError::UndeclaredFunction(name, _)) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected UndeclaredFunction error"),
        }
    });
}

#[test]
fn test_invalid_function_type() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_invalid_type".to_string(),
            args: vec![("x".to_string(), AstType::Function {
                args: vec![AstType::I64],
                return_type: Box::new(AstType::I64),
            })],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Integer64(0))],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
    });
}

#[test]
fn test_nested_conditionals() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::Return(Expression::Conditional {
                    scrutinee: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Integer64(1)),
                        op: ast::BinaryOperator::Equals,
                        right: Box::new(Expression::Integer64(1)),
                    }),
                    arms: vec![
                        ConditionalArm { 
                            pattern: Pattern::Literal(Expression::Integer64(1)), 
                            guard: None, 
                            body: Expression::Integer64(42) 
                        },
                        ConditionalArm { 
                            pattern: Pattern::Literal(Expression::Integer64(0)), 
                            guard: None, 
                            body: Expression::Integer64(0) 
                        },
                    ],
                }),
            ],
        }]);

        let result: i64 = compile_and_run(&mut test_context, &program);
        assert_eq!(result, 42);
    });
}

#[test]
fn test_function_pointer() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { 
                is_async: false,
                name: "add".to_string(),
                args: vec![
                    ("a".to_string(), AstType::I64),
                    ("b".to_string(), AstType::I64),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            },
            ast::Function { 
                is_async: false,
                name: "test_func_ptr".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration { 
                        name: "op".to_string(),
                        type_: Some(AstType::Function {
                            args: vec![AstType::I64, AstType::I64],
                            return_type: Box::new(AstType::I64),
                        }),
                        initializer: Some(Expression::Identifier("add".to_string())),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::FunctionCall {
                        name: "op".to_string(),
                        args: vec![
                            Expression::Integer64(40),
                            Expression::Integer64(2),
                        ],
                    }),
                ],
            },
        ]);

        let result = test_context.compile(&program);
        if let Err(e) = &result {
            println!("Function pointer test error: {:?}", e);
        }
        assert!(result.is_ok());
    });
}

#[test]
fn test_recursive_function() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { is_async: false, 
                name: "factorial".to_string(),
                args: vec![("n".to_string(), AstType::I64)],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(
                        Expression::Conditional {
                            scrutinee: Box::new(Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("n".to_string())),
                                op: ast::BinaryOperator::Equals,
                                right: Box::new(Expression::Integer64(0)),
                        }),
                        arms: vec![
                            ConditionalArm { pattern: Pattern::Literal(
                                Expression::Integer64(1)
                            ), 
                            guard: None, 
                            body: Expression::Integer64(1) 
                        },
                        ConditionalArm { pattern: Pattern::Literal(
                            Expression::Integer64(0)
                        ), 
                        guard: None, body: Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("n".to_string())),
                                op: ast::BinaryOperator::Multiply,
                                right: Box::new(Expression::FunctionCall {
                                    name: "factorial".to_string(),
                                    args: vec![Expression::BinaryOp {
                                        left: Box::new(Expression::Identifier("n".to_string())),
                                        op: ast::BinaryOperator::Subtract,
                                        right: Box::new(Expression::Integer64(1)),
                                    }],
                                }),
                            },
                        },
                    ],
                }),
                ],
            },
            ast::Function { 
                is_async: false, 
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Expression::FunctionCall {
                        name: "factorial".to_string(),
                        args: vec![Expression::Integer64(5)],
                    }),
                ],
            },
        ]);

        let result: i64 = compile_and_run(&mut test_context, &program);
        assert_eq!(result, 120); // 5! = 120
    });
}

#[test]
fn test_pointer_operations() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "x".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(42)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::VariableDeclaration { 
                    name: "p".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::I64))),
                    initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("x".to_string())))),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("p".to_string())))),
            ],
        }]);

        let result: i64 = compile_and_run(&mut test_context, &program);
        assert_eq!(result, 42);
    });
}

#[test]
fn test_pointer_arithmetic() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_ptr_arithmetic".to_string(),
            args: vec![("arr".to_string(), AstType::Pointer(Box::new(AstType::I64)))],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "ptr".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::I64))),
                    initializer: Some(Expression::PointerOffset {
                        pointer: Box::new(Expression::Identifier("arr".to_string())),
                        offset: Box::new(Expression::Integer64(1)),
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
            ],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_pointer_assignment() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_ptr_assign".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "x".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(42)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::VariableDeclaration { 
                    name: "ptr".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::I64))),
                    initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("x".to_string())))),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::PointerAssignment {
                    pointer: Expression::Identifier("ptr".to_string()),
                    value: Expression::Integer64(100),
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let ir = test_context.module().unwrap().print_to_string().to_string();
        assert!(ir.contains("define i64 @test_ptr_assign"));
        assert!(ir.contains("store i64 100"));
        assert!(ir.contains("load i64"));
    });
}

#[test]
fn test_invalid_dereferencing_non_pointer() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_invalid_deref".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "x".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(42)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("x".to_string())))),
            ],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
    });
}

#[test]
fn test_void_pointer_declaration() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(
            vec![
                ast::Function { is_async: false, 
                    name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "ptr".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::Void))),
                    initializer: None,
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Integer64(0)),
            ],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
    });
}

#[test]
fn test_struct_creation_and_access() {
    test_context!(|test_context: &mut TestContext| {
        // First declare the struct type
        let struct_type = test_context.module().unwrap().get_context().opaque_struct_type("Point");
        struct_type.set_body(&[
            test_context.module().unwrap().get_context().i64_type().into(),
            test_context.module().unwrap().get_context().i64_type().into(),
        ], false);

        let program = ast::Program::from_functions(
            vec![
            ast::Function { 
                is_async: false, 
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    // Create a struct instance
                    Statement::VariableDeclaration { 
                        name: "p".to_string(),
                        type_: Some(AstType::Struct {
                            name: "Point".to_string(),
                            fields: vec![
                                ("x".to_string(), AstType::I64),
                                ("y".to_string(), AstType::I64),
                            ],
                        }),
                        initializer: Some(Expression::StructLiteral {
                            name: "Point".to_string(),
                            fields: vec![
                                ("x".to_string(), Expression::Integer64(10)),
                                ("y".to_string(), Expression::Integer64(20)),
                            ],
                        }),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    // Access struct field
                    Statement::Return(Expression::StructField {
                        struct_: Box::new(Expression::Identifier("p".to_string())),
                        field: "x".to_string(),
                    }),
                ],
            },
        ]);

        let result: i64 = compile_and_run(&mut test_context, &program);
        assert_eq!(result, 10);
    });
}

#[test]
fn test_struct_pointer() {
    test_context!(|test_context: &mut TestContext| {
        let struct_decl = ast::Declaration::Struct(ast::StructDefinition {
            name: "Point".to_string(),
            fields: vec![
                ast::StructField {
                    name: "x".to_string(),
                    type_: ast::AstType::I64,
                    is_mutable: false,
                    default_value: None,
                },
                ast::StructField {
                    name: "y".to_string(),
                    type_: ast::AstType::I64,
                    is_mutable: false,
                    default_value: None,
                },
            ],
        });
        let func = ast::Function { is_async: false, 
            name: "test_struct_ptr".to_string(),
            args: vec![],
            return_type: ast::AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "s".to_string(),
                    type_: Some(AstType::Struct {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), AstType::I64),
                            ("y".to_string(), AstType::I64),
                        ],
                    }),
                    initializer: Some(Expression::StructLiteral {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), Expression::Integer64(10)),
                            ("y".to_string(), Expression::Integer64(20)),
                        ],
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::VariableDeclaration { 
                    name: "ptr".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::Struct {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), AstType::I64),
                            ("y".to_string(), AstType::I64),
                        ],
                    }))),
                    initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("s".to_string())))),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::StructField {
                    struct_: Box::new(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
                    field: "x".to_string(),
                }),
            ],
        };
        let program = ast::Program {
            declarations: vec![struct_decl, ast::Declaration::Function(func)],
        };
        test_context.compile(&program).unwrap();
        let ir = test_context.module().unwrap().print_to_string().to_string();
        assert!(ir.contains("define i64 @test_struct_ptr"));
    });
}

#[test]
fn test_struct_field_assignment() {
    test_context!(|test_context: &mut TestContext| {
        let struct_decl = ast::Declaration::Struct(ast::StructDefinition {
            name: "Point".to_string(),
            fields: vec![
                ast::StructField {
                    name: "x".to_string(),
                    type_: ast::AstType::I64,
                    is_mutable: false,
                    default_value: None,
                },
                ast::StructField {
                    name: "y".to_string(),
                    type_: ast::AstType::I64,
                    is_mutable: false,
                    default_value: None,
                },
            ],
        });
        let func = ast::Function { is_async: false, 
            name: "test_struct_assign".to_string(),
            args: vec![],
            return_type: ast::AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "s".to_string(),
                    type_: Some(AstType::Struct {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), AstType::I64),
                            ("y".to_string(), AstType::I64),
                        ],
                    }),
                    initializer: Some(Expression::StructLiteral {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), Expression::Integer64(10)),
                            ("y".to_string(), Expression::Integer64(20)),
                        ],
                    }),
                    is_mutable: true,
                    declaration_type: VariableDeclarationType::ExplicitMutable,
                },
                Statement::VariableAssignment {
                    name: "s.x".to_string(),
                    value: Expression::Integer64(100),
                },
                Statement::Return(Expression::StructField {
                    struct_: Box::new(Expression::Identifier("s".to_string())),
                    field: "x".to_string(),
                }),
            ],
        };
        let program = ast::Program {
            declarations: vec![struct_decl, ast::Declaration::Function(func)],
        };
        test_context.compile(&program).unwrap();
        let ir = test_context.module().unwrap().print_to_string().to_string();
        assert!(ir.contains("define i64 @test_struct_assign"));
    });
}

#[test]
fn test_loop_construct() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "test_loop".to_string(),
            args: vec![("n".to_string(), AstType::I64)],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "sum".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(0)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::VariableDeclaration { 
                    name: "i".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(0)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Loop { 
                    iterator: None, 
                    condition: Some(Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        op: ast::BinaryOperator::LessThan,
                        right: Box::new(Expression::Identifier("n".to_string())),
                    }),
                    body: vec![
                        Statement::VariableAssignment {
                            name: "sum".to_string(),
                            value: Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("sum".to_string())),
                                op: ast::BinaryOperator::Add,
                                right: Box::new(Expression::Identifier("i".to_string())),
                            },
                        },
                        Statement::VariableAssignment {
                            name: "i".to_string(),
                            value: Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("i".to_string())),
                                op: ast::BinaryOperator::Add,
                                right: Box::new(Expression::Integer64(1)),
                            },
                        },
                    ],
                    label: None,
                },
                Statement::Return(Expression::Identifier("sum".to_string())),
            ],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_string_concatenation() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { is_async: false, 
                name: "concat_strings".to_string(),
                args: vec![
                    ("s1".to_string(), AstType::String),
                    ("s2".to_string(), AstType::String),
                ],
                return_type: AstType::String,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("s1".to_string())),
                    op: ast::BinaryOperator::StringConcat,
                    right: Box::new(Expression::Identifier("s2".to_string())),
                })],
            },
            ast::Function { 
                is_async: false, 
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration { 
                        name: "result".to_string(),
                        type_: Some(AstType::String),
                        initializer: Some(Expression::FunctionCall {
                            name: "concat_strings".to_string(),
                            args: vec![
                                Expression::String("Hello, ".to_string()),
                                Expression::String("World!".to_string()),
                            ],
                        }),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::Integer64(0)),
                ],
            },
        ]);

        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_string_comparison() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { is_async: false, 
                name: "compare_strings".to_string(),
                args: vec![
                    ("s1".to_string(), AstType::String),
                    ("s2".to_string(), AstType::String),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("s1".to_string())),
                    op: ast::BinaryOperator::Equals,
                    right: Box::new(Expression::Identifier("s2".to_string())),
                })],
            },
            ast::Function { 
                is_async: false, 
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Expression::FunctionCall {
                        name: "compare_strings".to_string(),
                        args: vec![
                            Expression::String("hello".to_string()),
                            Expression::String("hello".to_string()),
                        ],
                    }),
                ],
            },
        ]);

        let result: i64 = compile_and_run(&mut test_context, &program);
        assert_eq!(result, 1); // Should return 1 for equal strings

        // Test with different strings
        let program = ast::Program::from_functions(vec![
            ast::Function { is_async: false, 
                name: "compare_strings".to_string(),
                args: vec![
                    ("s1".to_string(), AstType::String),
                    ("s2".to_string(), AstType::String),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("s1".to_string())),
                    op: ast::BinaryOperator::Equals,
                    right: Box::new(Expression::Identifier("s2".to_string())),
                })],
            },
            ast::Function { 
                is_async: false, 
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Expression::FunctionCall {
                        name: "compare_strings".to_string(),
                        args: vec![
                            Expression::String("hello".to_string()),
                            Expression::String("world".to_string()),
                        ],
                    }),
                ],
            },
        ]);

        let result: i64 = compile_and_run(&mut test_context, &program);
        assert_eq!(result, 0); // Should return 0 for different strings
    });
}

#[test]
fn test_string_length() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(
            vec![
            ast::Function { is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "s".to_string(),
                    type_: Some(AstType::String),
                    initializer: Some(Expression::String("hello".to_string())),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::StringLength(Box::new(Expression::Identifier("s".to_string())))),
            ],
        }]); 
        let result: i64 = compile_and_run(&mut test_context, &program);
        assert_eq!(result, 5);
    });
}

#[test]
fn test_string_literal_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::String("hello".to_string()))],
        }]);
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_full_pipeline_zen_syntax() {
    test_context!(|test_context: &mut TestContext| {
        // Test the full pipeline: Zen source → lexer → parser → codegen
        let zen_source = "main = () i32 { 42 }";
        
        // Lex the source
        let lexer = zen::lexer::Lexer::new(zen_source);
        
        // Parse the source
        let mut parser = zen::parser::Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        
        // Verify the program was parsed correctly
        assert_eq!(program.declarations.len(), 1);
        if let ast::Declaration::Function(func) = &program.declarations[0] {
            assert_eq!(func.name, "main");
            assert_eq!(func.return_type, AstType::I32);
            assert_eq!(func.body.len(), 1);
            if let Statement::Expression(Expression::Integer32(42)) = &func.body[0] {
                // Correctly parsed
            } else {
                panic!("Expected Expression(Integer32(42)), got {:?}", func.body[0]);
            }
        } else {
            panic!("Expected Function declaration");
        }
        
        // Compile the program
        let result = test_context.compile(&program);
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        
        // Run the program
        let execution_result = test_context.run();
        assert!(execution_result.is_ok(), "Execution failed: {:?}", execution_result);
        assert_eq!(execution_result.unwrap(), 42);
    });
}

#[test]
fn test_full_pipeline_with_variable() {
    test_context!(|test_context: &mut TestContext| {
        // Test the full pipeline with a variable declaration
        let zen_source = "main = () i32 { x := 42; x }";
        
        // Lex the source
        let lexer = zen::lexer::Lexer::new(zen_source);
        
        // Parse the source
        let mut parser = zen::parser::Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        
        // Verify the program was parsed correctly
        assert_eq!(program.declarations.len(), 1);
        if let ast::Declaration::Function(func) = &program.declarations[0] {
            assert_eq!(func.name, "main");
            assert_eq!(func.return_type, AstType::I32);
            assert_eq!(func.body.len(), 2);
            
            // First statement should be variable declaration
            if let Statement::VariableDeclaration { name, type_, initializer, is_mutable: false, declaration_type: _ } = &func.body[0] {
                assert_eq!(name, "x");
                assert_eq!(*type_, Some(AstType::I32));
                assert!(initializer.is_some());
                if let Some(Expression::Integer32(42)) = initializer {
                    // Correct
                } else {
                    panic!("Expected initializer to be Integer32(42)");
                }
            } else {
                panic!("Expected VariableDeclaration, got {:?}", func.body[0]);
            }
            
            // Second statement should be trailing expression
            if let Statement::Expression(Expression::Identifier(name)) = &func.body[1] {
                assert_eq!(name, "x");
            } else {
                panic!("Expected Expression(Identifier(\"x\")), got {:?}", func.body[1]);
            }
        } else {
            panic!("Expected Function declaration");
        }
        
        // Compile the program
        let result = test_context.compile(&program);
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        
        // Run the program
        let execution_result = test_context.run();
        assert!(execution_result.is_ok(), "Execution failed: {:?}", execution_result);
        assert_eq!(execution_result.unwrap(), 42);
    });
} 