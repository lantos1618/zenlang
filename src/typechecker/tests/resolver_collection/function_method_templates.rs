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
