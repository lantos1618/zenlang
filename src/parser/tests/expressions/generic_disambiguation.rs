use super::*;

fn final_expr(src: &str) -> Expression {
    let prog = parse_ok(src);
    let Declaration::Function { body, .. } = &prog.declarations[0] else {
        panic!("expected function declaration");
    };
    let Expression::Block {
        expr: Some(expr), ..
    } = body
    else {
        panic!("expected final block expression");
    };
    expr.as_ref().clone()
}

#[test]
fn speculative_generic_lookahead_preserves_shift_right_tokens() {
    let expr = final_expr("f = (x: i32, y: i32, z: i32) bool { x < y >> z }");
    let Expression::BinaryOp {
        op: BinaryOp::Lt,
        right,
        ..
    } = expr
    else {
        panic!("expected top-level less-than expression");
    };
    let Expression::BinaryOp {
        op: BinaryOp::ShiftRight,
        ..
    } = right.as_ref()
    else {
        panic!("expected right side to keep shift-right expression, got {right:?}");
    };
}

#[test]
fn spaced_comparison_is_not_parsed_as_generic_function_call() {
    let expr = final_expr("f = (x: i32, y: i32, z: i32) bool { x < y > (z) }");
    let Expression::BinaryOp {
        op: BinaryOp::Gt,
        left,
        ..
    } = expr
    else {
        panic!("expected top-level greater-than expression");
    };
    let Expression::BinaryOp {
        op: BinaryOp::Lt, ..
    } = left.as_ref()
    else {
        panic!("expected left side to be less-than expression, got {left:?}");
    };
}

#[test]
fn type_argument_lists_require_commas_between_args() {
    let errs = parse_err("f = () i32 { identity<i32 f64>(1) }");
    assert!(
        errs.iter()
            .any(|err| format!("{err:?}").contains("expected `,` or `>`")),
        "expected missing type-argument comma diagnostic, got {errs:?}"
    );
}
