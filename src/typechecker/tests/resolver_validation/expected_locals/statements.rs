use super::*;

#[test]
fn expected_resolver_statement_locals_preserve_mutable_handoff() {
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut locals = scope_cursor.new_scope();
    locals.insert("value".to_string(), true);
    let mut expected = HashSet::new();

    if resolver_var_decl_binds_local("value", false, false, &locals) {
        expected_resolver_var_decl_local("value", false, &mut locals, &mut expected);
    }

    assert!(
        expected.iter().all(|(name, _)| name != "value"),
        "immutable declaration should reuse the mutable handoff binding"
    );
}
