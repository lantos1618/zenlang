use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::ast::{Expression, Pattern};
use zen::stdlib::result::{
    create_result_type, create_option_type,
    ok_value, err_value, some_value, none_value,
    ok_pattern, err_pattern, some_pattern, none_pattern
};

#[test]
fn test_result_type_creation() {
    let result_type = create_result_type();
    
    match result_type {
        zen::ast::AstType::Enum { name, variants } => {
            assert_eq!(name, "Result");
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Ok");
            assert!(variants[0].payload.is_some());
            assert_eq!(variants[1].name, "Err");
            assert!(variants[1].payload.is_some());
        }
        _ => panic!("Expected Enum type for Result"),
    }
}

#[test]
fn test_option_type_creation() {
    let option_type = create_option_type();
    
    match option_type {
        zen::ast::AstType::Enum { name, variants } => {
            assert_eq!(name, "Option");
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Some");
            assert!(variants[0].payload.is_some());
            assert_eq!(variants[1].name, "None");
            assert!(variants[1].payload.is_none());
        }
        _ => panic!("Expected Enum type for Option"),
    }
}

#[test]
fn test_ok_value_creation() {
    let value = Expression::Integer32(42);
    let ok = ok_value(value);
    
    match ok {
        Expression::EnumVariant { enum_name, variant, payload } => {
            assert_eq!(enum_name, "Result");
            assert_eq!(variant, "Ok");
            assert!(payload.is_some());
            if let Some(boxed_expr) = payload {
                assert!(matches!(*boxed_expr, Expression::Integer32(42)));
            }
        }
        _ => panic!("Expected EnumVariant for Ok"),
    }
}

#[test]
fn test_err_value_creation() {
    let error = Expression::String("Error message".to_string());
    let err = err_value(error);
    
    match err {
        Expression::EnumVariant { enum_name, variant, payload } => {
            assert_eq!(enum_name, "Result");
            assert_eq!(variant, "Err");
            assert!(payload.is_some());
        }
        _ => panic!("Expected EnumVariant for Err"),
    }
}

#[test]
fn test_some_value_creation() {
    let value = Expression::Integer32(42);
    let some = some_value(value);
    
    match some {
        Expression::EnumVariant { enum_name, variant, payload } => {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant, "Some");
            assert!(payload.is_some());
        }
        _ => panic!("Expected EnumVariant for Some"),
    }
}

#[test]
fn test_none_value_creation() {
    let none = none_value();
    
    match none {
        Expression::EnumVariant { enum_name, variant, payload } => {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant, "None");
            assert!(payload.is_none());
        }
        _ => panic!("Expected EnumVariant for None"),
    }
}

#[test]
fn test_ok_pattern_with_binding() {
    let pattern = ok_pattern(Some("value".to_string()));
    
    match pattern {
        Pattern::EnumVariant { enum_name, variant, payload } => {
            assert_eq!(enum_name, "Result");
            assert_eq!(variant, "Ok");
            assert!(payload.is_some());
            if let Some(boxed_pattern) = payload {
                assert!(matches!(*boxed_pattern, Pattern::Identifier(ref name) if name == "value"));
            }
        }
        _ => panic!("Expected EnumVariant pattern for Ok"),
    }
}

#[test]
fn test_err_pattern_without_binding() {
    let pattern = err_pattern(None);
    
    match pattern {
        Pattern::EnumVariant { enum_name, variant, payload } => {
            assert_eq!(enum_name, "Result");
            assert_eq!(variant, "Err");
            assert!(payload.is_none());
        }
        _ => panic!("Expected EnumVariant pattern for Err"),
    }
}

#[test]
fn test_parse_result_ok_variant() {
    // Test parsing Result::Ok(42) syntax
    let input = "Result::Ok(42)";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_expression();
    assert!(result.is_ok());
    
    if let Ok(Expression::EnumVariant { enum_name, variant, payload }) = result {
        assert_eq!(enum_name, "Result");
        assert_eq!(variant, "Ok");
        assert!(payload.is_some());
    } else {
        panic!("Expected EnumVariant expression");
    }
}

#[test]
fn test_parse_option_some_variant() {
    // Test parsing Option::Some(42) syntax  
    let input = "Option::Some(42)";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_expression();
    assert!(result.is_ok());
    
    if let Ok(Expression::EnumVariant { enum_name, variant, .. }) = result {
        assert_eq!(enum_name, "Option");
        assert_eq!(variant, "Some");
    } else {
        panic!("Expected EnumVariant expression");
    }
}

#[test]
fn test_parse_option_none_variant() {
    // Test parsing Option::None syntax
    let input = "Option::None";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_expression();
    assert!(result.is_ok());
    
    if let Ok(Expression::EnumVariant { enum_name, variant, payload }) = result {
        assert_eq!(enum_name, "Option");
        assert_eq!(variant, "None");
        assert!(payload.is_none());
    } else {
        panic!("Expected EnumVariant expression");
    }
}