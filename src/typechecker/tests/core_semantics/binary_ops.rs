use super::*;

#[test]
fn binary_op_types() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.check_binary_op(BinaryOp::Add, &Type::I32, &Type::I32, &Span::dummy())
            .unwrap(),
        Type::I32
    );
    assert_eq!(
        tc.check_binary_op(BinaryOp::Eq, &Type::I32, &Type::I32, &Span::dummy())
            .unwrap(),
        Type::Bool
    );
    assert_eq!(
        tc.check_binary_op(BinaryOp::And, &Type::Bool, &Type::Bool, &Span::dummy())
            .unwrap(),
        Type::Bool
    );
}

#[test]
fn binary_op_type_mismatch() {
    let tc = TypeChecker::new();
    // Arithmetic on non-numeric type
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::I32, &Type::Str, &Span::dummy())
        .is_err());
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::Bool, &Type::I32, &Span::dummy())
        .is_err());
    // Logical op on non-bool
    assert!(tc
        .check_binary_op(BinaryOp::And, &Type::I32, &Type::Bool, &Span::dummy())
        .is_err());
    // Unknown is permissive (error recovery)
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::Unknown, &Type::Str, &Span::dummy())
        .is_ok());
}

#[test]
fn binary_op_mixed_numeric_width_requires_cast() {
    let tc = TypeChecker::new();
    let err = tc
        .check_binary_op(BinaryOp::Add, &Type::I32, &Type::I64, &Span::dummy())
        .expect_err("mixed integer arithmetic should fail");
    assert!(
        err.message
            .contains("arithmetic operands must have the same type"),
        "expected mixed numeric diagnostic, got {err:?}"
    );

    let err = tc
        .check_binary_op(BinaryOp::Mul, &Type::F32, &Type::F64, &Span::dummy())
        .expect_err("mixed float arithmetic should fail");
    assert!(
        err.message
            .contains("arithmetic operands must have the same type"),
        "expected mixed numeric diagnostic, got {err:?}"
    );
}
