use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{Declaration, Function, Statement, Expression, Pattern, AstType, ConditionalArm};

#[test]
fn test_parse_simple_pattern_match() {
    let input = r#"
    test_match = (x: i32) string {
        x ? | 0 => "zero"
            | 1 => "one"
            | _ => "other"
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "test_match");
        assert_eq!(func.body.len(), 1);
        
        if let Statement::Expression(Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            assert!(matches!(**scrutinee, Expression::Identifier(ref name) if name == "x"));
            assert_eq!(arms.len(), 3);
            
            // Check first arm: 0 => "zero"
            assert!(matches!(arms[0].pattern, Pattern::Literal(Expression::Integer32(0))));
            assert!(matches!(arms[0].body, Expression::String(ref s) if s == "zero"));
            
            // Check second arm: 1 => "one"
            assert!(matches!(arms[1].pattern, Pattern::Literal(Expression::Integer32(1))));
            assert!(matches!(arms[1].body, Expression::String(ref s) if s == "one"));
            
            // Check third arm: _ => "other"
            assert!(matches!(arms[2].pattern, Pattern::Wildcard));
            assert!(matches!(arms[2].body, Expression::String(ref s) if s == "other"));
        } else {
            panic!("Expected pattern match expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_enum_pattern_match() {
    let input = r#"
    Result<T, E> = | Ok(value: T) | Err(error: E)
    
    handle_result = (r: Result<i32, string>) i32 {
        r ? | .Ok -> val => val
            | .Err -> _ => -1
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 2);
    
    // Check function with enum pattern matching
    if let Declaration::Function(func) = &program.declarations[1] {
        assert_eq!(func.name, "handle_result");
        assert_eq!(func.body.len(), 1);
        
        if let Statement::Expression(Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            assert!(matches!(**scrutinee, Expression::Identifier(ref name) if name == "r"));
            assert_eq!(arms.len(), 2);
            
            // Check Ok pattern with binding
            if let Pattern::EnumVariant { variant, payload, .. } = &arms[0].pattern {
                assert_eq!(variant, "Ok");
                assert!(payload.is_some());
                if let Some(ref p) = payload {
                    assert!(matches!(**p, Pattern::Identifier(ref name) if name == "val"));
                }
            } else {
                panic!("Expected enum variant pattern for Ok");
            }
            
            // Check Err pattern with wildcard
            if let Pattern::EnumVariant { variant, payload, .. } = &arms[1].pattern {
                assert_eq!(variant, "Err");
                assert!(payload.is_some());
                if let Some(ref p) = payload {
                    assert!(matches!(**p, Pattern::Wildcard));
                }
            } else {
                panic!("Expected enum variant pattern for Err");
            }
        } else {
            panic!("Expected pattern match expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_range_patterns() {
    let input = r#"
    grade = (score: i32) string {
        score ? | 90..=100 => "A"
                | 80..=89  => "B"
                | 70..=79  => "C"
                | 60..=69  => "D"
                | _        => "F"
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "grade");
        assert_eq!(func.body.len(), 1);
        
        if let Statement::Expression(Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            assert!(matches!(**scrutinee, Expression::Identifier(ref name) if name == "score"));
            assert_eq!(arms.len(), 5);
            
            // Check first range pattern: 90..=100
            if let Pattern::Literal(Expression::Range { start, end, inclusive }) = &arms[0].pattern {
                assert!(matches!(**start, Expression::Integer32(90)));
                assert!(matches!(**end, Expression::Integer32(100)));
                assert!(*inclusive);
                assert!(matches!(arms[0].body, Expression::String(ref s) if s == "A"));
            } else {
                panic!("Expected range pattern for first arm");
            }
            
            // Check last pattern is wildcard
            assert!(matches!(arms[4].pattern, Pattern::Wildcard));
            assert!(matches!(arms[4].body, Expression::String(ref s) if s == "F"));
        } else {
            panic!("Expected pattern match expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_struct_patterns() {
    let input = r#"
    Person = {
        name: string,
        age: i32,
    }
    
    describe = (p: Person) string {
        p ? | Person { name: n, age: 18 } => "Adult named $(n)"
            | Person { name: n, age: _ }  => "Person named $(n)"
            | _ => "Unknown"
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 2);
    
    // Check function with struct pattern matching
    if let Declaration::Function(func) = &program.declarations[1] {
        assert_eq!(func.name, "describe");
        assert_eq!(func.body.len(), 1);
        
        if let Statement::Expression(Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            assert!(matches!(**scrutinee, Expression::Identifier(ref name) if name == "p"));
            assert_eq!(arms.len(), 3);
            
            // Check first struct pattern with field patterns
            if let Pattern::Struct { name, fields } = &arms[0].pattern {
                assert_eq!(name, "Person");
                assert_eq!(fields.len(), 2);
                
                // Check name field binding
                let name_field = fields.iter().find(|(fname, _)| fname == "name").unwrap();
                assert!(matches!(name_field.1, Pattern::Identifier(ref n) if n == "n"));
                
                // Check age field literal
                let age_field = fields.iter().find(|(fname, _)| fname == "age").unwrap();
                assert!(matches!(age_field.1, Pattern::Literal(Expression::Integer32(18))));
            } else {
                panic!("Expected struct pattern for first arm");
            }
            
            // Check second struct pattern with wildcard age
            if let Pattern::Struct { name, fields } = &arms[1].pattern {
                assert_eq!(name, "Person");
                assert_eq!(fields.len(), 2);
                
                // Check age field is wildcard
                let age_field = fields.iter().find(|(fname, _)| fname == "age").unwrap();
                assert!(matches!(age_field.1, Pattern::Wildcard));
            } else {
                panic!("Expected struct pattern for second arm");
            }
        } else {
            panic!("Expected pattern match expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_or_patterns() {
    let input = r#"
    is_weekend = (day: string) bool {
        day ? | "Saturday" | "Sunday" => true
              | _ => false
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "is_weekend");
        assert_eq!(func.body.len(), 1);
        
        if let Statement::Expression(Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            assert!(matches!(**scrutinee, Expression::Identifier(ref name) if name == "day"));
            assert_eq!(arms.len(), 2);
            
            // Check or pattern: "Saturday" | "Sunday"
            if let Pattern::Or(patterns) = &arms[0].pattern {
                assert_eq!(patterns.len(), 2);
                assert!(matches!(patterns[0], Pattern::Literal(Expression::String(ref s)) if s == "Saturday"));
                assert!(matches!(patterns[1], Pattern::Literal(Expression::String(ref s)) if s == "Sunday"));
                assert!(matches!(arms[0].body, Expression::Boolean(true)));
            } else {
                panic!("Expected or pattern for first arm");
            }
            
            // Check wildcard pattern
            assert!(matches!(arms[1].pattern, Pattern::Wildcard));
            assert!(matches!(arms[1].body, Expression::Boolean(false)));
        } else {
            panic!("Expected pattern match expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parse_binding_patterns() {
    let input = r#"
    Option<T> = | Some(value: T) | None
    
    unwrap_or = (opt: Option<i32>, default: i32) i32 {
        opt ? | .Some -> x => x
              | .None => default
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 2);
    
    // Check function with binding patterns
    if let Declaration::Function(func) = &program.declarations[1] {
        assert_eq!(func.name, "unwrap_or");
        assert_eq!(func.body.len(), 1);
        
        if let Statement::Expression(Expression::Conditional { scrutinee, arms }) = &func.body[0] {
            assert!(matches!(**scrutinee, Expression::Identifier(ref name) if name == "opt"));
            assert_eq!(arms.len(), 2);
            
            // Check Some pattern with binding
            if let Pattern::EnumVariant { variant, payload, .. } = &arms[0].pattern {
                assert_eq!(variant, "Some");
                assert!(payload.is_some());
                if let Some(ref p) = payload {
                    assert!(matches!(**p, Pattern::Identifier(ref name) if name == "x"));
                }
                assert!(matches!(arms[0].body, Expression::Identifier(ref name) if name == "x"));
            } else {
                panic!("Expected enum variant pattern for Some");
            }
            
            // Check None pattern
            if let Pattern::EnumVariant { variant, payload, .. } = &arms[1].pattern {
                assert_eq!(variant, "None");
                assert!(payload.is_none());
                assert!(matches!(arms[1].body, Expression::Identifier(ref name) if name == "default"));
            } else {
                panic!("Expected enum variant pattern for None");
            }
        } else {
            panic!("Expected pattern match expression");
        }
    } else {
        panic!("Expected function declaration");
    }
}