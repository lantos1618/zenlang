//! Async/await milestone 1 — surface (parser) and typing (typechecker) tests.
//!
//! These pin the `@async`/`@await` *surface and type rules*. State-machine
//! lowering is not yet implemented (ASYNC_PLAN.md milestone 1), so a *whole
//! program* that defines an `@async` function is rejected with the stable code
//! `E3082`; the runtime fixture for that is `#[ignore]`d below until lowering
//! lands. The parser and per-expression typing rules below are fully live.

use std::path::Path;

use zen::ast::{Declaration, Expression};
use zen::error::{Diagnostic, FileTable};
use zen::lexer::tokenize;
use zen::parser::parse;
use zen::typechecker::TypeChecker;

fn parse_source(source: &str) -> Vec<Declaration> {
    let tokens = tokenize(source, 0).expect("tokenize");
    parse(tokens, 0).expect("parse").declarations
}

/// Run the real frontend (module graph + typechecker) over a single source file
/// and return the diagnostics (empty when it type-checks clean).
fn frontend_diagnostics(source: &str) -> Vec<Diagnostic> {
    let tmp = tempfile::tempdir().expect("temp dir");
    let path = tmp.path().join("main.zen");
    std::fs::write(&path, source).expect("write source");
    diagnostics_for_path(&path)
}

fn diagnostics_for_path(path: &Path) -> Vec<Diagnostic> {
    let mut files = FileTable::default();
    let graph = match zen::module_system::load_module_graph(path, &mut files) {
        Ok(graph) => graph,
        Err(errs) => return errs.into_iter().map(Diagnostic::from).collect(),
    };
    let mut checker = TypeChecker::new();
    match checker.check_module_graph_entry(&graph) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics,
    }
}

fn has_code(diagnostics: &[Diagnostic], code: &str) -> bool {
    diagnostics.iter().any(|d| d.code() == code)
}

// ---------------------------------------------------------------------------
// Parser surface
// ---------------------------------------------------------------------------

#[test]
fn at_async_marks_function_async() {
    let decls = parse_source("f = @async (n: i32) i32 { n }\n");
    let Some(Declaration::Function {
        name, is_async, ..
    }) = decls.first()
    else {
        panic!("expected a function declaration, got {decls:?}");
    };
    assert_eq!(name, "f");
    assert!(is_async, "`@async` should set is_async on the function");
}

#[test]
fn plain_function_is_not_async() {
    let decls = parse_source("f = (n: i32) i32 { n }\n");
    let Some(Declaration::Function { is_async, .. }) = decls.first() else {
        panic!("expected a function declaration, got {decls:?}");
    };
    assert!(!is_async, "a plain function must not be async");
}

#[test]
fn at_await_parses_as_await_expression() {
    // Body tail is `@await g(n)`: an Await wrapping the call.
    let decls = parse_source("f = @async (n: i32) i32 { @await g(n) }\n");
    let Some(Declaration::Function { body, .. }) = decls.first() else {
        panic!("expected a function declaration");
    };
    let Expression::Block { expr: Some(tail), .. } = body else {
        panic!("expected a block body, got {body:?}");
    };
    let Expression::Await { expr, .. } = tail.as_ref() else {
        panic!("expected an Await tail, got {tail:?}");
    };
    assert!(
        matches!(expr.as_ref(), Expression::FunctionCall { name, .. } if name == "g"),
        "@await should wrap the call `g(n)`, got {expr:?}",
    );
}

#[test]
fn at_await_binds_tighter_than_binary_op() {
    // `@await a + b` must parse as `(@await a) + b`, matching unary `-`.
    let decls = parse_source("f = @async () i32 { @await a + b }\n");
    let Some(Declaration::Function { body, .. }) = decls.first() else {
        panic!("expected a function");
    };
    let Expression::Block { expr: Some(tail), .. } = body else {
        panic!("expected a block body");
    };
    assert!(
        matches!(tail.as_ref(), Expression::BinaryOp { left, .. }
            if matches!(left.as_ref(), Expression::Await { .. })),
        "expected `(@await a) + b`, got {tail:?}",
    );
}

#[test]
fn async_and_await_remain_plain_identifiers_without_the_at_sigil() {
    // Zen is keyword-free: bare `async`/`await` are ordinary identifiers.
    let decls = parse_source("async = 1\nawait = 2\n");
    assert_eq!(decls.len(), 2, "bare async/await should parse as bindings");
}

// ---------------------------------------------------------------------------
// Typechecker typing rules
// ---------------------------------------------------------------------------

#[test]
fn await_outside_async_is_e3080() {
    // `g` is async (so `g()` is a future), but `f` is a plain function — awaiting
    // there is illegal.
    let diags = frontend_diagnostics(
        "g = @async () i32 { 1 }\nf = () i32 { @await g() }\nmain = () i32 { 0 }\n",
    );
    assert!(
        has_code(&diags, "E3080"),
        "expected E3080 (await outside async), got {diags:?}",
    );
}

#[test]
fn await_of_non_future_is_e3081() {
    // Awaiting a plain `i32` (not a future) inside an async fn is E3081.
    let diags = frontend_diagnostics("f = @async () i32 { @await 1 }\nmain = () i32 { 0 }\n");
    assert!(
        has_code(&diags, "E3081"),
        "expected E3081 (await of non-future), got {diags:?}",
    );
}

#[test]
fn async_fn_is_gated_with_e3082_until_lowering_lands() {
    // A well-formed async program still cannot be compiled: lowering is pending.
    let diags = frontend_diagnostics(
        "g = @async () i32 { 1 }\nf = @async () i32 { @await g() }\nmain = () i32 { 0 }\n",
    );
    assert!(
        has_code(&diags, "E3082"),
        "expected E3082 (async lowering not implemented), got {diags:?}",
    );
    // The await itself must type-check (no spurious E3080/E3081) — the only
    // complaint is the lowering gate.
    assert!(
        !has_code(&diags, "E3080") && !has_code(&diags, "E3081"),
        "well-formed await must not trip the misuse codes, got {diags:?}",
    );
}

#[test]
fn awaiting_an_async_call_yields_the_inner_value_type() {
    // `g` returns i32 → `g()` is Future<i32> → `@await g()` is i32, which
    // satisfies `f`'s i32 return. Only the E3082 lowering gate should fire (no
    // return-type-mismatch E3030/E3031).
    let diags = frontend_diagnostics(
        "g = @async () i32 { 1 }\nf = @async () i32 { @await g() }\nmain = () i32 { 0 }\n",
    );
    assert!(
        !has_code(&diags, "E3030") && !has_code(&diags, "E3031"),
        "awaited future value should type as its inner type, got {diags:?}",
    );
}

// ---------------------------------------------------------------------------
// Runtime fixture — blocked on lowering
// ---------------------------------------------------------------------------

#[test]
#[ignore = "async state-machine lowering not implemented yet (ASYNC_PLAN.md milestone 1); \
            async programs are gated with E3082"]
fn async_await_ready_value_runs() {
    // When lowering lands, an async fn that awaits a ready value, driven by a
    // trivial block_on, should compile, link, run, and print the value.
    // Pinned here so the gap is visible and the test flips green when (2) ships.
    unimplemented!("blocked on async lowering");
}
