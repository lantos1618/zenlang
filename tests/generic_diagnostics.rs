use zen::error::Diagnostic;
use zen::lexer;
use zen::parser;
use zen::resolver::Resolver;
use zen::typechecker::TypeChecker;

fn typecheck_errors(source: &str) -> Vec<Diagnostic> {
    let tokens = lexer::tokenize(source, 0).expect("lex source");
    let program = parser::parse(tokens, 0).expect("parse source");
    let symbols = Resolver::new()
        .resolve_program(&program)
        .expect("resolve source");
    TypeChecker::new()
        .check_program_with_symbols(&program, &symbols)
        .expect_err("typecheck should fail")
}

fn frontend_errors(source: &str) -> Vec<Diagnostic> {
    let tokens = lexer::tokenize(source, 0).expect("lex source");
    let program = parser::parse(tokens, 0).expect("parse source");
    match Resolver::new().resolve_program(&program) {
        Ok(symbols) => TypeChecker::new()
            .check_program_with_symbols(&program, &symbols)
            .expect_err("typecheck should fail"),
        Err(errors) => errors,
    }
}

#[path = "generic_diagnostics/annotations.rs"]
mod annotations;
#[path = "generic_diagnostics/behavior_impls.rs"]
mod behavior_impls;
#[path = "generic_diagnostics/bounds.rs"]
mod bounds;
#[path = "generic_diagnostics/composite_annotations.rs"]
mod composite_annotations;
#[path = "generic_diagnostics/constructors.rs"]
mod constructors;
#[path = "generic_diagnostics/inference_conflicts.rs"]
mod inference_conflicts;
#[path = "generic_diagnostics/method_type_args.rs"]
mod method_type_args;
