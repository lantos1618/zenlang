use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{Statement, Expression, VariableDeclarationType};

#[test]
fn test_parse_std_namespace() {
    let input = "@std";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Identifier(name) if name == "@std"));
}

#[test]
fn test_parse_std_core() {
    let input = "@std.core";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::MemberAccess { object, member } = expr {
        assert!(matches!(*object, Expression::Identifier(name) if name == "@std"));
        assert_eq!(member, "core");
    } else {
        panic!("Expected MemberAccess expression");
    }
}

#[test]
fn test_parse_std_build() {
    let input = "@std.build";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::MemberAccess { object, member } = expr {
        assert!(matches!(*object, Expression::Identifier(name) if name == "@std"));
        assert_eq!(member, "build");
    } else {
        panic!("Expected MemberAccess expression");
    }
}

#[test]
fn test_comptime_std_usage() {
    let input = "comptime { core := @std.core }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Some(zen::ast::Declaration::ComptimeBlock(statements)) = program.declarations.first() {
        assert_eq!(statements.len(), 1);
        
        if let Statement::VariableDeclaration { name, initializer, declaration_type, .. } = &statements[0] {
            assert_eq!(name, "core");
            assert!(matches!(declaration_type, VariableDeclarationType::InferredImmutable));
            
            if let Some(Expression::MemberAccess { object, member }) = initializer {
                assert!(matches!(**object, Expression::Identifier(ref name) if name == "@std"));
                assert_eq!(member, "core");
            } else {
                panic!("Expected MemberAccess in core initialization");
            }
        } else {
            panic!("Expected VariableDeclaration");
        }
    } else {
        panic!("Expected ComptimeBlock");
    }
}

#[test]
fn test_build_import() {
    let input = r#"comptime { 
        build := @std.build
        io := build.import("io")
    }"#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Some(zen::ast::Declaration::ComptimeBlock(statements)) = program.declarations.first() {
        assert_eq!(statements.len(), 2);
        
        // Check second statement: io := build.import("io")
        if let Statement::VariableDeclaration { name, initializer, .. } = &statements[1] {
            assert_eq!(name, "io");
            
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
        panic!("Expected ComptimeBlock");
    }
}

#[test]
fn test_typecheck_std_module() {
    let input = "comptime { core := @std.core }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    // Type check the program
    let mut type_checker = zen::typechecker::TypeChecker::new();
    let result = type_checker.check_program(&program);
    assert!(result.is_ok());
}