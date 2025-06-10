use inkwell::context::Context;
use lynlang::ast::{self, Declaration, ExternalFunction, Function, Statement, Expression, AstType};
use lynlang::compiler::Compiler;

#[test]
fn test_printf() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String], // First arg is format string
                return_type: AstType::Int64,
                is_varargs: true, // printf is variadic
            }),
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Hello from Lynlang!\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
            }),
        ],
    };

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    
    // Check that printf is declared correctly
    assert!(ir.contains("declare i64 @printf(ptr, ...)"));
    
    // Check that main calls printf
    assert!(ir.contains("call i64"));
    assert!(ir.contains("@printf"));
}

#[test]
fn test_external_math_function() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    // Declare sqrt from math.h
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "sqrt".to_string(),
                args: vec![AstType::Float],
                return_type: AstType::Float,
                is_varargs: false,
            }),
            Declaration::Function(Function {
                name: "calculate_distance".to_string(),
                args: vec![("x".to_string(), AstType::Float), ("y".to_string(), AstType::Float)],
                return_type: AstType::Float,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "sum_squares".to_string(),
                        type_: AstType::Float,
                        initializer: Some(Expression::BinaryOp {
                            left: Box::new(Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("x".to_string())),
                                op: ast::BinaryOperator::Multiply,
                                right: Box::new(Expression::Identifier("x".to_string())),
                            }),
                            op: ast::BinaryOperator::Add,
                            right: Box::new(Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("y".to_string())),
                                op: ast::BinaryOperator::Multiply,
                                right: Box::new(Expression::Identifier("y".to_string())),
                            }),
                        }),
                    },
                    Statement::Return(Expression::FunctionCall {
                        name: "sqrt".to_string(),
                        args: vec![Expression::Identifier("sum_squares".to_string())],
                    }),
                ],
            }),
        ],
    };

    compiler.compile_program(&program).unwrap();
    let ir = compiler.module.print_to_string().to_string();
    
    // Check that sqrt is declared
    assert!(ir.contains("declare double @sqrt(double)"));
    
    // Check that calculate_distance calls sqrt
    assert!(ir.contains("call double @sqrt"));
} 