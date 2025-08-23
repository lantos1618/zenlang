use zen::ast::Expression;
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_parse_simple_member_access() {
    let input = "obj.field";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::MemberAccess { object, member } = expr {
        assert!(matches!(object.as_ref(), Expression::Identifier(name) if name == "obj"));
        assert_eq!(member, "field");
    } else {
        panic!("Expected MemberAccess expression, got {:?}", expr);
    }
}

#[test]
fn test_parse_chained_member_access() {
    let input = "obj.field1.field2";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    // Should parse as: MemberAccess(MemberAccess(obj, field1), field2)
    if let Expression::MemberAccess { object, member } = expr {
        assert_eq!(member, "field2");
        if let Expression::MemberAccess { object: inner_obj, member: inner_member } = object.as_ref() {
            assert_eq!(inner_member, "field1");
            assert!(matches!(inner_obj.as_ref(), Expression::Identifier(name) if name == "obj"));
        } else {
            panic!("Expected nested MemberAccess");
        }
    } else {
        panic!("Expected MemberAccess expression");
    }
}

#[test]
fn test_parse_member_access_with_function_call() {
    let input = "obj.method()";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    // Should parse as a function call with object.method name
    if let Expression::FunctionCall { name, args } = expr {
        assert_eq!(name, "obj.method");
        assert_eq!(args.len(), 0);
    } else {
        panic!("Expected FunctionCall expression, got {:?}", expr);
    }
}

#[test]
fn test_parse_member_access_with_args() {
    let input = "obj.method(1, 2)";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    if let Expression::FunctionCall { name, args } = expr {
        assert_eq!(name, "obj.method");
        assert_eq!(args.len(), 2);
        assert!(matches!(&args[0], Expression::Integer32(1)));
        assert!(matches!(&args[1], Expression::Integer32(2)));
    } else {
        panic!("Expected FunctionCall expression");
    }
}

#[test]
fn test_parse_complex_member_access() {
    let input = "person.address.city";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    // Should parse as: MemberAccess(MemberAccess(person, address), city)
    if let Expression::MemberAccess { object, member } = expr {
        assert_eq!(member, "city");
        if let Expression::MemberAccess { object: inner_obj, member: inner_member } = object.as_ref() {
            assert_eq!(inner_member, "address");
            assert!(matches!(inner_obj.as_ref(), Expression::Identifier(name) if name == "person"));
        } else {
            panic!("Expected nested MemberAccess");
        }
    } else {
        panic!("Expected MemberAccess expression");
    }
}

#[test]
fn test_parse_member_access_in_expression() {
    let input = "obj.x + obj.y";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let expr = parser.parse_expression().unwrap();
    
    // Should parse as BinaryOp with two MemberAccess expressions
    if let Expression::BinaryOp { left, op: _, right } = expr {
        assert!(matches!(left.as_ref(), Expression::MemberAccess { .. }));
        assert!(matches!(right.as_ref(), Expression::MemberAccess { .. }));
    } else {
        panic!("Expected BinaryOp expression");
    }
}