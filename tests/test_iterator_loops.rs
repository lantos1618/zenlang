// Iterator loop syntax has been removed - use items.loop() instead
// This file contains tests for the old iterator loop syntax which is no longer supported

/*
use zen::ast::{Declaration, ExternalFunction, Function, Statement, Expression, AstType, LoopKind, VariableDeclarationType};
use zen::compiler::Compiler;

mod common;
use common::{ExecutionHelper, CapturedOutput};

#[test]
fn test_loop_iterator_array_literal() {
    let helper = ExecutionHelper::new();
    
    // Malloc declaration for arrays
    let malloc_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "malloc".to_string(),
        args: vec![AstType::I64],
        return_type: AstType::Pointer(Box::new(AstType::Void)),
        is_varargs: false,
    });
    
    // sum := 0
    // loop x in [1, 2, 3, 4, 5] {
    //     sum ::= sum + x  
    // }
    // return sum
    let main_function = Function {
        name: "main".to_string(),
        type_params: vec![],
        args: vec![],
        return_type: AstType::I64,
        body: vec![
            Statement::VariableDeclaration {
                name: "sum".to_string(),
                type_: None,
                initializer: Some(Expression::Integer64(0)),
                is_mutable: true,
                declaration_type: VariableDeclarationType::InferredMutable,
            },
            Statement::Loop {
                kind: LoopKind::Iterator {
                    variable: "x".to_string(),
                    iterable: Expression::ArrayLiteral(vec![
                        Expression::Integer64(1),
                        Expression::Integer64(2),
                        Expression::Integer64(3),
                        Expression::Integer64(4),
                        Expression::Integer64(5),
                    ]),
                },
                label: None,
                body: vec![
                    Statement::VariableAssignment {
                        name: "sum".to_string(),
                        value: Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("sum".to_string())),
                            op: zen::ast::BinaryOperator::Add,
                            right: Box::new(Expression::Identifier("x".to_string())),
                        },
                    },
                ],
            },
            Statement::Return(Expression::Identifier("sum".to_string())),
        ],
        is_async: false,
    };
    
    let program = zen::ast::Program {
        declarations: vec![malloc_decl, Declaration::Function(main_function)],
    };
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run iterator loop test");
    output.assert_exit_code(15); // 1+2+3+4+5 = 15
}

#[test]
fn test_loop_iterator_empty_array() {
    let helper = ExecutionHelper::new();
    
    // Malloc declaration for arrays
    let malloc_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "malloc".to_string(),
        args: vec![AstType::I64],
        return_type: AstType::Pointer(Box::new(AstType::Void)),
        is_varargs: false,
    });
    
    // count := 0
    // loop x in [] {
    //     count ::= count + 1
    // }
    // return count
    let main_function = Function {
        name: "main".to_string(),
        type_params: vec![],
        args: vec![],
        return_type: AstType::I64,
        body: vec![
            Statement::VariableDeclaration {
                name: "count".to_string(),
                type_: None,
                initializer: Some(Expression::Integer64(0)),
                is_mutable: true,
                declaration_type: VariableDeclarationType::InferredMutable,
            },
            Statement::Loop {
                kind: LoopKind::Iterator {
                    variable: "x".to_string(),
                    iterable: Expression::ArrayLiteral(vec![]),
                },
                label: None,
                body: vec![
                    Statement::VariableAssignment {
                        name: "count".to_string(),
                        value: Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("count".to_string())),
                            op: zen::ast::BinaryOperator::Add,
                            right: Box::new(Expression::Integer64(1)),
                        },
                    },
                ],
            },
            Statement::Return(Expression::Identifier("count".to_string())),
        ],
        is_async: false,
    };
    
    let program = zen::ast::Program {
        declarations: vec![malloc_decl, Declaration::Function(main_function)],
    };
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run empty iterator loop test");
    output.assert_exit_code(0); // Should not enter loop body
}

#[test]
fn test_loop_iterator_with_printf() {
    let helper = ExecutionHelper::new();
    
    // Malloc declaration for arrays
    let malloc_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "malloc".to_string(),
        args: vec![AstType::I64],
        return_type: AstType::Pointer(Box::new(AstType::Void)),
        is_varargs: false,
    });
    
    // Iterate and print each element
    // loop x in [10, 20, 30] {
    //     printf("value: %d\n", x)
    // }
    // return 0
    
    // Printf declaration
    let printf_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "printf".to_string(),
        args: vec![AstType::String],
        return_type: AstType::I32,
        is_varargs: true,
    });
    
    let main_function = Function {
        name: "main".to_string(),
        type_params: vec![],
        args: vec![],
        return_type: AstType::I64,
        body: vec![
            Statement::Loop {
                kind: LoopKind::Iterator {
                    variable: "x".to_string(),
                    iterable: Expression::ArrayLiteral(vec![
                        Expression::Integer64(10),
                        Expression::Integer64(20),
                        Expression::Integer64(30),
                    ]),
                },
                label: None,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![
                            Expression::String("value: %lld\n".to_string()),
                            Expression::Identifier("x".to_string()),
                        ],
                    }),
                ],
            },
            Statement::Return(Expression::Integer64(0)),
        ],
        is_async: false,
    };
    
    let program = zen::ast::Program {
        declarations: vec![malloc_decl, printf_decl, Declaration::Function(main_function)],
    };
    
    // Compile and verify output
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run iterator loop with printf");
    
    // Verify all values are printed
    output.assert_stdout_contains("value: 10");
    output.assert_stdout_contains("value: 20");
    output.assert_stdout_contains("value: 30");
    output.assert_exit_code(0);
}*/
