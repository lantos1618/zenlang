use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_metadata() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(
        info.methods[0].params[1].ty,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.methods[0].return_type,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        })
    );
}
