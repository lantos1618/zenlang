use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{Statement, LoopKind, Expression};

#[test]
fn test_parse_infinite_loop() {
    let input = "test = () void { loop { x := 1 } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    // Check that it parsed as an infinite loop
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::Loop { kind, .. } = &func.body[0] {
            assert!(matches!(kind, LoopKind::Infinite));
        } else {
            panic!("Expected loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_condition_loop() {
    let input = "test = () void { loop x < 10 { x := x + 1 } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    // Check that it parsed as a condition loop
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::Loop { kind, .. } = &func.body[0] {
            assert!(matches!(kind, LoopKind::Condition(_)));
            if let LoopKind::Condition(expr) = kind {
                // Should be a binary op: x < 10
                assert!(matches!(expr, Expression::BinaryOp { .. }));
            }
        } else {
            panic!("Expected loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

// Range loop syntax has been removed - use range().loop() instead
// #[test]
// fn test_parse_range_loop() {
//     let input = "test = () void { loop i in 0..10 { x := i } }";
//     ...
// }

// Inclusive range loop syntax has been removed - use range().loop() instead
// #[test]
// fn test_parse_inclusive_range_loop() {
//     let input = "test = () void { loop i in 1..=5 { x := i } }";
//     ...
// }

// Iterator loop syntax has been removed - use items.loop() instead  
// #[test]
// fn test_parse_iterator_loop() {
//     let input = "test = () void { loop item in items { process(item) } }";
//     ...
// }

#[test]
fn test_loop_with_break_continue() {
    // Test simple break and continue statements in a loop
    let input = r#"
    test = () void { 
        loop {
            break
        }
        loop {
            continue  
        }
    }"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    // Verify the function contains loops with break and continue
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 2);
        
        // First loop should have break
        if let Statement::Loop { body, .. } = &func.body[0] {
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], Statement::Break { .. }));
        } else {
            panic!("Expected first loop statement");
        }
        
        // Second loop should have continue
        if let Statement::Loop { body, .. } = &func.body[1] {
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], Statement::Continue { .. }));
        } else {
            panic!("Expected second loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}