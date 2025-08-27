use zen::comptime::ComptimeInterpreter;
use zen::ast::*;
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_comptime_arithmetic() {
    let input = "main = () i32 { return comptime 10 + 20 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    if let Declaration::Function(func) = &program.declarations[0] {
        if let Statement::Return(expr) = &func.body[0] {
            if let Expression::Comptime(inner) = expr {
                let mut evaluator = ComptimeInterpreter::new();
                let result = evaluator.evaluate_expression(inner).unwrap();
                match result {
                    zen::comptime::ComptimeValue::I32(30) => {},
                    _ => panic!("Expected Integer32(30), got {:?}", result)
                }
            }
        }
    }
}

#[test]
fn test_comptime_array_literal() {
    let input = "main = () void { arr := comptime [1, 2, 3] }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    if let Declaration::Function(func) = &program.declarations[0] {
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::Comptime(inner)) = initializer {
                let mut evaluator = ComptimeInterpreter::new();
                let result = evaluator.evaluate_expression(inner).unwrap();
                match result {
                    zen::comptime::ComptimeValue::Array(values) => {
                        assert_eq!(values.len(), 3);
                    },
                    _ => panic!("Expected Array, got {:?}", result)
                }
            }
        }
    }
}

#[test]
fn test_comptime_range() {
    let input = "main = () void { r := comptime 0..5 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    if let Declaration::Function(func) = &program.declarations[0] {
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::Comptime(inner)) = initializer {
                let mut evaluator = ComptimeInterpreter::new();
                let result = evaluator.evaluate_expression(inner).unwrap();
                match result {
                    zen::comptime::ComptimeValue::Array(values) => {
                        assert_eq!(values.len(), 5);
                        // Check it contains [0, 1, 2, 3, 4]
                        for i in 0..5 {
                            match &values[i] {
                                zen::comptime::ComptimeValue::I32(val) => {
                                    assert_eq!(*val, i as i32);
                                },
                                _ => panic!("Expected Integer32 in range")
                            }
                        }
                    },
                    _ => panic!("Expected Array from range, got {:?}", result)
                }
            }
        }
    }
}

#[test]
fn test_comptime_string() {
    let input = r#"main = () void { s := comptime "hello world" }"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    if let Declaration::Function(func) = &program.declarations[0] {
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::Comptime(inner)) = initializer {
                let mut evaluator = ComptimeInterpreter::new();
                let result = evaluator.evaluate_expression(inner).unwrap();
                match result {
                    zen::comptime::ComptimeValue::String(s) => {
                        assert_eq!(s, "hello world");
                    },
                    _ => panic!("Expected String, got {:?}", result)
                }
            }
        }
    }
}

#[test]
fn test_comptime_comparison() {
    let input = "main = () void { b := comptime 10 < 20 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    if let Declaration::Function(func) = &program.declarations[0] {
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::Comptime(inner)) = initializer {
                let mut evaluator = ComptimeInterpreter::new();
                let result = evaluator.evaluate_expression(inner).unwrap();
                match result {
                    zen::comptime::ComptimeValue::Bool(true) => {},
                    _ => panic!("Expected Boolean(true), got {:?}", result)
                }
            }
        }
    }
}

#[test]
fn test_comptime_with_variables() {
    let mut evaluator = ComptimeInterpreter::new();
    
    // Set up a variable in the comptime context
    evaluator.set_variable("x".to_string(), zen::comptime::ComptimeValue::I32(10));
    
    // Create an expression that uses the variable
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Identifier("x".to_string())),
        op: BinaryOperator::Multiply,
        right: Box::new(Expression::Integer32(5))
    };
    
    let result = evaluator.evaluate_expression(&expr).unwrap();
    match result {
        zen::comptime::ComptimeValue::I32(50) => {},
        _ => panic!("Expected Integer32(50), got {:?}", result)
    }
}

#[test]
fn test_comptime_block_evaluation() {
    let input = r#"
comptime {
    x := 10
    y := x * 2
    z := y + 5
}
"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    if let Declaration::ComptimeBlock(statements) = &program.declarations[0] {
        let mut evaluator = ComptimeInterpreter::new();
        
        // Evaluate all statements in the block
        for stmt in statements {
            evaluator.execute_statement(stmt).unwrap();
        }
        
        // Check the final values
        match evaluator.get_variable("x") {
            Some(zen::comptime::ComptimeValue::I32(10)) => {},
            _ => panic!("Expected x = 10")
        }
        match evaluator.get_variable("y") {
            Some(zen::comptime::ComptimeValue::I32(20)) => {},
            _ => panic!("Expected y = 20")
        }
        match evaluator.get_variable("z") {
            Some(zen::comptime::ComptimeValue::I32(25)) => {},
            _ => panic!("Expected z = 25")
        }
    }
}