use zen::ast::{Expression, Program, Declaration, Function, Statement, AstType};
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_range_exclusive() {
    let input = "test = () i32 { x := 0..10; x }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 2);
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::Range { start, end, inclusive }) = initializer {
                assert!(matches!(**start, Expression::Integer32(0)));
                assert!(matches!(**end, Expression::Integer32(10)));
                assert_eq!(*inclusive, false);
            } else {
                panic!("Expected range expression, got {:?}", initializer);
            }
        }
    }
}

#[test]
fn test_parse_range_inclusive() {
    let input = "test = () i32 { x := 0..=10; x }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 2);
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::Range { start, end, inclusive }) = initializer {
                assert!(matches!(**start, Expression::Integer32(0)));
                assert!(matches!(**end, Expression::Integer32(10)));
                assert_eq!(*inclusive, true);
            } else {
                panic!("Expected range expression, got {:?}", initializer);
            }
        }
    }
}

#[test]
fn test_parse_range_with_expressions() {
    let input = "test = () i32 { x := 1+2..10*2; x }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 2);
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::Range { start, end, inclusive }) = initializer {
                // Start should be 1+2
                assert!(matches!(**start, Expression::BinaryOp { .. }));
                // End should be 10*2
                assert!(matches!(**end, Expression::BinaryOp { .. }));
                assert_eq!(*inclusive, false);
            } else {
                panic!("Expected range expression, got {:?}", initializer);
            }
        }
    }
}

#[test]
fn test_parse_range_with_method_call() {
    let input = "test = () void { x := (0..10).loop; x }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 2); // variable declaration and return
        if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(Expression::MemberAccess { object, member }) = initializer {
                assert_eq!(member, "loop");
                // Object should be a range expression wrapped in parentheses
                if let Expression::Range { start, end, inclusive } = &**object {
                    assert!(matches!(**start, Expression::Integer32(0)));
                    assert!(matches!(**end, Expression::Integer32(10)));
                    assert_eq!(*inclusive, false);
                } else {
                    panic!("Expected range expression as object, got {:?}", object);
                }
            } else {
                panic!("Expected member access expression in initializer, got {:?}", initializer);
            }
        } else {
            panic!("Expected variable declaration as first statement");
        }
    }
} 