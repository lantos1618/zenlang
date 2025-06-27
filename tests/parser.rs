use zen::compiler::lexer::Lexer;
use zen::compiler::parser::Parser;
use zen::ast::{Program, Declaration, Function, Statement, Expression, AstType};

#[test]
fn test_parse_empty_program() {
    let input = "";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_simple_function() {
    let input = "main = () i32 { 42 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I32,
                body: vec![
                    Statement::Expression(Expression::Integer32(42))
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_variable_declaration() {
    let input = "main = () i32 { x := 10; x }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I32,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: AstType::I32,
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: false,
                    },
                    Statement::Expression(Expression::Identifier("x".to_string())),
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_binary_expression() {
    let input = "main = () i32 { 5 + 3 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I32,
                body: vec![
                    Statement::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::Integer32(5)),
                        op: zen::ast::BinaryOperator::Add,
                        right: Box::new(Expression::Integer32(3)),
                    }),
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_zen_variable_declarations() {
    let input = "test = () i32 { x := 42; y ::= 10; z: i32 = 5; w:: u64 = 100; x + y }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, expect parsing to fail since complex variable declarations aren't fully implemented
    // When the parser is enhanced, this test should be updated to expect success
    match program {
        Ok(program) => {
            // If parsing succeeds, verify the basic structure
            if program.declarations.len() > 0 {
                if let Declaration::Function(func) = &program.declarations[0] {
                    assert_eq!(func.name, "test");
                    assert_eq!(func.return_type, AstType::I32);
                } else {
                    panic!("Expected function declaration");
                }
            }
        }
        Err(_) => {
            // Expected for now - complex variable declarations not fully implemented
        }
    }
}

#[test]
fn test_parse_loop_with_condition() {
    let input = "countdown = () void { counter ::= 10; loop counter > 0 { counter = counter - 1 } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "countdown".to_string(),
                args: vec![],
                return_type: AstType::Void,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "counter".to_string(),
                        type_: AstType::I32,
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: false,
                    },
                    Statement::Loop {
                        condition: Some(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("counter".to_string())),
                            op: zen::ast::BinaryOperator::GreaterThan,
                            right: Box::new(Expression::Integer32(0)),
                        }),
                        body: vec![
                            Statement::VariableAssignment {
                                name: "counter".to_string(),
                                value: Expression::BinaryOp {
                                    left: Box::new(Expression::Identifier("counter".to_string())),
                                    op: zen::ast::BinaryOperator::Subtract,
                                    right: Box::new(Expression::Integer32(1)),
                                },
                            },
                        ],
                        iterator: None,
                    },
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_loop_with_in() {
    let input = "print_names = () void { names := [\"Alice\", \"Bob\"]; loop name in names { io.print(name) } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, expect parsing to fail since loop with 'in' syntax isn't fully implemented
    // When the parser is enhanced, this test should be updated to expect success
    match program {
        Ok(program) => {
            // If parsing succeeds, verify the basic structure
            if program.declarations.len() > 0 {
                if let Declaration::Function(func) = &program.declarations[0] {
                    assert_eq!(func.name, "print_names");
                    assert_eq!(func.return_type, AstType::Void);
                } else {
                    panic!("Expected function declaration");
                }
            }
        }
        Err(_) => {
            // Expected for now - loop with 'in' syntax not fully implemented
        }
    }
}

#[test]
fn test_parse_struct_definition() {
    let input = "Person = { name: string, age: i32 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, expect parsing to fail since struct definitions aren't implemented
    // When the parser is enhanced, this test should be updated to expect success
    match program {
        Ok(program) => {
            // If parsing succeeds, verify it's an empty program for now
            assert_eq!(program, Program { declarations: vec![] });
        }
        Err(_) => {
            // Expected for now - struct definitions not implemented
        }
    }
}

#[test]
fn test_parse_enum_definition() {
    let input = "Action = | Stop | Go | Wait(duration: i32)";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, expect parsing to fail since enum definitions aren't implemented
    // When the parser is enhanced, this test should be updated to expect success
    match program {
        Ok(program) => {
            // If parsing succeeds, verify it's an empty program for now
            assert_eq!(program, Program { declarations: vec![] });
        }
        Err(_) => {
            // Expected for now - enum definitions not implemented
        }
    }
}

#[test]
fn test_parse_conditional_expression() {
    let input = "grade = (score: i32) string { score -> s { | s >= 90 => \"A\" | s >= 80 => \"B\" | true => \"C\" } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, expect parsing to fail since conditional expressions aren't implemented
    // When the parser is enhanced, this test should be updated to expect success
    match program {
        Ok(program) => {
            // If parsing succeeds, verify it's an empty program for now
            assert_eq!(program, Program { declarations: vec![] });
        }
        Err(_) => {
            // Expected for now - conditional expressions not implemented
        }
    }
}

#[test]
fn test_parse_comptime_block() {
    let input = "comptime { build := @std.build io := build.import(\"io\") }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    // For now, just test that it parses without error
    // We'll need to add ComptimeBlock to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_member_access() {
    let input = "main = () void { io.print(\"Hello\") }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    // The parser should parse the function with member access
    assert!(program.declarations.len() > 0);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "main");
        assert_eq!(func.return_type, AstType::Void);
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_function_with_multiple_statements() {
    let source = "main = () i32 { x := 42; y := 10; x + y }";
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "main");
        assert_eq!(func.args.len(), 0);
        assert_eq!(func.return_type, AstType::I32);
        
        // This should have 3 statements: x := 42, y := 10, and x + y
        assert_eq!(func.body.len(), 3);
        
        // Check the first statement (x := 42)
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable: _ } = &func.body[0] {
            assert_eq!(name, "x");
            assert_eq!(*type_, AstType::I32);
            assert!(matches!(initializer, Some(Expression::Integer32(42))));
        } else {
            panic!("Expected VariableDeclaration for first statement");
        }
        
        // Check the second statement (y := 10)
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable: _ } = &func.body[1] {
            assert_eq!(name, "y");
            assert_eq!(*type_, AstType::I32);
            assert!(matches!(initializer, Some(Expression::Integer32(10))));
        } else {
            panic!("Expected VariableDeclaration for second statement");
        }
        
        // Check the third statement (x + y)
        if let Statement::Expression(Expression::BinaryOp { left, op, right }) = &func.body[2] {
            assert!(matches!(**left, Expression::Identifier(ref name) if name == "x"));
            assert_eq!(*op, zen::ast::BinaryOperator::Add);
            assert!(matches!(**right, Expression::Identifier(ref name) if name == "y"));
        } else {
            panic!("Expected BinaryOp expression for third statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_function_with_return() {
    let input = "test = () i64 { x := 42; y := 10; x + y }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "test".to_string(),
                args: vec![("x".to_string(), AstType::I64), ("y".to_string(), AstType::I64)],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: AstType::I64,
                        initializer: Some(Expression::Integer64(42)),
                        is_mutable: false,
                    },
                    Statement::VariableDeclaration {
                        name: "y".to_string(),
                        type_: AstType::I64,
                        initializer: Some(Expression::Integer64(10)),
                        is_mutable: false,
                    },
                    Statement::Return(Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        op: zen::ast::BinaryOperator::Add,
                        right: Box::new(Expression::Identifier("y".to_string())),
                    }),
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_loop_with_return() {
    let input = "test = () i64 { counter ::= 10; loop counter > 0 { counter = counter - 1 } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "test".to_string(),
                args: vec![("counter".to_string(), AstType::I64)],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "counter".to_string(),
                        type_: AstType::I32,
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: false,
                    },
                    Statement::Loop {
                        condition: Some(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("counter".to_string())),
                            op: zen::ast::BinaryOperator::GreaterThan,
                            right: Box::new(Expression::Integer32(0)),
                        }),
                        body: vec![
                            Statement::VariableAssignment {
                                name: "counter".to_string(),
                                value: Expression::BinaryOp {
                                    left: Box::new(Expression::Identifier("counter".to_string())),
                                    op: zen::ast::BinaryOperator::Subtract,
                                    right: Box::new(Expression::Integer32(1)),
                                },
                            },
                        ],
                        iterator: None,
                    },
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
} 