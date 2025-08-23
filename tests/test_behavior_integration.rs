use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::typechecker::TypeChecker;

#[test]
fn test_behavior_definition_and_implementation() {
    let input = r#"
        Display = behavior {
            display = (self) string
        }
        
        Point = struct {
            x: f64,
            y: f64
        }
        
        Point.impl = {
            Display: {
                display = (self: Ptr<Point>) string {
                    return "Point"
                }
            }
        }
    "#;

    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Type check should succeed
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check_program(&program).is_ok());
}

#[test]
fn test_generic_behavior() {
    let input = r#"
        Iterator<T> = behavior {
            next = (self) Option<T>
        }
        
        List<T> = struct {
            items: [T]
        }
        
        List<T>.impl = {
            Iterator<T>: {
                next = (self: Ptr<List<T>>) Option<T> {
                    return Option::None
                }
            }
        }
    "#;

    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Type check should succeed
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check_program(&program).is_ok());
}

#[test]
fn test_inherent_impl_without_behavior() {
    let input = r#"
        Vector = struct {
            x: f64,
            y: f64,
            z: f64
        }
        
        Vector.impl = {
            magnitude = (self: Ptr<Vector>) f64 {
                return 0.0
            },
            
            normalize = (self: Ptr<Vector>) Vector {
                mag := self.magnitude()
                return Vector { 
                    x: self.x / mag, 
                    y: self.y / mag, 
                    z: self.z / mag 
                }
            }
        }
    "#;

    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Type check should succeed
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check_program(&program).is_ok());
}

#[test]
fn test_missing_behavior_method_error() {
    let input = r#"
        Drawable = behavior {
            draw = (self) void,
            get_bounds = (self) Rect
        }
        
        Rect = struct {
            x: f64, 
            y: f64,
            width: f64, 
            height: f64
        }
        
        Circle = struct {
            center_x: f64,
            center_y: f64,
            radius: f64
        }
        
        Circle.impl = {
            Drawable: {
                draw = (self: Ptr<Circle>) void {
                    // Drawing implementation
                }
                // Missing get_bounds method
            }
        }
    "#;

    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Type check should fail due to missing method
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check_program(&program);
    assert!(result.is_err());
    
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("Missing") || error_msg.contains("does not implement"));
    }
}

#[test]
fn test_multiple_methods_behavior() {
    let input = r#"
        Container<T> = behavior {
            size = (self) i32,
            is_empty = (self) bool,
            push = (self, item: T) void
        }
        
        Stack<T> = struct {
            items: [T],
            top: i32
        }
        
        Stack<T>.impl = {
            Container<T>: {
                size = (self: Ptr<Stack<T>>) i32 {
                    return self.top
                },
                
                is_empty = (self: Ptr<Stack<T>>) bool {
                    return self.top == 0
                },
                
                push = (self: Ptr<Stack<T>>, item: T) void {
                    // Implementation
                }
            }
        }
    "#;

    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Type check should succeed
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check_program(&program).is_ok());
}