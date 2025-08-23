use zen::ast::*;
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_behavior_definition() {
    let input = r#"
        Writer = behavior {
            write = (self, data: []u8) Result<u64, string>
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Behavior(behavior) = &program.declarations[0] {
        assert_eq!(behavior.name, "Writer");
        assert_eq!(behavior.methods.len(), 1);
        
        let method = &behavior.methods[0];
        assert_eq!(method.name, "write");
        assert_eq!(method.params.len(), 2);
        assert_eq!(method.params[0].name, "self");
        assert_eq!(method.params[1].name, "data");
    } else {
        panic!("Expected behavior declaration");
    }
}

#[test] 
fn test_parse_behavior_with_multiple_methods() {
    let input = r#"
        Stream = behavior {
            read = (self, buffer: []u8) Result<u64, string>,
            write = (self, data: []u8) Result<u64, string>,
            close = (self) void
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Behavior(behavior) = &program.declarations[0] {
        assert_eq!(behavior.name, "Stream");
        assert_eq!(behavior.methods.len(), 3);
        
        assert_eq!(behavior.methods[0].name, "read");
        assert_eq!(behavior.methods[1].name, "write");
        assert_eq!(behavior.methods[2].name, "close");
    } else {
        panic!("Expected behavior declaration");
    }
}

#[test]
fn test_parse_impl_block() {
    let input = r#"
        File.impl = {
            open = (path: string) File {
                // Implementation
                File { fd: 0 }
            }
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Impl(impl_block) = &program.declarations[0] {
        assert_eq!(impl_block.type_name, "File");
        assert_eq!(impl_block.behavior_name, None);
        assert_eq!(impl_block.methods.len(), 1);
        assert_eq!(impl_block.methods[0].name, "open");
    } else {
        panic!("Expected impl block");
    }
}

#[test]
fn test_parse_impl_block_for_behavior() {
    let input = r#"
        File.impl = {
            Writer: {
                write = (self: Ptr<File>, data: []u8) Result<u64, string> {
                    // Implementation
                    Ok(0)
                }
            }
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Impl(impl_block) = &program.declarations[0] {
        assert_eq!(impl_block.type_name, "File");
        assert_eq!(impl_block.behavior_name, Some("Writer".to_string()));
        assert_eq!(impl_block.methods.len(), 1);
        assert_eq!(impl_block.methods[0].name, "write");
    } else {
        panic!("Expected impl block for behavior");
    }
}

#[test]
fn test_parse_generic_behavior() {
    let input = r#"
        Container<T> = behavior {
            add = (self, item: T) void,
            get = (self, index: u64) T
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Behavior(behavior) = &program.declarations[0] {
        assert_eq!(behavior.name, "Container");
        assert_eq!(behavior.type_params.len(), 1);
        assert_eq!(behavior.type_params[0].name, "T");
        assert_eq!(behavior.methods.len(), 2);
    } else {
        panic!("Expected generic behavior declaration");
    }
}