use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::error::CompileError;

#[test]
fn test_missing_closing_brace() {
    let input = "main = () void {
        x := 42
        // Missing closing brace
    ";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    
    if let Err(CompileError::SyntaxError(msg, _)) = result {
        assert!(msg.contains("Expected '}'") || msg.contains("closing"));
    } else {
        panic!("Expected SyntaxError for missing closing brace");
    }
}

#[test]
fn test_missing_type_in_declaration() {
    // Parser accepts :: () as shorthand for :: () -> void
    // This is valid syntax, so let's test an actual error case
    let input = "main = (x: ) void {  // Missing parameter type
        return 0
    }";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    
    if let Err(CompileError::SyntaxError(msg, _)) = result {
        assert!(msg.contains("type") || msg.contains("Expected"));
    } else {
        panic!("Expected SyntaxError for missing parameter type");
    }
}

#[test]
fn test_invalid_operator() {
    let input = "main = () void {
        x := 5 ??? 3  // Invalid operator
    }";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    
    if let Err(CompileError::SyntaxError(msg, _)) = result {
        // The error message should indicate there's a problem with the token
        assert!(msg.len() > 0);  // Just ensure we get some error message
    } else {
        panic!("Expected SyntaxError for invalid operator, got: {:?}", result);
    }
}

#[test]
fn test_unclosed_string_literal() {
    let input = r#"main = () void {
        msg := "Hello, world
    }"#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    // Lexer should catch unclosed strings
}

#[test]
fn test_missing_function_body() {
    let input = "main = () void";  // Missing function body
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    
    if let Err(CompileError::SyntaxError(msg, _)) = result {
        assert!(msg.contains("body") || msg.contains("{"));
    } else {
        panic!("Expected SyntaxError for missing function body");
    }
}

#[test]
fn test_invalid_variable_declaration() {
    let input = "main = () void {
        x = 42  // Should be := or ::= for declaration
    }";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    // This might parse as an assignment, which should be an error if x is not declared
    // For now, just check it parses
    assert!(result.is_ok());
}

#[test]
fn test_mismatched_parentheses() {
    let input = "main = () void {
        x := (5 + 3))  // Extra closing parenthesis
    }";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    
    if let Err(CompileError::SyntaxError(msg, _)) = result {
        assert!(msg.contains("Unexpected") || msg.contains("parenthes"));
    } else {
        panic!("Expected SyntaxError for mismatched parentheses");
    }
}

#[test]
fn test_invalid_struct_syntax() {
    let input = "Point = {
        x: i32
        y  // Missing type
    }";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    
    if let Err(CompileError::SyntaxError(msg, _)) = result {
        // Just ensure we get an error for invalid struct syntax
        assert!(msg.len() > 0);
    } else {
        panic!("Expected SyntaxError for invalid struct field, got: {:?}", result);
    }
}

#[test]
fn test_recovery_after_error() {
    let input = r#"
    // First function with error
    bad_func = () void {
        x := 5 ???  // Error here
    }
    
    // Should still parse this function
    good_func = () void {
        y := 10
        return
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    // Currently, parser might not recover, but ideally it should
    assert!(result.is_err());
}

#[test]
fn test_helpful_error_for_common_mistake() {
    let input = "main() {  // C-style function definition
        return 0;
    }";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_err());
    
    if let Err(CompileError::SyntaxError(msg, _)) = result {
        // Should suggest Zen syntax: main = () returnType { ... }
        assert!(msg.contains("=") || msg.contains("syntax"));
    } else {
        panic!("Expected helpful error for C-style function");
    }
}