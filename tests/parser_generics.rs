use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{AstType, TypeParameter, Program, Declaration, Statement, Function};

#[test]
fn test_parse_generic_type_instantiation() {
    let input = "main :: () -> void { x: List<i32> = make_list(); }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok());
    
    // Check that List<i32> is parsed correctly
    let program = result.unwrap();
    if let Some(Declaration::Function(func)) = program.declarations.first() {
        if let Some(statement) = func.body.first() {
            if let Statement::VariableDeclaration { type_, .. } = statement {
                if let Some(AstType::Generic { name, type_args }) = type_ {
                    assert_eq!(name, "List");
                    assert_eq!(type_args.len(), 1);
                    assert_eq!(type_args[0], AstType::I32);
                } else {
                    panic!("Expected Generic type");
                }
            } else {
                panic!("Expected VariableDeclaration");
            }
        } else {
            panic!("Expected function body");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_nested_generic_types() {
    let input = "main :: () -> void { x: Option<List<i32>> = None; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok());
    
    let program = result.unwrap();
    if let Some(Declaration::Function(func)) = program.declarations.first() {
        if let Some(statement) = func.body.first() {
            if let Statement::VariableDeclaration { type_, .. } = statement {
                if let Some(AstType::Generic { name, type_args }) = type_ {
                    assert_eq!(name, "Option");
                    assert_eq!(type_args.len(), 1);
                    
                    if let AstType::Generic { name: inner_name, type_args: inner_args } = &type_args[0] {
                        assert_eq!(inner_name, "List");
                        assert_eq!(inner_args.len(), 1);
                        assert_eq!(inner_args[0], AstType::I32);
                    } else {
                        panic!("Expected nested Generic type");
                    }
                } else {
                    panic!("Expected Generic type");
                }
            } else {
                panic!("Expected VariableDeclaration");
            }
        } else {
            panic!("Expected function body");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_multiple_type_arguments() {
    let input = "main :: () -> void { x: Map<String, i32> = make_map(); }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok());
    
    let program = result.unwrap();
    if let Some(Declaration::Function(func)) = program.declarations.first() {
        if let Some(statement) = func.body.first() {
            if let Statement::VariableDeclaration { type_, .. } = statement {
                if let Some(AstType::Generic { name, type_args }) = type_ {
                    assert_eq!(name, "Map");
                    assert_eq!(type_args.len(), 2);
                    assert_eq!(type_args[0], AstType::String);
                    assert_eq!(type_args[1], AstType::I32);
                } else {
                    panic!("Expected Generic type");
                }
            } else {
                panic!("Expected VariableDeclaration");
            }
        } else {
            panic!("Expected function body");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_generic_struct_declaration() {
    let input = "List<T> = { items: [T], size: u64 }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok());
    
    let program = result.unwrap();
    if let Some(Declaration::Struct(struct_def)) = program.declarations.first() {
        assert_eq!(struct_def.name, "List");
        assert_eq!(struct_def.type_params.len(), 1);
        assert_eq!(struct_def.type_params[0].name, "T");
        
        // Check that field uses the type parameter
        assert_eq!(struct_def.fields.len(), 2);
        if let AstType::Array(elem_type) = &struct_def.fields[0].type_ {
            if let AstType::Generic { name, type_args } = elem_type.as_ref() {
                assert_eq!(name, "T");
                assert!(type_args.is_empty());
            } else {
                panic!("Expected Generic type parameter in field");
            }
        } else {
            panic!("Expected Array type for field");
        }
    } else {
        panic!("Expected Struct declaration");
    }
}

#[test]
fn test_parse_generic_function_declaration() {
    let input = "map<T, U> :: (list: List<T>, f: Function<T, U>) -> List<U> { list }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());
    
    let program = result.unwrap();
    if let Some(Declaration::Function(func)) = program.declarations.first() {
        assert_eq!(func.name, "map");
        assert_eq!(func.type_params.len(), 2);
        assert_eq!(func.type_params[0].name, "T");
        assert_eq!(func.type_params[1].name, "U");
        
        // Check parameter types
        assert_eq!(func.args.len(), 2);
        
        // First param: List<T>
        if let AstType::Generic { name, type_args } = &func.args[0].1 {
            assert_eq!(name, "List");
            assert_eq!(type_args.len(), 1);
            if let AstType::Generic { name: t_name, type_args: t_args } = &type_args[0] {
                assert_eq!(t_name, "T");
                assert!(t_args.is_empty());
            }
        } else {
            panic!("Expected Generic type for first parameter");
        }
        
        // Return type: List<U>
        if let AstType::Generic { name, type_args } = &func.return_type {
            assert_eq!(name, "List");
            assert_eq!(type_args.len(), 1);
            if let AstType::Generic { name: u_name, type_args: u_args } = &type_args[0] {
                assert_eq!(u_name, "U");
                assert!(u_args.is_empty());
            }
        } else {
            panic!("Expected Generic return type");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_generic_with_array() {
    let input = "main :: () -> void { x: [Option<i32>] = []; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok());
    
    let program = result.unwrap();
    if let Some(Declaration::Function(func)) = program.declarations.first() {
        if let Some(statement) = func.body.first() {
            if let Statement::VariableDeclaration { type_, .. } = statement {
                if let Some(AstType::Array(elem_type)) = type_ {
                    if let AstType::Generic { name, type_args } = elem_type.as_ref() {
                        assert_eq!(name, "Option");
                        assert_eq!(type_args.len(), 1);
                        assert_eq!(type_args[0], AstType::I32);
                    } else {
                        panic!("Expected Generic element type");
                    }
                } else {
                    panic!("Expected Array type");
                }
            } else {
                panic!("Expected VariableDeclaration");
            }
        } else {
            panic!("Expected function body");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_parse_generic_with_pointer() {
    let input = "main :: () -> void { x: *List<String> = 0; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    if let Err(e) = &result {
        eprintln!("Parse error in pointer test: {:?}", e);
    }
    assert!(result.is_ok());
    
    let program = result.unwrap();
    if let Some(Declaration::Function(func)) = program.declarations.first() {
        if let Some(statement) = func.body.first() {
            if let Statement::VariableDeclaration { type_, .. } = statement {
                if let Some(AstType::Pointer(pointee)) = type_ {
                    if let AstType::Generic { name, type_args } = pointee.as_ref() {
                        assert_eq!(name, "List");
                        assert_eq!(type_args.len(), 1);
                        assert_eq!(type_args[0], AstType::String);
                    } else {
                        panic!("Expected Generic pointee type");
                    }
                } else {
                    panic!("Expected Pointer type");
                }
            } else {
                panic!("Expected VariableDeclaration");
            }
        } else {
            panic!("Expected function body");
        }
    } else {
        panic!("Expected Function declaration");
    }
}