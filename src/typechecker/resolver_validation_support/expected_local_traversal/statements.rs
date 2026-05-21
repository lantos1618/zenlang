fn expected_resolver_statement_locals(
    statement: &ast::Statement,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match statement {
        ast::Statement::VarDecl {
            name,
            value,
            mutable,
            constant,
            ..
        } => {
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            if resolver_var_decl_binds_local(name, *mutable, *constant, locals) {
                expected_resolver_var_decl_local(name, *mutable, locals, expected);
            }
        }
        ast::Statement::Assignment { target, value, .. } => {
            expected_resolver_expr_locals(target, scope_cursor, locals, expected);
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
        }
        ast::Statement::Expression { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        ast::Statement::Block { stmts, .. } => {
            expected_resolver_block_locals(stmts, None, scope_cursor, locals, expected);
        }
    }
}
