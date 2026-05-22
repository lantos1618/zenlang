use super::*;

#[test]
fn source_ast_no_longer_has_return_expression_nodes() {
    for path in [
        "src/ast/expressions.rs",
        "src/ast/typed.rs",
        "src/typechecker/expressions.rs",
        "src/typechecker/expressions/simple_forms.rs",
        "src/codegen/c/emit.rs",
        "src/codegen/c/types.rs",
    ] {
        let source = read(&path);
        for forbidden in [
            "Expression::Return",
            "TypedExprKind::Return",
            "check_return_expr",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} still contains dead return-expression support: {forbidden}"
            );
        }
    }
}

#[test]
fn source_ast_does_not_carry_dead_char_literal_nodes() {
    for path in [
        "src/ast/expressions.rs",
        "src/typechecker/expressions.rs",
        "src/resolver/expression_validation.rs",
        "src/build_graph/lowering.rs",
        "src/typechecker/self_type_validation/expressions.rs",
        "src/typechecker/generic_type_reference_walker/expressions.rs",
        "src/typechecker/resolver_validation/local_traversal.rs",
        "src/typechecker/resolver_validation_support/expected_local_traversal.rs",
    ] {
        let source = read(path);
        for forbidden in ["CharLiteral", "TODO: implement char literal type"] {
            assert!(
                !source.contains(forbidden),
                "{path} still contains dead char-literal AST support: {forbidden}"
            );
        }
    }
}
