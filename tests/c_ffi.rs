use zen::ast::*;
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_extern_function() {
    let input = "extern printf = (ptr, ...) i32";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::ExternalFunction(ext_func) = &program.declarations[0] {
        assert_eq!(ext_func.name, "printf");
        assert_eq!(ext_func.args.len(), 1);
        assert!(matches!(ext_func.args[0], AstType::Pointer(_)));
        assert!(ext_func.is_varargs);
        assert!(matches!(ext_func.return_type, AstType::I32));
    } else {
        panic!("Expected ExternalFunction declaration");
    }
}

#[test]
fn test_parse_simple_extern() {
    let input = "extern puts = (ptr) i32";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::ExternalFunction(ext_func) = &program.declarations[0] {
        assert_eq!(ext_func.name, "puts");
        assert_eq!(ext_func.args.len(), 1);
        assert!(matches!(ext_func.args[0], AstType::Pointer(_)));
        assert!(!ext_func.is_varargs);
        assert!(matches!(ext_func.return_type, AstType::I32));
    } else {
        panic!("Expected ExternalFunction declaration");
    }
}

#[test]
fn test_parse_multi_arg_extern() {
    let input = "extern memcpy = (ptr, ptr, u64) ptr";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::ExternalFunction(ext_func) = &program.declarations[0] {
        assert_eq!(ext_func.name, "memcpy");
        assert_eq!(ext_func.args.len(), 3);
        assert!(matches!(ext_func.args[0], AstType::Pointer(_)));
        assert!(matches!(ext_func.args[1], AstType::Pointer(_)));
        assert!(matches!(ext_func.args[2], AstType::U64));
        assert!(!ext_func.is_varargs);
        assert!(matches!(ext_func.return_type, AstType::Pointer(_)));
    } else {
        panic!("Expected ExternalFunction declaration");
    }
}

#[test]
fn test_extern_with_function_call() {
    let input = r#"
extern puts = (ptr) i32

main :: () -> i32 {
    puts("Hello from C!")
    return 0
}
"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 2);
    
    // Check extern declaration
    if let Declaration::ExternalFunction(ext_func) = &program.declarations[0] {
        assert_eq!(ext_func.name, "puts");
    } else {
        panic!("Expected ExternalFunction declaration");
    }
    
    // Check function that calls the extern
    if let Declaration::Function(func) = &program.declarations[1] {
        assert_eq!(func.name, "main");
        assert_eq!(func.body.len(), 2);
        
        if let Statement::Expression(Expression::FunctionCall { name, args }) = &func.body[0] {
            assert_eq!(name, "puts");
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Expression::String(_)));
        } else {
            panic!("Expected function call statement");
        }
    } else {
        panic!("Expected Function declaration");
    }
}