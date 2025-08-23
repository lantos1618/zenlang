use zen::ast::{Expression, Declaration, Statement, VariableDeclarationType};
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_comptime_block() {
    let input = "comptime { x := 42 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::ComptimeBlock(statements) = &program.declarations[0] {
        assert_eq!(statements.len(), 1);
        if let Statement::VariableDeclaration { name, value, .. } = &statements[0] {
            assert_eq!(name, "x");
            assert!(matches!(value, Expression::Integer32(42)));
        } else {
            panic!("Expected variable declaration in comptime block");
        }
    } else {
        panic!("Expected ComptimeBlock declaration");
    }
}

#[test]
fn test_parse_comptime_expression() {
    let input = "main = () void { x := comptime 42 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::VariableDeclaration { value, .. } = &func.body[0] {
            if let Expression::Comptime(inner) = value {
                assert!(matches!(inner.as_ref(), Expression::Integer32(42)));
            } else {
                panic!("Expected Comptime expression, got {:?}", value);
            }
        } else {
            panic!("Expected variable declaration");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_comptime_function_call() {
    let input = "main = () void { result := comptime calculate() }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::VariableDeclaration { value, .. } = &func.body[0] {
            if let Expression::Comptime(inner) = value {
                assert!(matches!(inner.as_ref(), Expression::FunctionCall { name, .. } if name == "calculate"));
            } else {
                panic!("Expected Comptime expression");
            }
        } else {
            panic!("Expected variable declaration");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_nested_comptime_blocks() {
    let input = r#"
comptime {
    x := 10
    comptime {
        y := 20
    }
}
"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::ComptimeBlock(outer_statements) = &program.declarations[0] {
        assert_eq!(outer_statements.len(), 2);
        
        // Check nested comptime block
        if let Statement::ComptimeBlock(inner_statements) = &outer_statements[1] {
            assert_eq!(inner_statements.len(), 1);
            if let Statement::VariableDeclaration { name, .. } = &inner_statements[0] {
                assert_eq!(name, "y");
            } else {
                panic!("Expected variable declaration in nested comptime block");
            }
        } else {
            panic!("Expected nested ComptimeBlock statement");
        }
    } else {
        panic!("Expected ComptimeBlock declaration");
    }
}

#[test]
fn test_parse_comptime_with_multiple_statements() {
    let input = r#"
comptime {
    x := 42
    y := x * 2
    z := "hello"
}
"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::ComptimeBlock(statements) = &program.declarations[0] {
        assert_eq!(statements.len(), 3);
        
        // Check all three variable declarations
        for (i, expected_name) in ["x", "y", "z"].iter().enumerate() {
            if let Statement::VariableDeclaration { name, .. } = &statements[i] {
                assert_eq!(name, expected_name);
            } else {
                panic!("Expected variable declaration at index {}", i);
            }
        }
    } else {
        panic!("Expected ComptimeBlock declaration");
    }
}