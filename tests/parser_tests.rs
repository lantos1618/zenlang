use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_range_loop() {
    let code = r#"
main = () void {
    (0..3).loop((i) {
        io.println("Test: ${i}")
    })
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            // Verify we got a program with declarations
            assert!(
                !program.declarations.is_empty(),
                "Should have declarations (main function)"
            );

            // Verify first declaration is a function
            match &program.declarations[0] {
                zen::ast::Declaration::Function { .. } => {
                    // Good - it's a function declaration
                }
                other => {
                    panic!("Expected function declaration, got: {:?}", other);
                }
            }
        }
        Err(e) => {
            panic!("Parse error: {:?}", e);
        }
    }
}

#[test]
fn test_parse_ternary_with_comparison() {
    // Test that comparison operators followed by ternary operator parse correctly
    // This tests expressions like "x > y ? | true { } | false { }"
    let code = r#"
test = () void {
    x = 5
    y = 3
    result = x > y ? | true { 1 } | false { 0 }
    result2 = x >= y ? | true { 1 } | false { 0 }
    result3 = x < y ? | true { 1 } | false { 0 }
    result4 = x <= y ? | true { 1 } | false { 0 }
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(_program) => {
            // If parsing succeeds, the fix is working
        }
        Err(e) => {
            panic!("Parse error for ternary with comparison: {:?}", e);
        }
    }
}
#[test]
fn test_ternary_conditional_with_braces() {
    // Test ternary syntax: expr ? { true_block } : { false_block }
    let code = r#"
test = () void {
    x: bool = true
    val = x ? { 1 } : { 0 }
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            // If parsing succeeds, the ternary fix is working
            assert!(!program.declarations.is_empty(), "Should have declarations");
        }
        Err(e) => {
            panic!("Parse error for ternary conditional: {:?}", e);
        }
    }
}

#[test]
fn test_pattern_match_still_works() {
    // Ensure pattern matching with | still works
    let code = r#"
test = () void {
    x = 5
    result = x ? | 1 { "one" } | 2 { "two" } | _ { "other" }
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            assert!(!program.declarations.is_empty(), "Should have declarations");
        }
        Err(e) => {
            panic!("Parse error for pattern match: {:?}", e);
        }
    }
}

#[test]
fn test_not_operator_basic() {
    // Test basic ! operator: !true
    let code = r#"
test = () void {
    x = !true
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            assert!(!program.declarations.is_empty(), "Should have declarations");
        }
        Err(e) => {
            panic!("Parse error for ! operator: {:?}", e);
        }
    }
}

#[test]
fn test_not_operator_with_variable() {
    // Test ! operator with variable: !x
    let code = r#"
test = () void {
    x: bool = true
    y = !x
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            assert!(!program.declarations.is_empty(), "Should have declarations");
        }
        Err(e) => {
            panic!("Parse error for !variable: {:?}", e);
        }
    }
}

#[test]
fn test_not_operator_with_method_call() {
    // Test ! operator with method call: !foo.bar
    let code = r#"
test = () void {
    x = !entry.occupied()
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            assert!(!program.declarations.is_empty(), "Should have declarations");
        }
        Err(e) => {
            panic!("Parse error for !method_call: {:?}", e);
        }
    }
}

#[test]
fn test_not_operator_nested() {
    // Test nested ! operator: !!x
    let code = r#"
test = () void {
    x: bool = true
    y = !!x
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            assert!(!program.declarations.is_empty(), "Should have declarations");
        }
        Err(e) => {
            panic!("Parse error for nested ! operator: {:?}", e);
        }
    }
}

#[test]
fn test_not_equal_still_works() {
    // Verify != operator still works correctly
    let code = r#"
test = () void {
    x = 5
    y = 3
    result = x != y
}
"#;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            assert!(!program.declarations.is_empty(), "Should have declarations");
        }
        Err(e) => {
            panic!("Parse error for != operator: {:?}", e);
        }
    }
}
