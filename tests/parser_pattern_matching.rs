use zen::ast::{Expression, ConditionalArm, Pattern};
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_conditional_expression_basic() {
    let input = "? x -> val { | 0 => \"zero\" | 1 => \"one\" }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Conditional { scrutinee, arms } = expr {
        assert!(matches!(scrutinee.as_ref(), Expression::Identifier(name) if name == "x"));
        assert_eq!(arms.len(), 2);
        
        // Check first arm
        assert!(matches!(&arms[0].pattern, Pattern::Literal(Expression::Integer32(0))));
        assert!(arms[0].guard.is_none());
        assert!(matches!(&arms[0].body, Expression::String(s) if s == "zero"));
        
        // Check second arm
        assert!(matches!(&arms[1].pattern, Pattern::Literal(Expression::Integer32(1))));
        assert!(arms[1].guard.is_none());
        assert!(matches!(&arms[1].body, Expression::String(s) if s == "one"));
    } else {
        panic!("Expected Conditional expression");
    }
}

#[test]
fn test_parse_conditional_with_wildcard() {
    let input = "? score -> s { | 100 => \"perfect\" | _ => \"other\" }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Conditional { scrutinee, arms } = expr {
        assert!(matches!(scrutinee.as_ref(), Expression::Identifier(name) if name == "score"));
        assert_eq!(arms.len(), 2);
        
        // Check wildcard pattern
        assert!(matches!(&arms[1].pattern, Pattern::Wildcard));
    } else {
        panic!("Expected Conditional expression");
    }
}

#[test]
fn test_parse_conditional_with_guard() {
    // Test with guard conditions (if supported)
    let input = "? n -> x { | val -> val > 10 => \"big\" | _ => \"small\" }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Conditional { scrutinee, arms } = expr {
        assert!(matches!(scrutinee.as_ref(), Expression::Identifier(name) if name == "n"));
        assert_eq!(arms.len(), 2);
        
        // Check guard condition exists
        assert!(arms[0].guard.is_some());
    } else {
        panic!("Expected Conditional expression");
    }
}

#[test]
fn test_parse_nested_conditional() {
    let input = "? x -> val { | 0 => ? y -> y2 { | 1 => \"one\" | _ => \"other\" } | _ => \"default\" }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Conditional { scrutinee: _, arms } = expr {
        assert_eq!(arms.len(), 2);
        
        // Check that first arm body is another conditional
        assert!(matches!(&arms[0].body, Expression::Conditional { .. }));
    } else {
        panic!("Expected Conditional expression");
    }
}

#[test]
fn test_parse_conditional_with_complex_patterns() {
    let input = "? result -> r { | \"success\" => 200 | \"error\" => 500 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::Conditional { scrutinee, arms } = expr {
        assert!(matches!(scrutinee.as_ref(), Expression::Identifier(name) if name == "result"));
        assert_eq!(arms.len(), 2);
        
        // Check string pattern
        assert!(matches!(&arms[0].pattern, Pattern::Literal(Expression::String(s)) if s == "success"));
        assert!(matches!(&arms[0].body, Expression::Integer32(200)));
        
        assert!(matches!(&arms[1].pattern, Pattern::Literal(Expression::String(s)) if s == "error"));
        assert!(matches!(&arms[1].body, Expression::Integer32(500)));
    } else {
        panic!("Expected Conditional expression");
    }
}