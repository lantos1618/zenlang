use super::*;

#[test]
fn check_program_with_symbols_requires_resolver_declarations() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let empty_symbols = SymbolTable::default();
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &empty_symbols)
        .expect_err("missing resolver symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing value symbol 'main'")),
        "expected missing resolver symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_declarations() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
main = () i32 { 0 }
extra = () i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver declarations should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra value symbol 'extra'")),
        "expected extra resolver symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_imports_when_ast_imports_are_present() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
{ io, math } = std
main = () i32 { 0 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver imports should fail when AST imports are present");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra import symbol 'math'")),
        "expected extra resolver import diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_modules_when_ast_imports_are_present() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
{ io } = std
{ helper } = other
main = () i32 { 0 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver modules should fail when AST imports are present");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra module symbol 'other'")),
        "expected extra resolver module diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_method_receiver_type() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point.label = () str { "point" }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Type, "Point");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing receiver type resolver symbol should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing type symbol 'Point'")),
        "expected missing method receiver type symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_method_signature() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "Box.get",
        Some(vec!["Box<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver method signature mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Box.get' has parameter types '(Box<i32>)', expected '(Box<T>)'"
        )),
        "expected resolver method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_method_function_type_signature() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.map<T> = (self: Box<T>, callback: (T) T) (T) T {
    callback
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "Box.map",
        Some(vec!["Box<T>".to_string(), "T".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver method function type mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Box.map' has parameter types '(Box<T>, T)', expected '(Box<T>, (T) T)'"
            )),
            "expected resolver method function type diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_uses_resolver_import_bindings() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 {
    io.println("ok")
    0
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));

    let mut tc = TypeChecker::new();
    tc.check_program_with_symbols(&program, &symbols)
        .expect("resolver import symbols should seed typechecker imports");

    assert!(tc.is_root_std_import("io"));
}

#[test]
fn check_program_with_symbols_validates_stripped_resolver_import_sources() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Import, "io", None);
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver imports without sources should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver import symbol 'io' has source 'unknown', expected a module source"
        )),
        "expected stripped resolver import source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_stripped_resolver_import_visibility() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Import, "io", true);
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver import visibility should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has visibility public, expected private")),
        "expected stripped resolver import visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_stripped_resolver_import_modules() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Module, "std");
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver imports should require source module symbols");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing module symbol 'std'")),
        "expected stripped resolver import module diagnostic, got {err:?}"
    );
}
