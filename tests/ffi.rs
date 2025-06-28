use inkwell::context::Context;
use zen::ast::{self, Declaration, ExternalFunction, Function, Statement, Expression, AstType, VariableDeclarationType};
use zen::compiler::Compiler;

#[test]
fn test_external_function_declaration() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String],
                return_type: AstType::I64,
                is_varargs: true,
            }),
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Hello, World!\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    let ir = compiler.compile_llvm(&program).unwrap();
    println!("IR for external function declaration:");
    println!("{}", ir);
}

#[test]
fn test_float_operations() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "add_floats".to_string(),
                args: vec![AstType::F64, AstType::F64],
                return_type: AstType::F64,
                is_varargs: false,
            }),
            Declaration::Function(Function {
                name: "test_float_math".to_string(),
                args: vec![("x".to_string(), AstType::F64), ("y".to_string(), AstType::F64)],
                return_type: AstType::F64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "result".to_string(),
                        type_: Some(AstType::F64),
                        initializer: Some(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("x".to_string())),
                            op: ast::BinaryOperator::Add,
                            right: Box::new(Expression::Identifier("y".to_string())),
                        }),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::Identifier("result".to_string())),
                ],
                is_async: false,
            }),
        ],
    };

    let ir = compiler.compile_llvm(&program).unwrap();
    println!("IR for float operations:");
    println!("{}", ir);
}

#[test]
fn test_printf() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);

    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String], // First arg is format string
                return_type: AstType::I64,
                is_varargs: true, // printf is variadic
            }),
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Hello from Zen!\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    let ir = compiler.compile_llvm(&program).unwrap();
    println!("IR for printf:");
    println!("{}", ir);
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
                args: vec![AstType::F64],
                return_type: AstType::F64,
                is_varargs: false,
            }),
            Declaration::Function(Function {
                name: "calculate_distance".to_string(),
                args: vec![("x".to_string(), AstType::F64), ("y".to_string(), AstType::F64)],
                return_type: AstType::F64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "sum_squares".to_string(),
                        type_: Some(AstType::F64),
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
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::FunctionCall {
                        name: "sqrt".to_string(),
                        args: vec![Expression::Identifier("sum_squares".to_string())],
                    }),
                ],
                is_async: false,
            }),
        ],
    };

    let ir = compiler.compile_llvm(&program).unwrap();
    println!("IR for external math function:");
    println!("{}", ir);
} 