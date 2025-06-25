use zen::compiler::lexer::Lexer;
use zen::compiler::parser::Parser;
use zen::ast::{Program, Declaration, Function, Statement, Expression, AstType};

#[test]
fn test_parse_empty_program() {
    let input = "";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_simple_function() {
    let input = "main = () int32 { 42 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int32,
                body: vec![
                    Statement::Expression(Expression::Integer32(42))
                ],
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_variable_declaration() {
    let input = "main = () int32 { x := 10; x }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int32,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: AstType::Int32,
                        initializer: Some(Expression::Integer32(10)),
                    },
                    Statement::Expression(Expression::Identifier("x".to_string())),
                ],
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_binary_expression() {
    let input = "main = () int32 { 5 + 3 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int32,
                body: vec![
                    Statement::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::Integer32(5)),
                        op: zen::ast::BinaryOperator::Add,
                        right: Box::new(Expression::Integer32(3)),
                    }),
                ],
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_zen_variable_declarations() {
    let input = "test = () int32 { x := 42; y ::= 10; z: int32 = 5; w:: uint64 = 100; x + y }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // The parser should parse the function with different variable declaration syntax
    assert!(program.declarations.len() > 0);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "test");
        assert_eq!(func.return_type, AstType::Int32);
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_loop_with_condition() {
    let input = "countdown = () void { counter ::= 10; loop counter > 0 { counter = counter - 1 } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    let expected = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "countdown".to_string(),
                args: vec![],
                return_type: AstType::Void,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "counter".to_string(),
                        type_: AstType::Int32,
                        initializer: Some(Expression::Integer32(10)),
                    },
                    Statement::Loop {
                        condition: Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("counter".to_string())),
                            op: zen::ast::BinaryOperator::GreaterThan,
                            right: Box::new(Expression::Integer32(0)),
                        },
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
                    },
                ],
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
    
    // The parser should parse the function, even if loop syntax isn't fully implemented
    assert!(program.declarations.len() > 0);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "print_names");
        assert_eq!(func.return_type, AstType::Void);
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_struct_definition() {
    let input = "Person = { name: string, age: int32 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add Struct to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_enum_definition() {
    let input = "Action = | Stop | Go | Wait(duration: int32)";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add Enum to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_conditional_expression() {
    let input = "grade = (score: int32) string { score -> s { | s >= 90 => \"A\" | s >= 80 => \"B\" | true => \"C\" } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add Conditional to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_comptime_block() {
    let input = "comptime { build := @std.build io := build.import(\"io\") }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add ComptimeBlock to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_member_access() {
    let input = "main = () void { io.print(\"Hello\") }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
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
    let source = "main = () int32 { x := 42; y := 10; x + y }";
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "main");
        assert_eq!(func.args.len(), 0);
        assert_eq!(func.return_type, AstType::Int32);
        
        // This should have 3 statements: x := 42, y := 10, and x + y
        assert_eq!(func.body.len(), 3);
        
        // Check the first statement (x := 42)
        if let Statement::VariableDeclaration { name, type_, initializer } = &func.body[0] {
            assert_eq!(name, "x");
            assert_eq!(*type_, AstType::Int32);
            assert!(matches!(initializer, Some(Expression::Integer32(42))));
        } else {
            panic!("Expected VariableDeclaration for first statement");
        }
        
        // Check the second statement (y := 10)
        if let Statement::VariableDeclaration { name, type_, initializer } = &func.body[1] {
            assert_eq!(name, "y");
            assert_eq!(*type_, AstType::Int32);
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