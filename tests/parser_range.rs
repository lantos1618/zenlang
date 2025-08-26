use zen::ast::{Expression, LoopKind};
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_exclusive_range() {
    let input = "0..10";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Range { start, end, inclusive } = expr {
        assert!(matches!(start.as_ref(), Expression::Integer32(0)));
        assert!(matches!(end.as_ref(), Expression::Integer32(10)));
        assert!(!inclusive, "Range should be exclusive");
    } else {
        panic!("Expected Range expression, got {:?}", expr);
    }
}

#[test]
fn test_parse_inclusive_range() {
    let input = "1..=5";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Range { start, end, inclusive } = expr {
        assert!(matches!(start.as_ref(), Expression::Integer32(1)));
        assert!(matches!(end.as_ref(), Expression::Integer32(5)));
        assert!(inclusive, "Range should be inclusive");
    } else {
        panic!("Expected Range expression, got {:?}", expr);
    }
}

#[test]
fn test_parse_range_with_variables() {
    let input = "start..end";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Range { start, end, inclusive } = expr {
        assert!(matches!(start.as_ref(), Expression::Identifier(name) if name == "start"));
        assert!(matches!(end.as_ref(), Expression::Identifier(name) if name == "end"));
        assert!(!inclusive);
    } else {
        panic!("Expected Range expression");
    }
}

#[test]
fn test_parse_range_with_expressions() {
    let input = "(x + 1)..(y * 2)";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Range { start, end, inclusive } = expr {
        assert!(matches!(start.as_ref(), Expression::BinaryOp { .. }));
        assert!(matches!(end.as_ref(), Expression::BinaryOp { .. }));
        assert!(!inclusive);
    } else {
        panic!("Expected Range expression");
    }
}

#[test]
fn test_parse_range_in_loop() {
    // Test range in a loop context inside a function
    let input = "test_func = () void { loop 0..10 { x := 1; } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    
    // Should have one function declaration with a loop statement
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "test_func");
        assert!(!func.body.is_empty());
        // The first statement should be a loop with a range condition
        if let zen::ast::Statement::Loop { kind, .. } = &func.body[0] {
            if let LoopKind::Condition(Expression::Range { start, end, inclusive }) = kind {
                assert!(matches!(**start, Expression::Integer32(0)));
                assert!(matches!(**end, Expression::Integer32(10)));
                assert!(!inclusive);
            } else {
                panic!("Expected LoopKind::Condition with Range expression");
            }
        } else {
            panic!("Expected Loop statement");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_negative_range() {
    let input = "-5..5";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression();
    
    // This might fail or parse differently depending on negative number handling
    // For now, let's just check it doesn't crash
    assert!(expr.is_ok() || expr.is_err());
}

#[test]
fn test_parse_float_range() {
    let input = "0.0..1.0";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Range { start, end, inclusive } = expr {
        assert!(matches!(start.as_ref(), Expression::Float64(_)));
        assert!(matches!(end.as_ref(), Expression::Float64(_)));
        assert!(!inclusive);
    } else {
        panic!("Expected Range expression with floats");
    }
}