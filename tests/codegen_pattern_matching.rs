use zen::ast::{
    Program, Declaration, Statement, Expression, ConditionalArm, Pattern, AstType
};
use zen::codegen::llvm::LLVMCompiler;
use inkwell::context::Context;

#[test]
fn test_basic_pattern_literal() {
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context, "test_module");
    
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "test_pattern".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Some(Expression::Conditional {
                        scrutinee: Box::new(Expression::Integer32(5)),
                        arms: vec![
                            ConditionalArm {
                                pattern: Pattern::Literal(Expression::Integer32(5)),
                                guard: None,
                                body: Expression::Integer32(100),
                            },
                            ConditionalArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: Expression::Integer32(200),
                            },
                        ],
                    })),
                ],
                is_external: false,
            },
        ],
    };
    
    let result = compiler.compile(&program);
    assert!(result.is_ok());
    
    // Verify the generated IR contains the pattern matching logic
    let ir = compiler.module.print_to_string().to_string();
    assert!(ir.contains("test_pattern"));
    assert!(ir.contains("match") || ir.contains("then") || ir.contains("else"));
}

#[test]
fn test_pattern_with_binding() {
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context, "test_module");
    
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "test_binding".to_string(),
                args: vec![("x".to_string(), AstType::I64)],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Some(Expression::Conditional {
                        scrutinee: Box::new(Expression::Identifier("x".to_string())),
                        arms: vec![
                            ConditionalArm {
                                pattern: Pattern::Identifier("y".to_string()),
                                guard: None,
                                body: Expression::BinaryOp {
                                    left: Box::new(Expression::Identifier("y".to_string())),
                                    op: zen::ast::BinaryOp::Add,
                                    right: Box::new(Expression::Integer32(10)),
                                },
                            },
                        ],
                    })),
                ],
                is_external: false,
            },
        ],
    };
    
    let result = compiler.compile(&program);
    assert!(result.is_ok());
}

#[test]
fn test_pattern_range() {
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context, "test_module");
    
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "test_range".to_string(),
                args: vec![("x".to_string(), AstType::I64)],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Some(Expression::Conditional {
                        scrutinee: Box::new(Expression::Identifier("x".to_string())),
                        arms: vec![
                            ConditionalArm {
                                pattern: Pattern::Range {
                                    start: Box::new(Expression::Integer32(1)),
                                    end: Box::new(Expression::Integer32(10)),
                                    inclusive: false,
                                },
                                guard: None,
                                body: Expression::Integer32(100),
                            },
                            ConditionalArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: Expression::Integer32(200),
                            },
                        ],
                    })),
                ],
                is_external: false,
            },
        ],
    };
    
    let result = compiler.compile(&program);
    assert!(result.is_ok());
}

#[test]
fn test_pattern_with_guard() {
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context, "test_module");
    
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "test_guard".to_string(),
                args: vec![("x".to_string(), AstType::I64)],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Some(Expression::Conditional {
                        scrutinee: Box::new(Expression::Identifier("x".to_string())),
                        arms: vec![
                            ConditionalArm {
                                pattern: Pattern::Identifier("y".to_string()),
                                guard: Some(Expression::BinaryOp {
                                    left: Box::new(Expression::Identifier("y".to_string())),
                                    op: zen::ast::BinaryOp::Greater,
                                    right: Box::new(Expression::Integer32(0)),
                                }),
                                body: Expression::Integer32(100),
                            },
                            ConditionalArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: Expression::Integer32(200),
                            },
                        ],
                    })),
                ],
                is_external: false,
            },
        ],
    };
    
    let result = compiler.compile(&program);
    assert!(result.is_ok());
}

#[test]
fn test_pattern_or() {
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context, "test_module");
    
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "test_or".to_string(),
                args: vec![("x".to_string(), AstType::I64)],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Some(Expression::Conditional {
                        scrutinee: Box::new(Expression::Identifier("x".to_string())),
                        arms: vec![
                            ConditionalArm {
                                pattern: Pattern::Or(vec![
                                    Pattern::Literal(Expression::Integer32(1)),
                                    Pattern::Literal(Expression::Integer32(2)),
                                    Pattern::Literal(Expression::Integer32(3)),
                                ]),
                                guard: None,
                                body: Expression::Integer32(100),
                            },
                            ConditionalArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: Expression::Integer32(200),
                            },
                        ],
                    })),
                ],
                is_external: false,
            },
        ],
    };
    
    let result = compiler.compile(&program);
    assert!(result.is_ok());
}