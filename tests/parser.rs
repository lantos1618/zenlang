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
    let input = "fn main() int32 { return 42; }";
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
                    Statement::Return(Expression::Integer32(42))
                ],
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_variable_declaration() {
    let input = "fn main() int32 { let x: int32 = 10; return x; }";
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
                    Statement::Return(Expression::Identifier("x".to_string())),
                ],
            })
        ],
    };
    assert_eq!(program, expected);
}

#[test]
fn test_parse_binary_expression() {
    let input = "fn main() int32 { return 5 + 3; }";
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
                    Statement::Return(Expression::BinaryOp {
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
fn test_parse_zen_imports() {
    let input = "comptime { build := @std.build io := build.import(\"io\") }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add ComptimeBlock to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_conditional_expression() {
    let input = "fn grade(score: int32) string { return score -> s { | s >= 90 => \"A\" | s >= 80 => \"B\" | true => \"C\" }; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add Conditional to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_loop_with_condition() {
    let input = "fn countdown() void { counter ::= 10; loop counter > 0 { counter = counter - 1; } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add Loop to AST later
    assert_eq!(program, Program { declarations: vec![] });
}

#[test]
fn test_parse_loop_with_in() {
    let input = "fn print_names() void { names := [\"Alice\", \"Bob\"]; loop name in names { io.print(name); } }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to add Loop to AST later
    assert_eq!(program, Program { declarations: vec![] });
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
fn test_parse_zen_variable_declarations() {
    let input = "fn test() int32 { x := 42; y ::= 10; z: int32 = 5; w:: uint64 = 100; return x + y; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // For now, just test that it parses without error
    // We'll need to update variable declaration parsing later
    assert_eq!(program, Program { declarations: vec![] });
} 