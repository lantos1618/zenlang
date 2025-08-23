use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use zen::compiler::Compiler;
use inkwell::context::Context;
use inkwell::OptimizationLevel;

fn main() {
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "test", OptimizationLevel::None);
    
    let program = ast::Program::from_functions(vec![ast::Function { 
        is_async: false,
        name: "main".to_string(),
        args: vec![],
        return_type: AstType::I64,
        body: vec![
            Statement::VariableDeclaration { 
                name: "sum".to_string(),
                type_: Some(AstType::I64),
                initializer: Some(Expression::Integer64(0)),
                is_mutable: true,
                declaration_type: VariableDeclarationType::ExplicitMutable,
            },
            Statement::VariableDeclaration { 
                name: "i".to_string(),
                type_: Some(AstType::I64),
                initializer: Some(Expression::Integer64(0)),
                is_mutable: true,
                declaration_type: VariableDeclarationType::ExplicitMutable,
            },
            Statement::Loop {
                condition: Some(Expression::BinaryOp {
                    left: Box::new(Expression::Integer64(0)),
                    op: BinaryOperator::LessThan,
                    right: Box::new(Expression::Integer64(5)),
                }),
                label: None,
                body: vec![
                    Statement::Expression(Expression::Integer64(1)),
                    Statement::Break { label: None },
                ],
            },
            Statement::Return(Expression::Integer64(42)),
        ],
    }]);
    
    println!("Program: {:#?}", program);
    
    match compiler.compile(&program) {
        Ok(_) => {
            println!("Compilation successful!");
            let module = compiler.module();
            module.print_to_stderr();
        }
        Err(e) => {
            println!("Compilation error: {:?}", e);
        }
    }
}