mod test_utils;

use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::OptimizationLevel;
use lynlang::ast::{self, AstType, Expression, Statement, BinaryOperator};
use lynlang::compiler::Compiler;
use lynlang::error::CompileError;
use test_utils::TestContext;
use std::error::Error;

// Helper function to compile and execute a program
fn compile_and_run<'ctx>(test_context: &mut TestContext<'ctx>, program: &ast::Program) -> i64 {
    test_context.compile(program).unwrap();
    let execution_engine = test_context.module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let jit_function: JitFunction<unsafe extern "C" fn() -> i64> = unsafe { execution_engine.get_function("main").unwrap() };
    unsafe { jit_function.call() }
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
        let ir = test_context.get_ir();
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
                name: "get_string".to_string(),
                args: vec![],
                return_type: AstType::String,
                body: vec![Statement::Return(Expression::String("Hello, World!".to_string()))],
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "str".to_string(),
                        type_: AstType::String,
                        initializer: Some(Expression::FunctionCall {
                            name: "get_string".to_string(),
                            args: vec![],
                        }),
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
        let program = ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![
                Statement::Return(Expression::Conditional {
                    scrutinee: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Integer64(1)),
                        op: BinaryOperator::Equals,
                        right: Box::new(Expression::Integer64(1)),
                    }),
                    arms: vec![
                        (Expression::Integer64(1), Expression::Integer64(42)),
                        (Expression::Integer64(0), Expression::Integer64(0)),
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
                name: "add".to_string(),
                args: vec![
                    ("a".to_string(), AstType::Int64),
                    ("b".to_string(), AstType::Int64),
                ],
                return_type: AstType::Int64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int64,
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
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_undefined".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::Identifier("x".to_string()))],
    }]);

    let result = test_context.compile(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::UndefinedVariable(name)) => assert_eq!(name, "x"),
        _ => panic!("Expected UndefinedVariable error"),
    }
}

#[test]
fn test_type_mismatch() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_type_mismatch".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::BinaryOp {
            left: Box::new(Expression::Integer64(1)),
            op: ast::BinaryOperator::Add,
            right: Box::new(Expression::Float(2.0)),
        })],
    }]);

    let result = test_context.compile(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::InvalidBinaryOperation { op, left, right }) => {
            assert_eq!(op, "Add");
            assert!(left.contains("IntValue"));
            assert!(right.contains("FloatValue"));
        }
        _ => panic!("Expected InvalidBinaryOperation error"),
    }
}

#[test]
fn test_undefined_function() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_undefined_func".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::FunctionCall {
            name: "nonexistent".to_string(),
            args: vec![],
        })],
    }]);

    let result = test_context.compile(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::UndefinedFunction(name)) => assert_eq!(name, "nonexistent"),
        _ => panic!("Expected UndefinedFunction error"),
    }
}

#[test]
fn test_invalid_function_type() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_invalid_type".to_string(),
        args: vec![("x".to_string(), AstType::Function {
            args: vec![AstType::Int64],
            return_type: Box::new(AstType::Int64),
        })],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::Integer64(42))],
    }]);

    let result = test_context.compile(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::InvalidFunctionType(_)) => (),
        _ => panic!("Expected InvalidFunctionType error"),
    }
}

#[test]
fn test_nested_conditionals() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "main".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::Return(Expression::Conditional {
                scrutinee: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Integer64(1)),
                    op: ast::BinaryOperator::Equals,
                    right: Box::new(Expression::Integer64(1)),
                }),
                arms: vec![
                    (Expression::Integer64(1), Expression::Conditional {
                        scrutinee: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Integer64(2)),
                            op: ast::BinaryOperator::Equals,
                            right: Box::new(Expression::Integer64(2)),
                        }),
                        arms: vec![
                            (Expression::Integer64(1), Expression::Integer64(42)),
                            (Expression::Integer64(0), Expression::Integer64(0)),
                        ],
                    }),
                    (Expression::Integer64(0), Expression::Integer64(0)),
                ],
            }),
        ],
    }]);

    let result: i64 = compile_and_run(&mut test_context, &program);
    assert_eq!(result, 42);
}

#[test]
fn test_function_pointer() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![
        ast::Function {
            name: "add".to_string(),
            args: vec![
                ("a".to_string(), AstType::Int64),
                ("b".to_string(), AstType::Int64),
            ],
            return_type: AstType::Int64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Identifier("a".to_string())),
                op: ast::BinaryOperator::Add,
                right: Box::new(Expression::Identifier("b".to_string())),
            })],
        },
        ast::Function {
            name: "test_func_ptr".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![
                Statement::VariableDeclaration {
                    name: "op".to_string(),
                    type_: AstType::Function {
                        args: vec![AstType::Int64, AstType::Int64],
                        return_type: Box::new(AstType::Int64),
                    },
                    initializer: Some(Expression::Identifier("add".to_string())),
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

    test_context.compile(&program).unwrap();
    let ir = test_context.module.print_to_string().to_string();
    
    // Verify function pointer handling
    assert!(ir.contains("define i64 @test_func_ptr"));
    assert!(ir.contains("call i64")); // Indirect call
}

#[test]
fn test_recursive_function() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![
        ast::Function {
            name: "factorial".to_string(),
            args: vec![("n".to_string(), AstType::Int64)],
            return_type: AstType::Int64,
            body: vec![
                Statement::Return(Expression::Conditional {
                    scrutinee: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("n".to_string())),
                        op: ast::BinaryOperator::Equals,
                        right: Box::new(Expression::Integer64(0)),
                    }),
                    arms: vec![
                        (Expression::Integer64(1), Expression::Integer64(1)),
                        (Expression::Integer64(0), Expression::BinaryOp {
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
                        }),
                    ],
                }),
            ],
        },
        ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
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
}

#[test]
fn test_pointer_operations() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "main".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                type_: AstType::Int64,
                initializer: Some(Expression::Integer64(42)),
            },
            Statement::VariableDeclaration {
                name: "p".to_string(),
                type_: AstType::Pointer(Box::new(AstType::Int64)),
                initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("x".to_string())))),
            },
            Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("p".to_string())))),
        ],
    }]);

    let result: i64 = compile_and_run(&mut test_context, &program);
    assert_eq!(result, 42);
}

#[test]
fn test_pointer_arithmetic() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_ptr_arithmetic".to_string(),
        args: vec![("arr".to_string(), AstType::Pointer(Box::new(AstType::Int64)))],
        return_type: AstType::Int64,
        body: vec![
            // Get pointer to second element
            Statement::VariableDeclaration {
                name: "ptr".to_string(),
                type_: AstType::Pointer(Box::new(AstType::Int64)),
                initializer: Some(Expression::PointerOffset {
                    pointer: Box::new(Expression::Identifier("arr".to_string())),
                    offset: Box::new(Expression::Integer64(1)),
                }),
            },
            // Return dereferenced value
            Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
        ],
    }]);

    test_context.compile(&program).unwrap();
    let ir = test_context.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_ptr_arithmetic"));
    assert!(ir.contains("getelementptr"));
}

#[test]
fn test_pointer_assignment() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_ptr_assign".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                type_: AstType::Int64,
                initializer: Some(Expression::Integer64(42)),
            },
            Statement::VariableDeclaration {
                name: "ptr".to_string(),
                type_: AstType::Pointer(Box::new(AstType::Int64)),
                initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("x".to_string())))),
            },
            Statement::PointerAssignment {
                pointer: Expression::Identifier("ptr".to_string()),
                value: Expression::Integer64(100),
            },
            Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
        ],
    }]);

    test_context.compile(&program).unwrap();
    let ir = test_context.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_ptr_assign"));
    assert!(ir.contains("store i64 100"));
    assert!(ir.contains("load i64"));
}

#[test]
fn test_invalid_dereferencing_non_pointer() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    // Test dereferencing a non-pointer
    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_invalid_deref".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                type_: AstType::Int64,
                initializer: Some(Expression::Integer64(42)),
            },
            Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("x".to_string())))),
        ],
    }]);

    let result = test_context.compile(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::InvalidPointerOperation(msg)) if msg.contains("non-pointer") => (),
        _ => panic!("Expected InvalidPointerOperation error"),
    }
}

#[test]
fn test_void_pointer_declaration() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_void_ptr".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::VariableDeclaration {
                name: "ptr".to_string(),
                type_: AstType::Pointer(Box::new(AstType::Void)),
                initializer: None,
            }],
    }]);

    let result = test_context.compile(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::InvalidPointerOperation(msg)) if msg.contains("pointer to void") => (),
        _ => panic!("Expected InvalidPointerOperation error"),
    }
}

#[test]
fn test_struct_creation_and_access() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    // First declare the struct type
    let struct_type = context.opaque_struct_type("Point");
    struct_type.set_body(&[
        context.i64_type().into(),
        context.i64_type().into(),
    ], false);

    let program = ast::Program::from_functions(vec![
        ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![
                // Create a struct instance
                Statement::VariableDeclaration {
                    name: "p".to_string(),
                    type_: AstType::Struct {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), AstType::Int64),
                            ("y".to_string(), AstType::Int64),
                        ],
                    },
                    initializer: Some(Expression::StructLiteral {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), Expression::Integer64(10)),
                            ("y".to_string(), Expression::Integer64(20)),
                        ],
                    }),
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
}

#[test]
fn test_struct_pointer() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![
        ast::Function {
            name: "test_struct_ptr".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![
                Statement::VariableDeclaration {
                    name: "s".to_string(),
                    type_: AstType::Struct {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), AstType::Int64),
                            ("y".to_string(), AstType::Int64),
                        ],
                    },
                    initializer: Some(Expression::StructLiteral {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), Expression::Integer64(10)),
                            ("y".to_string(), Expression::Integer64(20)),
                        ],
                    }),
                },
                Statement::VariableDeclaration {
                    name: "ptr".to_string(),
                    type_: AstType::Pointer(Box::new(AstType::Struct {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), AstType::Int64),
                            ("y".to_string(), AstType::Int64),
                        ],
                    })),
                    initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("s".to_string())))),
                },
                Statement::Return(Expression::StructField {
                    struct_: Box::new(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
                    field: "x".to_string(),
                }),
            ],
        },
    ]);

    test_context.compile(&program).unwrap();
    let ir = test_context.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_struct_ptr"), "IR should define test function");
    assert!(ir.contains("load ptr"), "IR should load from struct pointer");
    assert!(ir.contains("getelementptr"), "IR should use GEP to access dereferenced struct field");
}

#[test]
fn test_struct_field_assignment() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    // First declare the struct type
    let struct_type = context.opaque_struct_type("Point");
    struct_type.set_body(&[
        context.i64_type().into(),
        context.i64_type().into(),
    ], false);

    let program = ast::Program::from_functions(vec![
        ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![
                // Create a struct instance
                Statement::VariableDeclaration {
                    name: "p".to_string(),
                    type_: AstType::Struct {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), AstType::Int64),
                            ("y".to_string(), AstType::Int64),
                        ],
                    },
                    initializer: Some(Expression::StructLiteral {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), Expression::Integer64(10)),
                            ("y".to_string(), Expression::Integer64(20)),
                        ],
                    }),
                },
                // Assign to struct field
                Statement::VariableAssignment {
                    name: "p".to_string(),
                    value: Expression::StructLiteral {
                        name: "Point".to_string(),
                        fields: vec![
                            ("x".to_string(), Expression::Integer64(42)),
                            ("y".to_string(), Expression::Integer64(20)),
                        ],
                    },
                },
                // Return the updated field
                Statement::Return(Expression::StructField {
                    struct_: Box::new(Expression::Identifier("p".to_string())),
                    field: "x".to_string(),
                }),
            ],
        },
    ]);

    let result: i64 = compile_and_run(&mut test_context, &program);
    assert_eq!(result, 42);
}

#[test]
fn test_loop_construct() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_loop".to_string(),
        args: vec![("n".to_string(), AstType::Int64)],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "sum".to_string(),
                type_: AstType::Int64,
                initializer: Some(Expression::Integer64(0)),
            },
            Statement::VariableDeclaration {
                name: "i".to_string(),
                type_: AstType::Int64,
                initializer: Some(Expression::Integer64(0)),
            },
            Statement::Loop {
                condition: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("i".to_string())),
                    op: ast::BinaryOperator::LessThan,
                    right: Box::new(Expression::Identifier("n".to_string())),
                },
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
            },
            Statement::Return(Expression::Identifier("sum".to_string())),
        ],
    }]);

    test_context.compile(&program).unwrap();
    let ir = test_context.module.print_to_string().to_string();
    
    // Check for loop structure
    assert!(ir.contains("loop_cond:"), "Missing loop condition block");
    assert!(ir.contains("loop_body:"), "Missing loop body block");
    assert!(ir.contains("after_loop:"), "Missing after loop block");
    
    // Check for loop condition
    assert!(ir.contains("icmp slt i64"), "Missing loop condition check");
    
    // Check for loop body operations
    assert!(ir.contains("add i64"), "Missing addition in loop body");
    
    // Check for loop control flow
    assert!(ir.contains("br label %loop_cond"), "Missing branch back to loop condition");
    assert!(ir.contains("br i1"), "Missing conditional branch for loop");
}

#[test]
fn test_string_concatenation() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);
    let program = ast::Program::from_functions(vec![
        ast::Function {
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
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![
                Statement::VariableDeclaration {
                    name: "result".to_string(),
                    type_: AstType::String,
                    initializer: Some(Expression::FunctionCall {
                        name: "concat_strings".to_string(),
                        args: vec![
                            Expression::String("Hello, ".to_string()),
                            Expression::String("World!".to_string()),
                        ],
                    }),
                },
                Statement::Return(Expression::Integer64(0)),
            ],
        },
    ]);

    let result = test_context.compile(&program);
    assert!(result.is_ok());
}

#[test]
fn test_string_comparison() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);
    let program = ast::Program::from_functions(vec![
        ast::Function {
            name: "compare_strings".to_string(),
            args: vec![
                ("s1".to_string(), AstType::String),
                ("s2".to_string(), AstType::String),
            ],
            return_type: AstType::Int64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Identifier("s1".to_string())),
                op: ast::BinaryOperator::Equals,
                right: Box::new(Expression::Identifier("s2".to_string())),
            })],
        },
        ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
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
        ast::Function {
            name: "compare_strings".to_string(),
            args: vec![
                ("s1".to_string(), AstType::String),
                ("s2".to_string(), AstType::String),
            ],
            return_type: AstType::Int64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Identifier("s1".to_string())),
                op: ast::BinaryOperator::Equals,
                right: Box::new(Expression::Identifier("s2".to_string())),
            })],
        },
        ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
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
}

#[test]
fn test_string_length() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);
    let program = ast::Program::from_functions(vec![ast::Function {
        name: "main".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "s".to_string(),
                type_: AstType::String,
                initializer: Some(Expression::String("hello".to_string())),
            },
            Statement::Return(Expression::StringLength(Box::new(Expression::Identifier("s".to_string())))),
        ],
    }]);
    let result: i64 = compile_and_run(&mut test_context, &program);
    assert_eq!(result, 5);
}

#[test]
fn test_string_literal_ir() {
    let context = Context::create();
    let mut test_context = TestContext::new(&context);
    let program = ast::Program::from_functions(vec![ast::Function {
        name: "main".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::String("hello".to_string()))],
    }]);
    let result = test_context.compile(&program);
    assert!(result.is_ok());
} 