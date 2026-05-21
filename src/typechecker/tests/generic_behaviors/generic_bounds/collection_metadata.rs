use super::*;

#[test]
fn func_info_non_generic_has_empty_type_params() {
    use crate::ast::Expression;
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Function {
        name: "add".into(),
        type_params: Vec::new(),
        params: vec![
            crate::ast::Param {
                name: "a".into(),
                ty: AstType::I32,
                mutable: false,
                span: Span::dummy(),
            },
            crate::ast::Param {
                name: "b".into(),
                ty: AstType::I32,
                mutable: false,
                span: Span::dummy(),
            },
        ],
        return_type: Some(AstType::I32),
        body: Expression::Block {
            statements: Vec::new(),
            expr: None,
            span: Span::dummy(),
        },
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    let info = tc.functions.get("add").unwrap();
    assert!(info.type_params.is_empty());
}
