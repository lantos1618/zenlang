extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
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
fn test_struct_creation_and_access() {
    test_context!(|test_context: &mut TestContext| {
        let struct_decl = ast::Declaration::Struct(ast::StructDefinition {
            name: "Point".to_string(),
            type_params: vec![],
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
            methods: vec![],
        });
        let func = ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
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
                Statement::Return(Expression::StructField {
                    struct_: Box::new(Expression::Identifier("s".to_string())),
                    field: "x".to_string(),
                }),
            ],
        };
        let program = ast::Program {
            declarations: vec![struct_decl, ast::Declaration::Function(func)],
        };
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_struct_pointer() {
    test_context!(|test_context: &mut TestContext| {
        let struct_decl = ast::Declaration::Struct(ast::StructDefinition {
            name: "Point".to_string(),
            type_params: vec![],
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
            methods: vec![],
        });
        let func = ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
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
                    name: "p".to_string(),
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
                    struct_: Box::new(Expression::Dereference(Box::new(Expression::Identifier("p".to_string())))),
                    field: "x".to_string(),
                }),
            ],
        };
        let program = ast::Program {
            declarations: vec![struct_decl, ast::Declaration::Function(func)],
        };
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_struct_field_assignment() {
    test_context!(|test_context: &mut TestContext| {
        let struct_decl = ast::Declaration::Struct(ast::StructDefinition {
            name: "Point".to_string(),
            type_params: vec![],
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
            methods: vec![],
        });
        let func = ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
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
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
} 