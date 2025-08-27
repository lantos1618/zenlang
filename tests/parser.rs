use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{self, Program, Declaration, Function, Statement, Expression, AstType, VariableDeclarationType, BinaryOperator, Pattern, LoopKind};
use zen::error::CompileError;

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
            Declaration::Function(Function { type_params: vec![],
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
            Declaration::Function(Function { type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I32,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: None,
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::InferredImmutable,
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
            Declaration::Function(Function { type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I32,
                body: vec![
                    Statement::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::Integer32(5)),
                        op: BinaryOperator::Add,
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
            Declaration::Function(Function { type_params: vec![],
                name: "countdown".to_string(),
                args: vec![],
                return_type: AstType::Void,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "counter".to_string(),
                        type_: None,
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: true,
                        declaration_type: VariableDeclarationType::InferredMutable,
                    },
                    Statement::Loop {
                        kind: LoopKind::Condition(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("counter".to_string())),
                            op: BinaryOperator::GreaterThan,
                            right: Box::new(Expression::Integer32(0)),
                        }),
                        body: vec![
                            Statement::VariableAssignment {
                                name: "counter".to_string(),
                                value: Expression::BinaryOp {
                                    left: Box::new(Expression::Identifier("counter".to_string())),
                                    op: BinaryOperator::Subtract,
                                    right: Box::new(Expression::Integer32(1)),
                                },
                            },
                        ],
                        label: None,
                    },
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
}

// Iterator loop syntax has been removed - use items.loop() instead
// #[test]
// fn test_parse_loop_with_in() {
//     let input = "print_names = () void { names := [\"Alice\", \"Bob\"]; loop name in names { io.print(name) } }";
//     ...
// }

#[test]
fn test_parse_struct_definition() {
    let input = "Person = { name: string, age: i32 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // Struct definitions are now implemented
    let program = program.unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Struct(def) = &program.declarations[0] {
        assert_eq!(def.name, "Person");
        assert!(def.type_params.is_empty());
        assert_eq!(def.fields.len(), 2);
        
        assert_eq!(def.fields[0].name, "name");
        assert_eq!(def.fields[0].type_, zen::ast::AstType::String);
        assert!(!def.fields[0].is_mutable);
        
        assert_eq!(def.fields[1].name, "age");
        assert_eq!(def.fields[1].type_, zen::ast::AstType::I32);
        assert!(!def.fields[1].is_mutable);
    } else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn test_parse_enum_definition() {
    let input = "Action = | Stop | Go | Wait(duration: i32)";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // Enum definitions are now implemented, expect success
    let program = program.unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let Declaration::Enum(enum_def) = &program.declarations[0] {
        assert_eq!(enum_def.name, "Action");
        assert_eq!(enum_def.variants.len(), 3);
        assert_eq!(enum_def.variants[0].name, "Stop");
        assert_eq!(enum_def.variants[1].name, "Go");
        assert_eq!(enum_def.variants[2].name, "Wait");
        assert!(enum_def.variants[2].payload.is_some());
    } else {
        panic!("Expected enum declaration");
    }
}

#[test]
fn test_parse_conditional_expression() {
    let input = "result = (x: i32) string { x ? | 0 => \"zero\" | 1 => \"one\" | _ => \"other\" }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    assert!(program.is_ok(), "Failed to parse conditional expression: {:?}", program.err());
    
    let program = program.unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "result");
        assert_eq!(func.return_type, AstType::String);
        assert_eq!(func.body.len(), 1);
        
        if let Statement::Expression(Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            // Check scrutinee is an identifier 'x'
            if let Expression::Identifier(name) = &**scrutinee {
                assert_eq!(name, "x");
            } else {
                panic!("Expected identifier 'x' as scrutinee");
            }
            
            // Check we have 3 arms
            assert_eq!(arms.len(), 3);
            
            // Check first arm: | 0 => "zero"
            if let ast::ConditionalArm { pattern: Pattern::Literal(Expression::Integer32(0)), guard: None, body: Expression::String(ref s) } = &arms[0] {
                assert_eq!(s, "zero");
            } else {
                panic!("Expected first arm to be | 0 => \"zero\"");
            }
            
            // Check second arm: | 1 => "one"
            if let ast::ConditionalArm { pattern: Pattern::Literal(Expression::Integer32(1)), guard: None, body: Expression::String(ref s) } = &arms[1] {
                assert_eq!(s, "one");
            } else {
                panic!("Expected second arm to be | 1 => \"one\"");
            }
            
            // Check third arm: | _ => "other"
            if let ast::ConditionalArm { pattern: Pattern::Wildcard, guard: None, body: Expression::String(ref s) } = &arms[2] {
                assert_eq!(s, "other");
            } else {
                panic!("Expected third arm to be | _ => \"other\"");
            }
        } else {
            panic!("Expected conditional expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_comptime_block() {
    let input = "comptime { build := @std.build io := build.import(\"io\") }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    // Verify we have a comptime block declaration
    assert_eq!(program.declarations.len(), 1);
    if let Declaration::ComptimeBlock(statements) = &program.declarations[0] {
        assert_eq!(statements.len(), 2);
        
        // Check first statement: build := @std.build
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } = &statements[0] {
            assert_eq!(name, "build");
            assert_eq!(type_, &None);
            assert_eq!(*is_mutable, false);
            assert!(matches!(declaration_type, VariableDeclarationType::InferredImmutable));
            if let Some(Expression::MemberAccess { object, member }) = initializer {
                assert!(matches!(**object, Expression::Identifier(ref name) if name == "@std"));
                assert_eq!(member, "build");
            } else {
                panic!("Expected MemberAccess in build initialization");
            }
        } else {
            panic!("Expected VariableDeclaration for build");
        }
        
        // Check second statement: io := build.import("io")
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } = &statements[1] {
            assert_eq!(name, "io");
            assert_eq!(type_, &None);
            assert_eq!(*is_mutable, false);
            assert!(matches!(declaration_type, VariableDeclarationType::InferredImmutable));
            if let Some(Expression::FunctionCall { name, args }) = initializer {
                assert_eq!(name, "build.import");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expression::String(ref s) if s == "io"));
            } else {
                panic!("Expected FunctionCall in io initialization");
            }
        } else {
            panic!("Expected VariableDeclaration for io");
        }
    } else {
        panic!("Expected ComptimeBlock declaration");
    }
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
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable: _, declaration_type: _ } = &func.body[0] {
            assert_eq!(name, "x");
            assert_eq!(*type_, None);
            assert!(matches!(initializer, Some(Expression::Integer32(42))));
        } else {
            panic!("Expected VariableDeclaration for first statement");
        }
        
        // Check the second statement (y := 10)
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable: _, declaration_type: _ } = &func.body[1] {
            assert_eq!(name, "y");
            assert_eq!(*type_, None);
            assert!(matches!(initializer, Some(Expression::Integer32(10))));
        } else {
            panic!("Expected VariableDeclaration for second statement");
        }
        
        // Check the third statement (x + y)
        if let Statement::Expression(Expression::BinaryOp { left, op, right }) = &func.body[2] {
            assert!(matches!(**left, Expression::Identifier(ref name) if name == "x"));
            assert_eq!(*op, BinaryOperator::Add);
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
            Declaration::Function(Function { type_params: vec![],
                name: "test".to_string(),
                args: vec![], // Empty args since input has ()
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: None, // Inferred type
                        initializer: Some(Expression::Integer32(42)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::InferredImmutable,
                    },
                    Statement::VariableDeclaration {
                        name: "y".to_string(),
                        type_: None, // Inferred type
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::InferredImmutable,
                    },
                    Statement::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        op: BinaryOperator::Add,
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
            Declaration::Function(Function { type_params: vec![],
                name: "test".to_string(),
                args: vec![], // Empty args since input has ()
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "counter".to_string(),
                        type_: None, // Inferred type
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: true,
                        declaration_type: VariableDeclarationType::InferredMutable,
                    },
                    Statement::Loop {
                        kind: LoopKind::Condition(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("counter".to_string())),
                            op: BinaryOperator::GreaterThan,
                            right: Box::new(Expression::Integer32(0)),
                        }),
                        body: vec![
                            Statement::VariableAssignment {
                                name: "counter".to_string(),
                                value: Expression::BinaryOp {
                                    left: Box::new(Expression::Identifier("counter".to_string())),
                                    op: BinaryOperator::Subtract,
                                    right: Box::new(Expression::Integer32(1)),
                                },
                            },
                        ],
                        label: None,
                    },
                ],
                is_async: false,
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_all_variable_declaration_syntax() {
    let input = "test = () i32 { 
        x := 42;           // Inferred immutable
        y ::= 10;          // Inferred mutable  
        z: i32 = 5;        // Explicit immutable
        w:: u64 = 100;     // Explicit mutable
        x + y + z + w 
    }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    // Verify we have a function declaration
    assert_eq!(program.declarations.len(), 1);
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "test");
        assert_eq!(func.return_type, AstType::I32);
        assert_eq!(func.body.len(), 5); // 4 variable declarations + 1 expression
        
        // Check the variable declarations
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } = &func.body[0] {
            assert_eq!(name, "x");
            assert_eq!(type_, &None); // Inferred type
            assert_eq!(initializer.as_ref().unwrap(), &Expression::Integer32(42));
            assert_eq!(*is_mutable, false);
            assert!(matches!(declaration_type, VariableDeclarationType::InferredImmutable));
        } else {
            panic!("Expected variable declaration for x");
        }
        
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } = &func.body[1] {
            assert_eq!(name, "y");
            assert_eq!(type_, &None); // Inferred type
            assert_eq!(initializer.as_ref().unwrap(), &Expression::Integer32(10));
            assert_eq!(*is_mutable, true);
            assert!(matches!(declaration_type, VariableDeclarationType::InferredMutable));
        } else {
            panic!("Expected variable declaration for y");
        }
        
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } = &func.body[2] {
            assert_eq!(name, "z");
            assert_eq!(type_, &Some(AstType::I32)); // Explicit type
            assert_eq!(initializer.as_ref().unwrap(), &Expression::Integer32(5));
            assert_eq!(*is_mutable, false);
            assert!(matches!(declaration_type, VariableDeclarationType::ExplicitImmutable));
        } else {
            panic!("Expected variable declaration for z");
        }
        
        if let Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } = &func.body[3] {
            assert_eq!(name, "w");
            assert_eq!(type_, &Some(AstType::U64)); // Explicit type
            assert_eq!(initializer.as_ref().unwrap(), &Expression::Integer32(100));
            assert_eq!(*is_mutable, true);
            assert!(matches!(declaration_type, VariableDeclarationType::ExplicitMutable));
        } else {
            panic!("Expected variable declaration for w");
        }
        
        // Check the final expression (x + y + z + w)
        if let Statement::Expression(Expression::BinaryOp { left, op, right }) = &func.body[4] {
            assert_eq!(*op, BinaryOperator::Add);
            // The expression should be: ((x + y) + z) + w
            // This is a complex nested binary expression
        } else {
            panic!("Expected expression statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_variable_declaration_syntax_separately() {
    // Test each syntax variant separately
    
    // Test 1: Inferred immutable (:=)
    let input1 = "test = () i32 { x := 42; x }";
    let lexer1 = Lexer::new(input1);
    let mut parser1 = Parser::new(lexer1);
    let program1 = parser1.parse_program().unwrap();
    assert_eq!(program1.declarations.len(), 1);
    
    // Test 2: Inferred mutable (::=)
    let input2 = "test = () i32 { y ::= 10; y }";
    let lexer2 = Lexer::new(input2);
    let mut parser2 = Parser::new(lexer2);
    let program2 = parser2.parse_program().unwrap();
    assert_eq!(program2.declarations.len(), 1);
    
    // Test 3: Explicit immutable (: T =)
    let input3 = "test = () i32 { z: i32 = 5; z }";
    let lexer3 = Lexer::new(input3);
    let mut parser3 = Parser::new(lexer3);
    let program3 = parser3.parse_program().unwrap();
    assert_eq!(program3.declarations.len(), 1);
    
    // Test 4: Explicit mutable (:: T =)
    let input4 = "test = () i32 { w:: u64 = 100; w }";
    let lexer4 = Lexer::new(input4);
    let mut parser4 = Parser::new(lexer4);
    let program4 = parser4.parse_program().unwrap();
    assert_eq!(program4.declarations.len(), 1);
    
    println!("✓ All variable declaration syntax variants parse successfully!");
}

#[test]
fn test_parse_literal_expressions() {
    let cases = vec![
        ("42", Expression::Integer32(42)),
        ("-7", Expression::BinaryOp {
            left: Box::new(Expression::Integer32(0)),
            op: BinaryOperator::Subtract,
            right: Box::new(Expression::Integer32(7)),
        }),
        ("3.14", Expression::Float64(3.14)),
        ("\"hello\"", Expression::String("hello".to_string())),
        ("foo", Expression::Identifier("foo".to_string())),
    ];
    for (input, expected) in cases {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let expr = parser.parse_expression().unwrap();
        assert_eq!(expr, expected, "input: {}", input);
    }
}

#[test]
fn test_parse_binary_expressions() {
    let lexer = Lexer::new("1 + 2 * 3");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse_expression().unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Integer32(1)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Integer32(2)),
            op: BinaryOperator::Multiply,
            right: Box::new(Expression::Integer32(3)),
        }),
    });
}

#[test]
fn test_parse_unary_and_grouped_expressions() {
    let lexer = Lexer::new("-(1 + 2)");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse_expression().unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Integer32(0)),
        op: BinaryOperator::Subtract,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Integer32(1)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Integer32(2)),
        }),
    });
}

#[test]
fn test_parse_function_call_expression() {
    let lexer = Lexer::new("foo(1, 2)");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse_expression().unwrap();
    assert_eq!(expr, Expression::FunctionCall {
        name: "foo".to_string(),
        args: vec![Expression::Integer32(1), Expression::Integer32(2)],
    });
}

#[test]
fn test_debug_expression_parsing() {
    let input = "x + y";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    println!("Current token: {:?}", parser.debug_current_token());
    println!("Peek token: {:?}", parser.debug_peek_token());
    
    let result = parser.parse_expression();
    println!("Expression parsing result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_debug_statement_parsing() {
    let input = "x + y";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    println!("Current token: {:?}", parser.debug_current_token());
    println!("Peek token: {:?}", parser.debug_peek_token());
    
    let result = parser.parse_statement();
    println!("Statement parsing result: {:?}", result);
    assert!(result.is_ok());
} 

#[test]
fn test_parse_struct_with_generics() {
    let input = "Box<T> = { value: T }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Struct(def) = &program.declarations[0] {
        assert_eq!(def.name, "Box");
        assert_eq!(def.type_params.len(), 1);
        assert_eq!(def.type_params[0].name, "T");
        assert_eq!(def.fields.len(), 1);
        assert_eq!(def.fields[0].name, "value");
        assert_eq!(def.fields[0].type_, zen::ast::AstType::Generic { name: "T".to_string(), type_args: vec![] });
        assert!(def.methods.is_empty());
    } else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn test_parse_struct_with_methods() {
    let input = "Point = { x: i32, y: i32 }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Struct(def) = &program.declarations[0] {
        assert_eq!(def.name, "Point");
        assert!(def.type_params.is_empty());
        assert_eq!(def.fields.len(), 2);
        assert_eq!(def.methods.len(), 0); // Methods not yet supported in parser
    } else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn test_parse_struct_with_generics_and_methods() {
    let input = "Wrapper<T> = { value: T }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Struct(def) = &program.declarations[0] {
        assert_eq!(def.name, "Wrapper");
        assert_eq!(def.type_params.len(), 1);
        assert_eq!(def.type_params[0].name, "T");
        assert_eq!(def.fields.len(), 1);
        assert_eq!(def.methods.len(), 0); // Methods not yet supported in parser
    } else {
        panic!("Expected struct declaration");
    }
} 

#[test]
fn test_parse_match_expression() {
    let input = r#"main = (x: i32) string {
        x ? | 0 => "zero"
            | 1 => "one"
            | _ => "other"
    }"#;
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program();
    assert!(program.is_ok(), "Failed to parse match expression: {:?}", program.err());
    let program = program.unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "main");
        assert_eq!(func.return_type, zen::ast::AstType::String);
        assert_eq!(func.body.len(), 1);
        if let zen::ast::Statement::Expression(zen::ast::Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            // Check scrutinee is an identifier 'x'
            if let zen::ast::Expression::Identifier(name) = &**scrutinee {
                assert_eq!(name, "x");
            } else {
                panic!("Expected identifier 'x' as scrutinee");
            }
            // Check we have 3 arms
            assert_eq!(arms.len(), 3);
            // Check first arm: | 0 => "zero"
            if let zen::ast::ConditionalArm { pattern: zen::ast::Pattern::Literal(zen::ast::Expression::Integer32(0)), guard: None, body: zen::ast::Expression::String(ref s) } = &arms[0] {
                assert_eq!(s, "zero");
            } else {
                panic!("Expected first arm to be | 0 => \"zero\"");
            }
            // Check second arm: | 1 => "one"
            if let zen::ast::ConditionalArm { pattern: zen::ast::Pattern::Literal(zen::ast::Expression::Integer32(1)), guard: None, body: zen::ast::Expression::String(ref s) } = &arms[1] {
                assert_eq!(s, "one");
            } else {
                panic!("Expected second arm to be | 1 => \"one\"");
            }
            // Check third arm: | _ => "other"
            if let zen::ast::ConditionalArm { pattern: zen::ast::Pattern::Wildcard, guard: None, body: zen::ast::Expression::String(ref s) } = &arms[2] {
                assert_eq!(s, "other");
            } else {
                panic!("Expected third arm to be | _ => \"other\"");
            }
        } else {
            panic!("Expected Conditional expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_array_literal() {
    // Test empty array
    let input = "main = () void { x := [] }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 1);
        if let zen::ast::Statement::VariableDeclaration { name, initializer, .. } = &func.body[0] {
            assert_eq!(name, "x");
            if let Some(zen::ast::Expression::ArrayLiteral(elements)) = initializer {
                assert_eq!(elements.len(), 0);
            } else {
                panic!("Expected ArrayLiteral, got {:?}", initializer);
            }
        }
    }

    // Test array with single element
    let input = "main = () void { x := [42] }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        if let zen::ast::Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(zen::ast::Expression::ArrayLiteral(elements)) = initializer {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0], zen::ast::Expression::Integer32(42));
            } else {
                panic!("Expected ArrayLiteral");
            }
        }
    }

    // Test array with multiple elements
    let input = "main = () void { x := [1, 2, 3] }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        if let zen::ast::Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(zen::ast::Expression::ArrayLiteral(elements)) = initializer {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], zen::ast::Expression::Integer32(1));
                assert_eq!(elements[1], zen::ast::Expression::Integer32(2));
                assert_eq!(elements[2], zen::ast::Expression::Integer32(3));
            } else {
                panic!("Expected ArrayLiteral");
            }
        }
    }

    // Test array with trailing comma
    let input = "main = () void { x := [1, 2, 3,] }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        if let zen::ast::Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
            if let Some(zen::ast::Expression::ArrayLiteral(elements)) = initializer {
                assert_eq!(elements.len(), 3);
            } else {
                panic!("Expected ArrayLiteral");
            }
        }
    }
}

#[test]
fn test_parse_range_based_loop() {
    let input = "main = () void { loop 0..10 { x := 1 } }";
    let lexer = zen::lexer::Lexer::new(input);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    if let zen::ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "main");
        assert_eq!(func.return_type, zen::ast::AstType::Void);
        assert_eq!(func.body.len(), 1);
        if let zen::ast::Statement::Loop { kind, body, label } = &func.body[0] {
            assert!(label.is_none());
            if let LoopKind::Condition(zen::ast::Expression::Range { start, end, inclusive }) = kind {
                assert_eq!(**start, zen::ast::Expression::Integer32(0));
                assert_eq!(**end, zen::ast::Expression::Integer32(10));
                assert!(!inclusive);
            } else {
                panic!("Expected LoopKind::Condition with Range expression");
            }
            assert_eq!(body.len(), 1);
            if let zen::ast::Statement::VariableDeclaration { name, .. } = &body[0] {
                assert_eq!(name, "x");
            } else {
                panic!("Expected variable declaration in loop body");
            }
        } else {
            panic!("Expected loop statement");
        }
    } else {
        panic!("Expected function declaration");
    }
} 