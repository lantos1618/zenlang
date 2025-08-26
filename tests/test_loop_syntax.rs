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

#[test]
fn test_parse_range_loop() {
    let input = "test = () void { loop i in 0..10 { x := i } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    // Check that it parsed as a range loop
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::Loop { kind, .. } = &func.body[0] {
            if let LoopKind::Range { variable, start, end, inclusive } = kind {
                assert_eq!(variable, "i");
                assert!(!inclusive);
                // Check start is 0
                if let Expression::Integer32(val) = start {
                    assert_eq!(*val, 0);
                }
                // Check end is 10
                if let Expression::Integer32(val) = end {
                    assert_eq!(*val, 10);
                }
            } else {
                panic!("Expected range loop kind");
            }
        } else {
            panic!("Expected loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_inclusive_range_loop() {
    let input = "test = () void { loop i in 1..=5 { x := i } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    // Check that it parsed as an inclusive range loop
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::Loop { kind, .. } = &func.body[0] {
            if let LoopKind::Range { variable, start, end, inclusive } = kind {
                assert_eq!(variable, "i");
                assert!(inclusive); // Should be inclusive
                // Check start is 1
                if let Expression::Integer32(val) = start {
                    assert_eq!(*val, 1);
                }
                // Check end is 5
                if let Expression::Integer32(val) = end {
                    assert_eq!(*val, 5);
                }
            } else {
                panic!("Expected range loop kind");
            }
        } else {
            panic!("Expected loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_iterator_loop() {
    let input = "test = () void { loop item in items { process(item) } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    // Check that it parsed as an iterator loop
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::Loop { kind, .. } = &func.body[0] {
            if let LoopKind::Iterator { variable, iterable } = kind {
                assert_eq!(variable, "item");
                // Check iterable is "items"
                if let Expression::Identifier(name) = iterable {
                    assert_eq!(name, "items");
                }
            } else {
                panic!("Expected iterator loop kind");
            }
        } else {
            panic!("Expected loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_loop_with_break_continue() {
    let input = r#"
    test = () void { 
        loop i in 0..10 { 
            loop i == 5 => break
            loop i == 3 => continue
            x := i
        } 
    }"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    // Verify the loop contains break and continue
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let Statement::Loop { kind, body, .. } = &func.body[0] {
            assert!(matches!(kind, LoopKind::Range { .. }));
            assert_eq!(body.len(), 3);
        } else {
            panic!("Expected loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}