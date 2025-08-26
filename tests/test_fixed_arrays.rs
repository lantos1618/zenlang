use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{AstType, Declaration, Statement};

#[test]
fn test_parse_fixed_array_type() {
    let input = r#"
        test_arrays = () void {
            arr1: [i32; 10] = 0;
            arr2: [f64; 5] = 0.0;
            arr3: [[u8; 3]; 4] = 0;
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    
    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.declarations.len(), 1);
    
    // Check that the function was parsed
    match &ast.declarations[0] {
        Declaration::Function(func) => {
            assert_eq!(func.name, "test_arrays");
            assert_eq!(func.body.len(), 3);
            
            // Check each variable declaration
            for stmt in &func.body {
                match stmt {
                    Statement::VariableDeclaration { name, type_, .. } => {
                        match name.as_str() {
                            "arr1" => {
                                match type_ {
                                    Some(AstType::FixedArray { element_type, size }) => {
                                        assert_eq!(**element_type, AstType::I32);
                                        assert_eq!(*size, 10);
                                    }
                                    _ => panic!("Expected FixedArray type for arr1"),
                                }
                            }
                            "arr2" => {
                                match type_ {
                                    Some(AstType::FixedArray { element_type, size }) => {
                                        assert_eq!(**element_type, AstType::F64);
                                        assert_eq!(*size, 5);
                                    }
                                    _ => panic!("Expected FixedArray type for arr2"),
                                }
                            }
                            "arr3" => {
                                match type_ {
                                    Some(AstType::FixedArray { element_type, size }) => {
                                        assert_eq!(*size, 4);
                                        match &**element_type {
                                            AstType::FixedArray { element_type: inner, size: inner_size } => {
                                                assert_eq!(**inner, AstType::U8);
                                                assert_eq!(*inner_size, 3);
                                            }
                                            _ => panic!("Expected nested FixedArray type for arr3"),
                                        }
                                    }
                                    _ => panic!("Expected FixedArray type for arr3"),
                                }
                            }
                            _ => panic!("Unexpected variable name"),
                        }
                    }
                    _ => panic!("Expected variable declaration"),
                }
            }
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_fixed_array_vs_dynamic_array() {
    let input = r#"
        test_arrays = () void {
            fixed: [i32; 10] = 0;
            dynamic: [i32] = 0;
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    
    assert!(result.is_ok());
    let ast = result.unwrap();
    
    match &ast.declarations[0] {
        Declaration::Function(func) => {
            assert_eq!(func.body.len(), 2);
            
            // Check fixed array
            match &func.body[0] {
                Statement::VariableDeclaration { name, type_, .. } => {
                    assert_eq!(name, "fixed");
                    match type_ {
                        Some(AstType::FixedArray { element_type, size }) => {
                            assert_eq!(**element_type, AstType::I32);
                            assert_eq!(*size, 10);
                        }
                        _ => panic!("Expected FixedArray type"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
            
            // Check dynamic array
            match &func.body[1] {
                Statement::VariableDeclaration { name, type_, .. } => {
                    assert_eq!(name, "dynamic");
                    match type_ {
                        Some(AstType::Array(element_type)) => {
                            assert_eq!(**element_type, AstType::I32);
                        }
                        _ => panic!("Expected Array type"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }
        _ => panic!("Expected function declaration"),
    }
}