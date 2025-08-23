extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use test_utils::test_context;

#[test]
fn test_minimal_loop() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { 
            is_async: false,
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                // Simple loop with immediate break
                Statement::Loop {
                    condition: Some(Expression::Boolean(true)),
                    label: None,
                    body: vec![
                        Statement::Break { label: None },
                    ],
                },
                Statement::Return(Expression::Integer64(42)),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

fn main() {
    test_minimal_loop();
    println!("Test passed!");
}