use super::*;
use crate::ast::{self, AstType, BinaryOperator, Declaration, Expression, Statement};

#[test]
fn test_variant_name_expression() {
    let node = ASTNodeValue::Expression(Expression::Integer32(42));
    assert_eq!(variant_name(&node), "Integer32");
}

#[test]
fn test_variant_name_binary_op() {
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer32(2)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Integer32(3)),
    };
    assert_eq!(variant_name(&ASTNodeValue::Expression(expr)), "BinaryOp");
}

#[test]
fn test_fields_integer() {
    let node = ASTNodeValue::Expression(Expression::Integer32(42));
    let flds = fields(&node).unwrap();
    assert_eq!(flds.len(), 1);
    if let ComptimeValue::Struct { fields: f, .. } = &flds[0] {
        if let ComptimeValue::String(name) = &f["name"] {
            assert_eq!(name, "value");
        }
    }
}

#[test]
fn test_fields_binary_op() {
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer32(2)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Integer32(3)),
    };
    let flds = fields(&ASTNodeValue::Expression(expr)).unwrap();
    assert_eq!(flds.len(), 3);
}

#[test]
fn test_fields_function_call() {
    let expr = Expression::FunctionCall {
        module: None,
        name: "add".to_string(),
        type_args: vec![],
        args: vec![Expression::Integer32(1), Expression::Integer32(2)],
        span: None,
    };
    let flds = fields(&ASTNodeValue::Expression(expr)).unwrap();
    assert_eq!(flds.len(), 3);
}

#[test]
fn test_type_info_returns_struct() {
    let node = ASTNodeValue::Expression(Expression::Integer32(42));
    let info = type_info(&node).unwrap();
    if let ComptimeValue::Struct { name, fields: f } = &info {
        assert_eq!(name, "TypeInfo");
        assert!(f.contains_key("variant"));
        assert!(f.contains_key("fields"));
        assert!(f.contains_key("kind"));
        if let ComptimeValue::String(v) = &f["variant"] {
            assert_eq!(v, "Integer32");
        }
    } else {
        panic!("Expected TypeInfo struct");
    }
}

#[test]
fn test_fields_variable_declaration() {
    let stmt = Statement::VariableDeclaration {
        name: "x".to_string(),
        type_: Some(AstType::I32),
        initializer: Some(Expression::Integer32(10)),
        is_mutable: false,
        declaration_type: ast::VariableDeclarationType::InferredImmutable,
        span: None,
    };
    let flds = fields(&ASTNodeValue::Statement(stmt)).unwrap();
    assert_eq!(flds.len(), 5);
}

#[test]
fn test_fields_function_declaration() {
    let func = ast::Function {
        name: "add".to_string(),
        type_params: vec![],
        args: vec![
            ("a".to_string(), AstType::I32),
            ("b".to_string(), AstType::I32),
        ],
        return_type: AstType::I32,
        body: vec![Statement::Return {
            expr: Expression::BinaryOp {
                left: Box::new(Expression::Identifier("a".to_string())),
                op: BinaryOperator::Add,
                right: Box::new(Expression::Identifier("b".to_string())),
            },
            span: None,
        }],
        is_varargs: false,
        is_public: false,
    };
    let flds = fields(&ASTNodeValue::Declaration(Declaration::Function(func))).unwrap();
    assert_eq!(flds.len(), 7);
}

#[test]
fn test_fields_program() {
    let prog = ast::Program {
        declarations: vec![],
        statements: vec![],
    };
    let flds = fields(&ASTNodeValue::Program(prog)).unwrap();
    assert_eq!(flds.len(), 2);
}

#[test]
fn test_variant_name_all_statement_types() {
    let stmts = vec![
        Statement::Expression {
            expr: Expression::Unit,
            span: None,
        },
        Statement::Return {
            expr: Expression::Unit,
            span: None,
        },
        Statement::Break {
            label: None,
            span: None,
        },
        Statement::Continue {
            label: None,
            span: None,
        },
    ];
    let expected = vec!["Expression", "Return", "Break", "Continue"];
    for (stmt, exp) in stmts.iter().zip(expected.iter()) {
        assert_eq!(variant_name(&ASTNodeValue::Statement(stmt.clone())), *exp);
    }
}

#[test]
fn test_variant_constants_expression() {
    let variants = expression_variants();
    if let ComptimeValue::Struct { name, fields } = &variants {
        assert_eq!(name, "Expression");
        assert_eq!(
            fields.get("BinaryOp"),
            Some(&ComptimeValue::String("BinaryOp".to_string()))
        );
        assert_eq!(
            fields.get("FunctionCall"),
            Some(&ComptimeValue::String("FunctionCall".to_string()))
        );
        assert_eq!(fields.get("Nonexistent"), None);
    } else {
        panic!("Expected Struct");
    }
}

#[test]
fn test_variant_constants_match_variant_name() {
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer32(1)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Integer32(2)),
    };
    let vname = variant_name(&ASTNodeValue::Expression(expr));
    let variants = expression_variants();
    if let ComptimeValue::Struct { fields, .. } = &variants {
        assert_eq!(
            fields.get(&vname),
            Some(&ComptimeValue::String(vname.clone()))
        );
    }
}

#[test]
fn test_variant_constants_all_enums() {
    let e = expression_variants();
    let s = statement_variants();
    let d = declaration_variants();
    let t = type_variants();
    let p = pattern_variants();

    for (val, expected_name) in [
        (&e, "Expression"),
        (&s, "Statement"),
        (&d, "Declaration"),
        (&t, "AstType"),
        (&p, "Pattern"),
    ] {
        if let ComptimeValue::Struct { name, fields } = val {
            assert_eq!(name, expected_name);
            assert!(!fields.is_empty());
        } else {
            panic!("Expected Struct for {}", expected_name);
        }
    }
}
