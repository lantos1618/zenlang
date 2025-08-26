use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{Statement, Expression, AstType};
use zen::stdlib::io::IOModule;

#[test]
fn test_io_module_creation() {
    let io = IOModule::new();
    let functions = io.list_functions();
    
    // Check that all essential functions exist
    assert!(functions.contains(&"print".to_string()));
    assert!(functions.contains(&"println".to_string()));
    assert!(functions.contains(&"read_line".to_string()));
    assert!(functions.contains(&"read_file".to_string()));
    assert!(functions.contains(&"write_file".to_string()));
}

#[test]
fn test_print_function_signature() {
    let io = IOModule::new();
    let print_fn = io.get_function("print").unwrap();
    
    assert_eq!(print_fn.name, "print");
    assert_eq!(print_fn.params.len(), 1);
    assert_eq!(print_fn.params[0].0, "message");
    assert_eq!(print_fn.params[0].1, AstType::String);
    assert_eq!(print_fn.return_type, AstType::Void);
    assert!(print_fn.is_builtin);
}

#[test]
fn test_read_file_function_signature() {
    let io = IOModule::new();
    let read_file = io.get_function("read_file").unwrap();
    
    assert_eq!(read_file.name, "read_file");
    assert_eq!(read_file.params.len(), 1);
    assert_eq!(read_file.params[0].0, "path");
    assert_eq!(read_file.params[0].1, AstType::String);
    
    // Check that it returns Result<String, String>
    match &read_file.return_type {
        AstType::Result { ok_type, err_type } => {
            assert_eq!(**ok_type, AstType::String);
            assert_eq!(**err_type, AstType::String);
        }
        _ => panic!("read_file should return Result type"),
    }
}

#[test]
fn test_file_exists_function_signature() {
    let io = IOModule::new();
    let file_exists = io.get_function("file_exists").unwrap();
    
    assert_eq!(file_exists.name, "file_exists");
    assert_eq!(file_exists.params.len(), 1);
    assert_eq!(file_exists.params[0].0, "path");
    assert_eq!(file_exists.params[0].1, AstType::String);
    assert_eq!(file_exists.return_type, AstType::Bool);
}

#[test]
fn test_parse_io_module_access() {
    // Test that we can parse @std.io access
    let input = "@std.io";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::MemberAccess { object, member } = expr {
        assert!(matches!(*object, Expression::Identifier(name) if name == "@std"));
        assert_eq!(member, "io");
    } else {
        panic!("Expected MemberAccess for @std.io");
    }
}

#[test]
fn test_io_println_usage() {
    // Test that IO module has print functions
    let io = IOModule::new();
    
    assert!(io.get_function("print").is_some());
    assert!(io.get_function("println").is_some());
    assert!(io.get_function("eprint").is_some());
    assert!(io.get_function("eprintln").is_some());
}

#[test]
fn test_io_file_operations() {
    // Test that IO module has file operations
    let io = IOModule::new();
    
    assert!(io.get_function("read_file").is_some());
    assert!(io.get_function("write_file").is_some());
    assert!(io.get_function("append_file").is_some());
    assert!(io.get_function("file_exists").is_some());
    assert!(io.get_function("create_dir").is_some());
    assert!(io.get_function("remove_file").is_some());
}