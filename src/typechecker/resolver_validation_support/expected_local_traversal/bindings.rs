fn expected_resolver_var_decl_local(
    name: &str,
    mutable: bool,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected_resolver_local(name, mutable, locals, expected);
}

fn resolver_var_decl_binds_local(
    name: &str,
    mutable: bool,
    constant: bool,
    locals: &ResolverLocalScope,
) -> bool {
    constant || mutable || !locals.is_mutable(name)
}

fn expected_resolver_pattern_binding(
    name: &str,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected_resolver_local(name, false, locals, expected);
}

fn expected_resolver_local(
    name: &str,
    mutable: bool,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected.insert((name.to_string(), locals.current_scope_id));
    locals.insert(name.to_string(), mutable);
}
