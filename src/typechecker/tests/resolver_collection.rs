use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_type_metadata() {
    let mut program = parse_program(
        r#"
apply = (callback: (i32) i32) (i32) i32 {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.functions.get("apply").expect("function info");
    assert_eq!(
        info.params[0].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.return_type,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_function_signature() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "main", None);
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should not keep AST-only function metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_function_signature_after_name_restore() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "main", None);
    if let Declaration::Function {
        name,
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST function signature key after resolver name restoration"
        );
    assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should clear the restored function signature key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_function_template() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should not keep AST-only generic function templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_generic_function_body_refs_when_signature_incomplete(
) {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T {
    same: T = value
    same
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should remove generic template when resolver signature metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST generic body refs when resolver signature metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_signature_for_type_refs() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Missing".to_string());
        *return_type = Some(AstType::Named("AlsoMissing".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.get = (self: Box, value: i32) i32 { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        params[1].ty = AstType::Named("Missing".to_string());
        *return_type = Some(AstType::Named("AlsoMissing".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored method signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { method_name, .. } = &mut program.declarations[1] {
        *method_name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Point.missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        type_name,
        method_name,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Missing.missing"));
}

#[test]
fn collect_declarations_with_symbols_clears_stale_method_signature_after_key_restore() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.get", None);
    if let Declaration::Method {
        type_name,
        method_name,
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST method signature key after resolver key restoration"
        );
    assert!(
            !tc.methods.contains_key("Point.get"),
            "resolver-backed collection should clear the restored method signature key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_name_metadata() {
    let mut program = parse_program(
        r#"
main = () i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { name, .. } = &mut program.declarations[0] {
        *name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.functions.contains_key("main"));
    assert!(!tc.functions.contains_key("missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_name_metadata() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { name, .. } = &mut program.declarations[0] {
        *name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.generic_functions.contains_key("identity"));
    assert!(!tc.generic_functions.contains_key("missing"));
}

#[test]
fn collect_declarations_with_symbols_clears_stale_generic_function_template_after_name_restore() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function {
        name,
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST generic function template key after resolver name restoration"
        );
    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should clear the restored generic function template key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_type_params_for_type_refs() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs() {
    let mut program = parse_program(
        r#"
Box<T>: { value: T }
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "StaleBox".to_string();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[1] {
        type_params[0].name = "StaleOption".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored type metadata should avoid stale AST type-ref diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_type_params_for_type_refs() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored function bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("MissingBox".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
        type_params[0].constraint = Some("MissingOption".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored type bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_type_bounds() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Box", None);
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Option", None);
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("MissingBox".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
        type_params[0].constraint = Some("MissingOption".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.structs
                .get("Box")
                .expect("struct info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed struct collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
    assert!(
            tc.enums
                .get("Option")
                .expect("enum info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed enum collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_bounds() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Behavior, "Serializable", None);
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Serializable").expect("behavior info");
    assert!(
            info.type_param_bounds.is_empty(),
            "resolver-backed behavior collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box: { value: i32 }
Box.impl = {
    keep<T: Json<T>> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { type_params, .. } = &mut methods[0] {
            type_params[0].constraint = Some("Missing".to_string());
            type_params[0].constraint_type_args.clear();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method bounds should avoid stale AST generic-bound diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Point.missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_target_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[1] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Missing.get"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_impl_method_signature() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.get", None);
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Point.get"),
            "resolver-backed collection should not keep AST-only impl method metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_name_metadata()
{
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert!(!tc.generic_methods.contains_key("Box.missing"));
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "value");
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Box: { value: i32 }

Box.impl = {
    apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
        callback
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[3] {
        if let Declaration::Function {
            type_params,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.apply")
        .expect("generic impl method template");
    assert_eq!(template.type_params, vec!["U".to_string()]);
    assert_eq!(
        tc.methods
            .get("Box.apply")
            .expect("impl method info")
            .type_param_bounds
            .get("U"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("U".to_string())],
        })
    );
    assert_eq!(
        template.params[1].ty,
        AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_return_presence(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T {
        value
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { return_type, .. } = &mut methods[0] {
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_parameter_count(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    choose<T> = (self: Box, left: T, right: T) T {
        left
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.pop();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic impl method template");
    assert_eq!(template.params.len(), 3);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("Box".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[2].ty, AstType::Named("T".to_string()));
}

#[test]
fn collect_declarations_with_symbols_preserves_type_impl_generic_template_param_mutability_by_position(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, mut value: T) T {
        value = value
        value
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params[1].name = "stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert_eq!(template.params[1].name, "value");
    assert!(
        template.params[1].mutable,
        "resolver-restored impl method parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_type_impl_generic_template_param_names_for_mutability(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    choose<T> = (self: Box, left: T, mut right: T) T {
        right = right
        right
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.swap(1, 2);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic impl method template");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert!(
        template.params[1].mutable,
        "resolver-restored first non-self impl parameter should keep first AST position mutability"
    );
    assert!(
            !template.params[2].mutable,
            "resolver-restored second non-self impl parameter should keep second AST position mutability"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_impl_method_template() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[1].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should not keep AST-only generic impl method templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_generic_impl_method_template_after_key_restore() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[1].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST generic impl method template key after resolver key restoration"
        );
    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should clear the restored generic impl method template key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_target_and_name_metadata(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert!(!tc.generic_methods.contains_key("Missing.missing"));
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "value");
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_name_for_body_type_refs(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T {
        same: T = value
        same
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            name, type_params, ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            type_params[0].name = "Stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic impl method name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
apply<T: Json<T>> = (callback: (T) T) (T) T {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        type_params,
        params,
        return_type,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        params[0].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc.generic_functions.get("apply").expect("generic template");
    assert_eq!(template.type_params, vec!["T".to_string()]);
    assert_eq!(
        tc.functions
            .get("apply")
            .expect("function info")
            .type_param_bounds
            .get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        template.params[0].ty,
        AstType::Function {
            params: vec![AstType::Named("T".to_string())],
            ret: Box::new(AstType::Named("T".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("T".to_string())],
            ret: Box::new(AstType::Named("T".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_name_for_body_type_refs() {
    let mut program = parse_program(
        r#"
keep<T> = (value: T) T {
    same: T = value
    same
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        name, type_params, ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic function name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_generic_function_template_type_params_when_resolver_bounds_missing(
) {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Value, "identity", None);
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("identity")
        .expect("generic function template");
    assert!(
            template.type_params.is_empty(),
            "resolver-backed generic templates should not keep type parameter names when typed bound metadata is incomplete"
        );
    assert!(
            tc.functions
                .get("identity")
                .expect("function info")
                .type_params
                .is_empty(),
            "function info and template type parameter handoff should agree when resolver metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_return_presence() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T {
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { return_type, .. } = &mut program.declarations[0] {
        *return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("identity")
        .expect("generic template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_parameter_count() {
    let mut program = parse_program(
        r#"
choose<T> = (left: T, right: T) T {
    left
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("choose")
        .expect("generic template");
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "left");
    assert_eq!(template.params[1].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
}

#[test]
fn collect_declarations_with_symbols_preserves_generic_template_param_mutability_by_position() {
    let mut program = parse_program(
        r#"
keep<T> = (mut value: T) T {
    value = value
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc.generic_functions.get("keep").expect("generic template");
    assert_eq!(template.params[0].name, "value");
    assert!(
        template.params[0].mutable,
        "resolver-restored parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_generic_template_param_names_for_mutability() {
    let mut program = parse_program(
        r#"
choose<T> = (left: T, mut right: T) T {
    right = right
    right
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params.swap(0, 1);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("choose")
        .expect("generic template");
    assert_eq!(template.params[0].name, "left");
    assert_eq!(template.params[1].name, "right");
    assert!(
        template.params[0].mutable,
        "resolver-restored first parameter should keep first AST position mutability"
    );
    assert!(
        !template.params[1].mutable,
        "resolver-restored second parameter should keep second AST position mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Box: { value: i32 }
Box.apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        type_params,
        params,
        return_type,
        ..
    } = &mut program.declarations[3]
    {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        params[1].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.apply")
        .expect("generic method template");
    assert_eq!(template.type_params, vec!["U".to_string()]);
    assert_eq!(
        tc.methods
            .get("Box.apply")
            .expect("method info")
            .type_param_bounds
            .get("U"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("U".to_string())],
        })
    );
    assert_eq!(
        template.params[1].ty,
        AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_clears_generic_method_template_type_params_when_resolver_bounds_missing(
) {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box: { value: i32 }
Box.keep<U: Json<U>> = (self: Box, value: U) U { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Value, "Box.keep", None);
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic method template");
    assert!(
            template.type_params.is_empty(),
            "resolver-backed generic method templates should not keep type parameter names when typed bound metadata is incomplete"
        );
    assert!(
            tc.methods
                .get("Box.keep")
                .expect("method info")
                .type_params
                .is_empty(),
            "method info and template type parameter handoff should agree when resolver metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_template_return_presence() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T {
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { return_type, .. } = &mut program.declarations[1] {
        *return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic method template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_template_parameter_count() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.choose<T> = (self: Box, left: T, right: T) T {
    left
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic method template");
    assert_eq!(template.params.len(), 3);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("Box".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[2].ty, AstType::Named("T".to_string()));
}

#[test]
fn collect_declarations_with_symbols_preserves_generic_method_template_param_mutability_by_position(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, mut value: T) T {
    value = value
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params[1].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic method template");
    assert_eq!(template.params[1].name, "value");
    assert!(
        template.params[1].mutable,
        "resolver-restored method parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_generic_method_template_param_names_for_mutability(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.choose<T> = (self: Box, left: T, mut right: T) T {
    right = right
    right
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params.swap(1, 2);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic method template");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert!(
            template.params[1].mutable,
            "resolver-restored first non-self method parameter should keep first AST position mutability"
        );
    assert!(
            !template.params[2].mutable,
            "resolver-restored second non-self method parameter should keep second AST position mutability"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_method_template() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::Method {
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        params[1].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should not keep AST-only generic method templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_generic_method_template_after_key_restore() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
    if let Declaration::Method {
        type_name,
        method_name,
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
        params[1].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST generic method template key after resolver key restoration"
        );
    assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should clear the restored generic method template key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_method_name_for_body_type_refs() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T {
    same: T = value
    same
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        method_name,
        type_params,
        ..
    } = &mut program.declarations[1]
    {
        *method_name = "missing".to_string();
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic method name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_field_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Pipeline<T: Json<T>>: { callback: (i32) i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct {
        type_params,
        fields,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        fields[0].ty = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.structs.get("Pipeline").expect("struct info");
    assert_eq!(
        info.type_param_bounds.get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        info.fields[0].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.structs.contains_key("Point"));
    assert!(!tc.structs.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_field_names_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
        fields[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should use resolver-restored field names: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_struct_field_defaults_validate_from_type_metadata_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Struct { fields, .. } = &mut stale_declarations[0] {
        fields.clear();
    }
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| checker.collect_declarations(&stale_declarations));
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_resolver_struct_field_default_tasks(&tasks, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should use precollected type tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_backed_struct_field_defaults_reuse_metadata_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Struct { fields, .. } = &mut stale_declarations[0] {
        fields.clear();
    }
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| checker.collect_declarations(&stale_declarations));
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_struct_field_defaults(&program.declarations, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should reuse shared metadata tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_backed_semantic_validation_reuses_metadata_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations)
    });
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_collected_declaration_semantics(&program.declarations, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed semantic validation should reuse resolver metadata tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_struct_fields() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
        fields[0].ty = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should not keep AST-only struct fields when resolver field metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete(
) {
    let mut program = parse_program(
        r#"
Box<T>: {
    value: T = {
        same: T = 1
        same
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Box", None);
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Box"),
            "resolver-backed collection should remove struct fields when resolver field metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST struct field default refs when resolver field metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_struct_fields_after_name_restore() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    if let Declaration::Struct { name, fields, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        fields[0].ty = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST struct key after resolver name restoration"
        );
    assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should clear the restored struct key when resolver field metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_enum_payload_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Callback<T: Json<T>>: Wrap((i32) i32), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Enum {
        type_params,
        variants,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        variants[0].payload = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.enums.get("Callback").expect("enum info");
    assert_eq!(
        info.type_param_bounds.get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        info.variants[0].1,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_enum_name_metadata() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Enum { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.enums.contains_key("Option"));
    assert!(!tc.enums.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_enum_variants() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    if let Declaration::Enum { variants, .. } = &mut program.declarations[0] {
        variants[0].payload = Some(AstType::Named("Stale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should not keep AST-only enum variants when resolver variant metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_enum_variants_after_name_restore() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    if let Declaration::Enum { name, variants, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        variants[0].payload = Some(AstType::Named("Stale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.enums.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST enum key after resolver name restoration"
        );
    assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should clear the restored enum key when resolver variant metadata is incomplete"
        );
}

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

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_name_metadata() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.behaviors.contains_key("Json"));
    assert!(!tc.behaviors.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_methods() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
        methods[0].params[1].ty = AstType::Named("Stale".to_string());
        methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should not keep AST-only behavior methods when resolver method metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_behavior_default_body_refs_when_methods_incomplete(
) {
    let mut program = parse_program(
        r#"
Mapper<T>: behavior {
    map: (Self, value: T) T {
        same: T = value
        same
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should remove behavior methods when resolver method metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST behavior default body refs when resolver method metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_methods_after_name_restore() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { name, methods, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        methods[0].params[1].ty = AstType::Named("Stale".to_string());
        methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST behavior key after resolver name restoration"
        );
    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should clear the restored behavior key when resolver method metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].name, "encode");
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method name metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_return_presence_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].return_type, Some(AstType::Str));
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method return metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.push(Param {
            name: "stale".to_string(),
            ty: AstType::I32,
            mutable: false,
            span: Span::dummy(),
        });
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].params.len(), 1);
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_missing_parameter_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (Self, i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(info.methods[0].params.len(), 2);
    assert_eq!(info.methods[0].params[0].name, "__arg0");
    assert_eq!(info.methods[0].params[1].name, "__arg1");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert_eq!(info.methods[0].params[1].ty, AstType::I32);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_names() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].params[0].name, "value");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_behavior_method_parameter_order() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.swap(0, 1);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(info.methods[0].params[0].name, "value");
    assert_eq!(info.methods[0].params[1].name, "input");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert_eq!(info.methods[0].params[1].ty, AstType::I32);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
    describe: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
    describe = (value: Point) str { "desc" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods.len(), 2);
    assert_eq!(info.methods[0].name, "encode");
    assert_eq!(info.methods[1].name, "describe");
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior methods should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32 { callback }
}

Point.implements(Mapper) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.map").expect("default method info");
    assert_eq!(
        info.params[1].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.return_type,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored default method");
    assert_eq!(info.params[0].0, "self");
    assert_eq!(info.return_type, AstType::Str);
    assert!(
        !tc.methods.contains_key("Point.missing"),
        "stale AST-only behavior default method name should not be synthesized"
    );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior default method name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_skips_default_when_resolver_restores_impl_method_name() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let method = tc
        .methods
        .get("Point.encode")
        .expect("restored impl method");
    assert_eq!(
        method.params[0].0, "value",
        "resolver-restored explicit impl method should not be overwritten by the behavior default"
    );
    assert!(
        !tc.methods.contains_key("Point.missing"),
        "stale AST-only impl method key should be removed"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method name should suppress default insertion: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_behavior_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[3] {
        *behavior = Some("Debug".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let method = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored behavior default");
    assert_eq!(method.params[0].0, "self");
    assert_eq!(method.return_type, AstType::Str);
    assert!(
        !tc.methods.contains_key("Point.describe"),
        "stale AST-only behavior default should not be synthesized"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl metadata should drive default synthesis: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[2] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl target should drive omitted default synthesis: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("Debug".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        !tc.methods.contains_key("Point.describe"),
        "stale AST-only behavior default should not be synthesized"
    );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl target and name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_defers_impl_checks_until_resolver_metadata_is_collected() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { callback }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_metadata_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { callback }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method name metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    extra = (value: Point) str { "extra" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "encode".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| *message == "method `extra` is not declared by behavior `Json`"),
        "resolver-owned extra impl method should not be hidden by stale AST required name: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_parameter_names_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params[0].name = "stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.encode").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_impl_method_parameter_order_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.swap(0, 1);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.map").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[1].0, "input");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert_eq!(info.params[1].1, AstType::I32);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[2] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl target should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.impl = {
    get = (self: Point) i32 { self.x }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    assert!(
        tc.methods.contains_key("Point.get"),
        "non-behavior impl methods should still be refreshed by declaration metadata"
    );
    assert!(
        !tc.methods.contains_key("Point.encode"),
        "behavior impl method signatures should be owned by the behavior impl metadata pass"
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_method_signature()
{
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should not keep AST-only method metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_impl_method_signature_after_key_restore()
{
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        !tc.methods.contains_key("Missing.missing"),
        "resolver-backed behavior impl collection should not keep stale AST method keys"
    );
    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored method keys when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_method_signature_target_and_name_metadata(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.encode").expect("impl method info");
    assert!(!tc.methods.contains_key("Missing.missing"));
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert_eq!(info.return_type, AstType::Str);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl method signature should avoid stale AST diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Point.encode")
        .expect("generic behavior impl method template");
    assert!(!tc.generic_methods.contains_key("Missing.missing"));
    assert!(!tc.generic_methods.contains_key("Point.missing"));
    assert_eq!(template.type_params, vec!["T".to_string()]);
    assert_eq!(template.params.len(), 1);
    assert_eq!(template.params[0].name, "value");
    assert_eq!(template.params[0].ty, AstType::Named("Point".to_string()));
    assert_eq!(template.return_type, Some(AstType::Str));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl generic template should avoid stale AST diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        !tc.generic_methods.contains_key("Missing.missing"),
        "resolver-backed behavior impl collection should clear stale AST generic method templates"
    );
    assert!(
            !tc.generic_methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored generic method templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    assert_eq!(parents[0].behavior, "Json");
    assert_eq!(parents[0].type_args, vec![AstType::Str]);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_refs_for_test(Namespace::Behavior, "PrettyJson", None);
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = &mut program.declarations[2]
    {
        parent_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_extends.contains_key("PrettyJson"),
            "resolver-backed collection should not keep AST-only behavior parent refs when resolver parent metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args() {
    let mut program = parse_program(
        r#"
Marker<T>: behavior {
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Marker<str>)
PrettyJson.extends(Marker<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = second_parent
    {
        parent_type_args[0] = AstType::Str;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
    assert_eq!(parent_keys, vec!["Marker_str", "Marker_i32"]);
    assert!(
        tc.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("duplicate behavior inheritance")),
        "resolver-restored parent type args should avoid false duplicate diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_parent_and_type_param_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[2] {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc.behavior_extends.get("Pretty").expect("behavior parents");
    assert_eq!(parents[0].behavior, "Serializable");
    assert_eq!(parents[0].type_args, vec![AstType::Named("T".to_string())]);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent and type-parameter metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message == "type `Point` implementation of `PrettyJson` is missing required method `encode`"
            }),
            "resolver-restored parent metadata should report the inherited missing method, got {:?}",
            messages
        );
    assert!(
        messages.iter().all(|message| !message.contains("Missing")),
        "stale AST-only behavior parent names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_reports_conflict_from_restored_parent_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Debug<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = second_parent
    {
        parent_type_args[0] = AstType::Str;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message == "conflicting behavior method `encode` inherited by `PrettyJson`"
            }),
            "resolver-restored parent type args should drive inherited method coherence diagnostics, got {:?}",
            messages
        );
    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
    assert_eq!(parent_keys, vec!["Json_str", "Debug_i32"]);
}

#[test]
fn collect_declarations_with_symbols_reports_cycle_from_restored_parent_refs() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
PrettyJson: behavior {
    pretty: (Self) str
}
Debug: behavior {
    debug: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends { parent, .. } = second_parent {
        *parent = "Debug".to_string();
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("behavior inheritance cycle")),
        "resolver-restored parent refs should drive cycle diagnostics, got {:?}",
        messages
    );
    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    assert_eq!(parents[0].behavior, "Json");
}

#[test]
fn collect_declarations_with_symbols_synthesizes_defaults_from_restored_behavior_parent() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str { "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends { parent, .. } = &mut program.declarations[3] {
        *parent = "Missing".to_string();
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.methods.contains_key("Point.encode"),
        "resolver-restored parent metadata should synthesize inherited default method"
    );
    assert!(
        !tc.methods.contains_key("Point.Missing"),
        "stale AST-only parent names should not synthesize default methods"
    );
}

#[test]
fn collect_declarations_with_symbols_synthesizes_generic_defaults_from_restored_parent_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let encode = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored parent should synthesize inherited default");
    assert_eq!(
        encode.return_type,
        AstType::Str,
        "resolver-restored parent type args should drive inherited default return type"
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = &mut program.declarations[2]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string())),
        "resolver metadata should restore the validated Json<str> impl"
    );
    assert!(
        !tc.behavior_impls
            .contains(&("Point".to_string(), "Json_i32".to_string())),
        "AST-only Json<i32> impl drift should not remain after resolver collection"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = &mut program.declarations[2]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_synthesize_stale_impl_defaults_after_target_restore()
{
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
    assert!(
        !tc.methods.contains_key("Missing.encode"),
        "resolver-backed default synthesis should not keep stale AST target method keys"
    );
    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed default synthesis should not synthesize behavior defaults when resolver impl metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only impl refs after target restoration when resolver impl metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[2] {
        *behavior = Some("Missing".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl name metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string())),
        "resolver metadata should restore the validated Point implements Json<str> association"
    );
    assert!(
            !tc.behavior_impls
                .contains(&("Missing".to_string(), "AlsoMissing_i32".to_string())),
            "stale AST-only impl target and behavior metadata should not remain after resolver collection"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl target and behavior metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_impl_target_and_name() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message == "type `Point` implementation of `Json_str` is missing required method `encode`"
            }),
            "resolver-restored impl metadata should report the validated missing method, got {:?}",
            messages
        );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
        "stale AST-only impl names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_reports_overlap_from_restored_impl_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let first_impl = program
        .declarations
        .iter_mut()
        .find(|declaration| {
            matches!(
                declaration,
                Declaration::ImplBlock {
                    behavior: Some(behavior),
                    ..
                } if behavior == "Json"
            )
        })
        .expect("Json impl declaration");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = first_impl
    {
        behavior_type_args[0] = AstType::I32;
    } else {
        panic!("expected Json impl declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message
                    == "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
            }),
            "resolver-restored impl type args should drive overlap diagnostics, got {:?}",
            messages
        );
    assert!(
        messages.iter().all(|message| !message.contains("Json_i32")),
        "stale AST-only impl type args should not leak into overlap diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_impl_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_impl = program
        .declarations
        .iter_mut()
        .filter(|declaration| {
            matches!(
                declaration,
                Declaration::ImplBlock {
                    behavior: Some(behavior),
                    ..
                } if behavior == "Json"
            )
        })
        .nth(1)
        .expect("second Json impl declaration");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = second_impl
    {
        behavior_type_args[0] = AstType::Str;
    } else {
        panic!("expected second Json impl declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("duplicate implementation")),
        "resolver-restored impl type args should avoid false duplicate diagnostics, got {:?}",
        messages
    );
    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string()))
            && tc
                .behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
        "resolver-restored impl type args should keep distinct impl specializations: {:?}",
        tc.behavior_impls
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        behavior_type_args, ..
    } = &mut program.declarations[3]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored requires metadata should avoid stale AST requires diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires { type_name, .. } = &mut program.declarations[3] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target and behavior metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_required_target_and_name() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| *message
                == "type `Point` does not implement required behavior `Json_str`"),
        "resolver-restored requires metadata should report the validated missing impl, got {:?}",
        messages
    );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
        "stale AST-only requires names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_uses_restored_requires_ref_for_inherited_impl() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let requires = program
        .declarations
        .iter_mut()
        .find(|declaration| matches!(declaration, Declaration::Requires { .. }))
        .expect("requires declaration");
    if let Declaration::Requires {
        behavior,
        behavior_type_args,
        ..
    } = requires
    {
        *behavior = "Missing".to_string();
        behavior_type_args[0] = AstType::I32;
    } else {
        panic!("expected requires declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored requires ref should be satisfied by inherited child impl: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_distinct_restored_requires_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_requires = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::Requires { .. }))
        .nth(1)
        .expect("second requires declaration");
    if let Declaration::Requires {
        behavior_type_args, ..
    } = second_requires
    {
        behavior_type_args[0] = AstType::Str;
    } else {
        panic!("expected requires declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("does not implement required behavior")),
        "resolver-restored requires type args should keep distinct satisfied specializations: {:?}",
        tc.diagnostics
    );
    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string()))
            && tc
                .behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
        "resolver-restored impl refs should keep both required specializations available: {:?}",
        tc.behavior_impls
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_required_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::Requires {
        behavior_type_args, ..
    } = &mut program.declarations[3]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_requires_after_target_restore() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs after target restoration when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires { behavior, .. } = &mut program.declarations[3] {
        *behavior = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires name metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}
