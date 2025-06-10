use inkwell::context::Context;
use lynlang::ast::{self, Expression, Statement, AstType};
use lynlang::compiler::Compiler;
use lynlang::error::CompileError;

#[test]
fn test_simple_return() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_return".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::Integer64(42))],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_return"));
    assert!(ir.contains("ret i64 42"));
}

#[test]
fn test_binary_operations() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_binary".to_string(),
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
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_binary"));
    assert!(ir.contains("add i64"));
}

#[test]
fn test_string_literal() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_string".to_string(),
        args: vec![],
        return_type: AstType::String,
        body: vec![Statement::Return(Expression::String("Hello, World!".to_string()))],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define ptr @test_string"));
    assert!(ir.contains("constant [13 x i8]"));
}

#[test]
fn test_conditional_expression() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_conditional".to_string(),
        args: vec![("x".to_string(), AstType::Int64)],
        return_type: AstType::Int64,
        body: vec![
            Statement::Return(Expression::Conditional {
                scrutinee: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: ast::BinaryOperator::Equals,
                    right: Box::new(Expression::Integer64(1)),
                }),
                arms: vec![
                    (Expression::Integer64(1), Expression::Integer64(42)),
                    (Expression::Integer64(0), Expression::Integer64(0)),
                ],
            }),
        ],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_conditional"));
    assert!(ir.contains("icmp eq"));
}

#[test]
fn test_variable_declaration() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_vars".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                type_: AstType::Int64,
                initializer: Some(Expression::Integer64(42)),
            },
            Statement::Return(Expression::Identifier("x".to_string())),
        ],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("store i64 42"));
    assert!(ir.contains("load i64"));
}

#[test]
fn test_function_call() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @add"));
    assert!(ir.contains("call i64 @add"));
}

#[test]
fn test_undefined_variable() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_undefined".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::Identifier("x".to_string()))],
    }]);

    let result = compiler.compile_program(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::UndefinedVariable(name)) => assert_eq!(name, "x"),
        _ => panic!("Expected UndefinedVariable error"),
    }
}

#[test]
fn test_type_mismatch() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    let result = compiler.compile_program(&program);
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
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_undefined_func".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::FunctionCall {
            name: "nonexistent".to_string(),
            args: vec![],
        })],
    }]);

    let result = compiler.compile_program(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::UndefinedFunction(name)) => assert_eq!(name, "nonexistent"),
        _ => panic!("Expected UndefinedFunction error"),
    }
}

#[test]
fn test_invalid_function_type() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_invalid_type".to_string(),
        args: vec![("x".to_string(), AstType::Function {
            args: vec![AstType::Int64],
            return_type: Box::new(AstType::Int64),
        })],
        return_type: AstType::Int64,
        body: vec![Statement::Return(Expression::Integer64(42))],
    }]);

    let result = compiler.compile_program(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::InvalidFunctionType(_)) => (),
        _ => panic!("Expected InvalidFunctionType error"),
    }
}

#[test]
fn test_nested_conditionals() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_nested_ifs".to_string(),
        args: vec![("x".to_string(), AstType::Int64)],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "result".to_string(),
                type_: AstType::Int64,
                initializer: None,
            },
            Statement::Expression(Expression::Conditional {
                scrutinee: Box::new(Expression::Identifier("x".to_string())),
                arms: vec![
                    (Expression::Integer64(1), Expression::Integer64(10)),
                    (Expression::Integer64(2), Expression::Integer64(20)),
                    (Expression::Integer64(3), Expression::Integer64(30)),
                ],
            }),
            Statement::Return(Expression::Identifier("result".to_string())),
        ],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_nested_ifs"));
    assert!(ir.contains("icmp eq"));
}

#[test]
fn test_function_pointer() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    
    // Verify function pointer handling
    assert!(ir.contains("define i64 @test_func_ptr"));
    assert!(ir.contains("call i64")); // Indirect call
}

#[test]
fn test_recursive_function() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
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
                    (
                        Expression::Integer64(0),
                        Expression::BinaryOp {
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
                    ),
                ],
            }),
        ],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @factorial"));
    assert!(ir.contains("call i64 @factorial"));
}

#[test]
fn test_pointer_operations() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_pointers".to_string(),
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
            Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
        ],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_pointers"));
    assert!(ir.contains("alloca ptr")); // Opaque pointer style
    assert!(ir.contains("load i64"));
}

#[test]
fn test_pointer_arithmetic() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_ptr_arithmetic"));
    assert!(ir.contains("getelementptr"));
}

#[test]
fn test_pointer_assignment() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_ptr_assign"));
    assert!(ir.contains("store i64 100"));
    assert!(ir.contains("load i64"));
}

#[test]
fn test_invalid_dereferencing_non_pointer() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    let result = compiler.compile_program(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::InvalidPointerOperation(msg)) if msg.contains("non-pointer") => (),
        _ => panic!("Expected InvalidPointerOperation error"),
    }
}

#[test]
fn test_void_pointer_declaration() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    let result = compiler.compile_program(&program);
    assert!(result.is_err());
    match result {
        Err(CompileError::InvalidPointerOperation(msg)) if msg.contains("pointer to void") => (),
        _ => panic!("Expected InvalidPointerOperation error"),
    }
}

#[test]
fn test_struct_creation_and_access() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_struct".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "point".to_string(),
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
            Statement::Return(Expression::StructField {
                struct_: Box::new(Expression::Identifier("point".to_string())),
                field: "x".to_string(),
            }),
        ],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("%Point = type { i64, i64 }"), "IR should define the Point struct type");
    assert!(ir.contains("define i64 @test_struct"), "IR should define the test_struct function");
    assert!(ir.contains("getelementptr"), "IR should use GEP to access struct field");
}

#[test]
fn test_struct_pointer() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_struct_ptr"), "IR should define test function");
    assert!(ir.contains("load ptr"), "IR should load from struct pointer");
    assert!(ir.contains("getelementptr"), "IR should use GEP to access dereferenced struct field");
}

#[test]
fn test_struct_field_assignment() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program::from_functions(vec![ast::Function {
        name: "test_struct_assign".to_string(),
        args: vec![],
        return_type: AstType::Int64,
        body: vec![
            Statement::VariableDeclaration {
                name: "point".to_string(),
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
                        ("x".to_string(), Expression::Integer64(0)),
                        ("y".to_string(), Expression::Integer64(0)),
                    ],
                }),
            },
            Statement::VariableAssignment {
                name: "point".to_string(),
                value: Expression::StructLiteral {
                    name: "Point".to_string(),
                    fields: vec![
                        ("x".to_string(), Expression::Integer64(42)),
                        ("y".to_string(), Expression::Integer64(24)),
                    ],
                },
            },
            Statement::Return(Expression::StructField {
                struct_: Box::new(Expression::Identifier("point".to_string())),
                field: "x".to_string(),
            }),
        ],
    }]);

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("define i64 @test_struct_assign"));
    assert!(ir.contains("%Point = type { i64, i64 }"));
    assert!(ir.contains("store { i64, i64 }"), "IR should store the whole struct on assignment");
    assert!(ir.contains("getelementptr"), "IR should use GEP to access the field for the return");
}

#[test]
fn test_loop_construct() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

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

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    
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