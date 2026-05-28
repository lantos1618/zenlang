use zen::{error::Diagnostic, lexer, parser, resolver::Resolver, typechecker::TypeChecker};

const BOX: &str = r#"
Box<T>: {
    value: T
}
"#;

const OPTION: &str = r#"
Option<T>:
    None,
    Some(T)
"#;

const POINT: &str = r#"
Point: {
    x: i32
}
"#;

const STATUS: &str = r#"
Status:
    Ready,
    Done(i32)
"#;

const BOX_OPTION: &str = r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)
"#;

fn typecheck_errors(source: &str) -> Vec<Diagnostic> {
    let tokens = lexer::tokenize(source, 0).expect("lex source");
    let program = parser::parse(tokens, 0).expect("parse source");
    TypeChecker::new()
        .check_program(&program)
        .expect_err("typecheck should fail")
}

fn frontend_errors(source: &str) -> Vec<Diagnostic> {
    let tokens = lexer::tokenize(source, 0).expect("lex source");
    let program = parser::parse(tokens, 0).expect("parse source");
    match Resolver.resolve_program(&program) {
        Ok(_) => TypeChecker::new()
            .check_program(&program)
            .expect_err("typecheck should fail"),
        Err(errors) => errors,
    }
}

fn assert_diagnostic_code_and_message(
    errors: &[Diagnostic],
    code: &str,
    message_fragment: &str,
    context: &str,
) {
    assert!(
        errors
            .iter()
            .any(|diagnostic| diagnostic.code() == code
                && diagnostic.message.contains(message_fragment)),
        "expected {code} {context} diagnostic containing `{message_fragment}`, got {errors:?}"
    );
}

fn assert_diagnostic_message(errors: &[Diagnostic], message_fragment: &str, context: &str) {
    assert!(
        errors.iter().any(|d| d.message.contains(message_fragment)),
        "expected {context} diagnostic containing `{message_fragment}`, got {errors:?}"
    );
}

fn assert_inference_conflict(
    errors: &[Diagnostic],
    kind: &str,
    callee: &str,
    param: &str,
    inferred: &str,
    actual: &str,
    context: &str,
) {
    assert_diagnostic_code_and_message(
        errors,
        "E5000",
        &format!(
            "conflicting inferred type argument `{param}` for generic {kind} `{callee}`: inferred `{inferred}` and `{actual}`"
        ),
        context,
    );
}

fn assert_generic_arity_diagnostic(
    errors: &[Diagnostic],
    kind: &str,
    name: &str,
    expected: usize,
    found: usize,
    context: &str,
) {
    assert_diagnostic_code_and_message(
        errors,
        "E5001",
        &format!("generic {kind} `{name}` expects {expected} type arguments, found {found}"),
        context,
    );
}

fn assert_nongeneric_type_args_diagnostic(
    errors: &[Diagnostic],
    kind: &str,
    name: &str,
    context: &str,
) {
    assert_diagnostic_code_and_message(
        errors,
        "E5002",
        &format!("non-generic {kind} `{name}` does not accept type arguments"),
        context,
    );
}

fn assert_point_json_bound_failure(errors: &[Diagnostic], param: &str, context: &str) {
    assert_diagnostic_code_and_message(
        errors,
        "E6004",
        &format!("type `Point` does not implement behavior `Json` required by `{param}`"),
        context,
    );
}

fn assert_no_diagnostic_message(errors: &[Diagnostic], message: &str, context: &str) {
    assert!(
        errors.iter().all(|d| !d.message.contains(message)),
        "{context} should not include diagnostic containing `{message}`, got {errors:?}"
    );
}

mod annotations;
mod behavior_impls;
mod bounds;
mod call_site_annotations;
mod call_site_bounds;
mod composite_annotations;
mod constructors;
mod inference_conflicts;
mod method_type_args;
mod module_calls;
