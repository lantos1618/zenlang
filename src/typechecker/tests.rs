use super::*;
use crate::ast::declarations::StructField;
use crate::ast::expressions::BinaryOp;
use crate::error::Span;

fn parse_program(src: &str) -> ast::Program {
    let mut files = crate::error::FileTable::new();
    let file_id = files.add_file("test.zen".to_string(), src.to_string());
    let tokens = crate::lexer::tokenize(src, file_id).expect("tokenize");
    crate::parser::parse(tokens, file_id).expect("parse")
}

mod resolver_metadata;

mod resolver_validation;

mod resolver_absence;

mod core_semantics;
mod declaration_validation;
mod generic_behaviors;

mod resolver_collection;

#[test]
fn scope_variable_lookup() {
    let mut tc = TypeChecker::new();
    tc.define_var("x", Type::I32);
    assert_eq!(tc.lookup_var("x"), Some(Type::I32));

    tc.push_scope();
    tc.define_var("y", Type::Bool);
    assert_eq!(tc.lookup_var("y"), Some(Type::Bool));
    assert_eq!(tc.lookup_var("x"), Some(Type::I32)); // parent scope

    tc.pop_scope();
    assert_eq!(tc.lookup_var("y"), None); // out of scope
}

#[test]
fn collect_struct_info() {
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Struct {
        name: "Point".into(),
        type_params: Vec::new(),
        fields: vec![
            StructField {
                name: "x".into(),
                ty: AstType::F64,
                default: None,
                mutable: false,
                span: Span::dummy(),
            },
            StructField {
                name: "y".into(),
                ty: AstType::F64,
                default: None,
                mutable: false,
                span: Span::dummy(),
            },
        ],
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    assert!(tc.structs.contains_key("Point"));
    assert_eq!(tc.structs["Point"].fields.len(), 2);
}

#[test]
fn collect_enum_info() {
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Enum {
        name: "OptionI32".into(),
        type_params: Vec::new(),
        variants: vec![
            EnumVariant {
                name: "Some".into(),
                payload: Some(AstType::I32),
                span: Span::dummy(),
            },
            EnumVariant {
                name: "None".into(),
                payload: None,
                span: Span::dummy(),
            },
        ],
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    assert!(tc.enums.contains_key("OptionI32"));
    assert_eq!(tc.enums["OptionI32"].variants.len(), 2);
}

#[test]
fn collect_import_info() {
    let program = parse_program(
        r#"
{ io, fmt } = std

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.collect_declarations(&program.declarations);
    assert_eq!(tc.imports.get("io"), Some(&vec!["std".to_string()]));
    assert_eq!(tc.imports.get("fmt"), Some(&vec!["std".to_string()]));
}

#[test]
fn ast_import_declaration_tasks_collect_import_bindings() {
    let program = parse_program("{ Channel, Mutex } = std.sync");

    let tasks = TypeChecker::collect_ast_import_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks[0].names,
        &["Channel".to_string(), "Mutex".to_string()]
    );
    assert_eq!(
        tasks[0].module_path,
        &["std".to_string(), "sync".to_string()]
    );
}

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

#[test]
fn check_module_graph_entry_uses_graph_import_bindings() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n")
        .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 { add(1, 2) }\n",
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");
    let entry = graph.module(graph.entry).expect("entry module");
    assert!(
        !entry
            .program
            .declarations
            .iter()
            .any(|decl| decl.name() == Some("add")),
        "graph entry should not merge imported declarations"
    );

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed imported signatures");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "main"));
    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "add"));
}

#[test]
fn check_module_graph_entry_seeds_imported_function_type_signatures() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let callbacks_path = tmp.path().join("callbacks.zen");
    std::fs::write(
        &callbacks_path,
        "pub apply = (callback: (i32) i32, value: i32) i32 { value }\n",
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ apply } = callbacks

main = () i32 {
    callback = (value: i32) i32 { value }
    apply(callback, 1)
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    tc.check_module_graph_entry(&graph)
        .expect("graph import bindings should seed function-typed signatures");
}

#[test]
fn check_module_graph_entry_specializes_imported_generic_functions() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let identity_path = tmp.path().join("identity.zen");
    std::fs::write(&identity_path, "pub id<T> = (value: T) T { value }\n")
        .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        "{ id } = identity\n\nmain = () i32 { id<i32>(1) }\n",
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed generic templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "id_i32"));
}

#[test]
fn check_module_graph_entry_specializes_imported_generic_enums() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let option_path = tmp.path().join("option.zen");
    std::fs::write(
        &option_path,
        r#"pub Option<T>:
    None,
    Some(T)

pub Result<T, E>:
    Ok(T),
    Err(E)
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Option, Result } = option

main = () i32 {
    maybe = Option<i32>.Some(7)
    result = Result<i32, str>.Ok(9)
    0
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed generic enum templates");

    assert!(typed.types.iter().any(|ty| ty.name == "Option_i32"));
    assert!(typed.types.iter().any(|ty| ty.name == "Result_i32_str"));
}

#[test]
fn check_module_graph_entry_seeds_public_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

pub Point.value = (self: Point) i32 {
    self.x
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    tc.check_module_graph_entry(&graph)
        .expect("imported public type should seed its public methods");
}

#[test]
fn check_module_graph_entry_does_not_seed_private_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

Point.value = (self: Point) i32 {
    self.x
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let err = TypeChecker::new()
        .check_module_graph_entry(&graph)
        .expect_err("private imported methods should not be seeded");

    assert!(
        err.iter()
            .any(|d| d.message.contains("type `Point` has no method `value`")),
        "expected private imported method diagnostic, got {err:?}"
    );
}

#[test]
fn check_module_graph_entry_specializes_public_generic_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

pub Point.keep<T> = (self: Point, value: T) T {
    value
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.keep<i32>(1)
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("imported public type should seed public generic method templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "Point.keep_i32"));
}

#[test]
fn check_program_with_symbols_validates_resolver_import_sources() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Import, "io", Some("other".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import source mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has source 'other', expected 'std'")),
        "expected resolver import source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_import_visibility() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Import, "io", true);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has visibility public, expected private")),
        "expected resolver import visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_import_absent_declaration_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Import, "io", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has parameter count metadata, expected none")),
        "expected resolver import parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has return type metadata, expected none")),
        "expected resolver import return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_import_absent_type_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Import, "io", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Import, "io", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Import, "io", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_type_parameter_names_for_test(Namespace::Import, "io", Some(vec!["T".to_string()]));
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Import,
        "io",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Import,
        "io",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Import,
        "io",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Import, "io", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Import, "io", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Import,
        "io",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Import, "io", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Import,
        "io",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import type metadata should fail");

    for expected in [
        "resolver import symbol 'io' has parameter names metadata, expected none",
        "resolver import symbol 'io' has parameter types metadata, expected none",
        "resolver import symbol 'io' has typed parameter types metadata, expected none",
        "resolver import symbol 'io' has typed return type metadata, expected none",
        "resolver import symbol 'io' has type parameter count metadata, expected none",
        "resolver import symbol 'io' has type parameter names metadata, expected none",
        "resolver import symbol 'io' has type parameter bounds metadata, expected none",
        "resolver import symbol 'io' has typed type parameter bound refs metadata, expected none",
        "resolver import symbol 'io' has field count metadata, expected none",
        "resolver import symbol 'io' has field types metadata, expected none",
        "resolver import symbol 'io' has typed field types metadata, expected none",
        "resolver import symbol 'io' has variant names metadata, expected none",
        "resolver import symbol 'io' has variant owner metadata, expected none",
        "resolver import symbol 'io' has variant payload count metadata, expected none",
        "resolver import symbol 'io' has variant payload type metadata, expected none",
        "resolver import symbol 'io' has typed variant payload type metadata, expected none",
        "resolver import symbol 'io' has behavior methods metadata, expected none",
        "resolver import symbol 'io' has typed behavior methods metadata, expected none",
        "resolver import symbol 'io' has behavior parents metadata, expected none",
        "resolver import symbol 'io' has typed behavior parents metadata, expected none",
        "resolver import symbol 'io' has behavior impls metadata, expected none",
        "resolver import symbol 'io' has typed behavior impls metadata, expected none",
        "resolver import symbol 'io' has behavior requires metadata, expected none",
        "resolver import symbol 'io' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver import metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_import_and_module_absent_mutability() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_mutability_for_test(Namespace::Import, "io", Some(true));
    symbols.set_mutability_for_test(Namespace::Module, "std", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import/module mutability metadata should fail");

    for expected in [
        "resolver import symbol 'io' has mutability metadata, expected none",
        "resolver module symbol 'std' has mutability metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver import/module mutability diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_module_symbols() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Module, "std", true);
    symbols.set_import_source_for_test(Namespace::Module, "std", Some("other".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has visibility public, expected private")),
        "expected resolver module visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has source 'other', expected none")),
        "expected resolver module source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_module_absent_declaration_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Module, "std", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has parameter count metadata, expected none")),
        "expected resolver module parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has return type metadata, expected none")),
        "expected resolver module return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_module_absent_type_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Module, "std", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Module, "std", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Module, "std", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_type_parameter_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["T".to_string()]),
    );
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Module,
        "std",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Module,
        "std",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Module,
        "std",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Module, "std", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Module, "std", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Module,
        "std",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Module, "std", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Module,
        "std",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module type metadata should fail");

    for expected in [
        "resolver module symbol 'std' has parameter names metadata, expected none",
        "resolver module symbol 'std' has parameter types metadata, expected none",
        "resolver module symbol 'std' has typed parameter types metadata, expected none",
        "resolver module symbol 'std' has typed return type metadata, expected none",
        "resolver module symbol 'std' has type parameter count metadata, expected none",
        "resolver module symbol 'std' has type parameter names metadata, expected none",
        "resolver module symbol 'std' has type parameter bounds metadata, expected none",
        "resolver module symbol 'std' has typed type parameter bound refs metadata, expected none",
        "resolver module symbol 'std' has field count metadata, expected none",
        "resolver module symbol 'std' has field types metadata, expected none",
        "resolver module symbol 'std' has typed field types metadata, expected none",
        "resolver module symbol 'std' has variant names metadata, expected none",
        "resolver module symbol 'std' has variant owner metadata, expected none",
        "resolver module symbol 'std' has variant payload count metadata, expected none",
        "resolver module symbol 'std' has variant payload type metadata, expected none",
        "resolver module symbol 'std' has typed variant payload type metadata, expected none",
        "resolver module symbol 'std' has behavior methods metadata, expected none",
        "resolver module symbol 'std' has typed behavior methods metadata, expected none",
        "resolver module symbol 'std' has behavior parents metadata, expected none",
        "resolver module symbol 'std' has typed behavior parents metadata, expected none",
        "resolver module symbol 'std' has behavior impls metadata, expected none",
        "resolver module symbol 'std' has typed behavior impls metadata, expected none",
        "resolver module symbol 'std' has behavior requires metadata, expected none",
        "resolver module symbol 'std' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver module metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_requires_resolver_impl_methods() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { "point" }
}
"#,
    );
    let symbols = SymbolTable::default();
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing impl method resolver symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing value symbol 'Point.stringify'")),
        "expected missing impl method symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_impl_method_signature() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(
        Namespace::Value,
        "Point.stringify",
        Some("i32".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver impl method signature mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Point.stringify' has return type 'i32', expected 'str'"
        )),
        "expected resolver impl method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_impl_function_type_signature() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}

Point: { x: i32 }

Point.implements(Mapper) {
    map = (value: Point, callback: (i32) i32) (i32) i32 {
        callback
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "Point.map", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver impl method function type mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Point.map' has return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver impl method function type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_impl_method_body_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str {
        label = "point"
        label
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "label");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver impl method body local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'label'")),
        "expected missing resolver impl method body local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_enum_variants() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Variant, "Some");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver enum variant symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing variant symbol 'Some'")),
        "expected missing enum variant symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_arity() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Value, "add", Some(1));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function arity mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'add' has parameter count 1, expected 2")),
        "expected resolver function arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_parameter_types() {
    let program = parse_program(
        r#"
add = (a: i32, b: f64) f64 { b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "add",
        Some(vec!["i32".to_string(), "i32".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function parameter type mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'add' has parameter types '(i32, i32)', expected '(i32, f64)'"
        )),
        "expected resolver function parameter type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_metadata() {
    let program = parse_program(
        r#"
apply = (callback: (i32) i32, value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "apply",
        Some(vec!["i32".to_string(), "i32".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function type parameter metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has parameter types '(i32, i32)', expected '((i32) i32, i32)'"
            )),
            "expected resolver function type parameter metadata diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_parameter_names() {
    let program = parse_program(
        r#"
add = (a: i32, b: f64) f64 { b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(
        Namespace::Value,
        "add",
        Some(vec!["a".to_string(), "other".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function parameter name mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'add' has parameter names '(a, other)', expected '(a, b)'"
        )),
        "expected resolver function parameter name diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_parameter_locals() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "a");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver parameter local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'a'")),
        "expected missing resolver parameter local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_parameter_local_mutability() {
    let program = parse_program(
        r#"
add = (mut a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("a", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver parameter local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has mutability immutable, expected mutable")),
        "expected resolver parameter local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_visibility_and_source() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Local, "a", true);
    symbols.set_import_source_for_test(Namespace::Local, "a", Some("std".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local visibility/source mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has visibility public, expected private")),
        "expected resolver local visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has source 'std', expected none")),
        "expected resolver local source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_absent_declaration_metadata() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has parameter count metadata, expected none")),
        "expected resolver local parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has return type metadata, expected none")),
        "expected resolver local return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_absent_type_metadata() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Local, "a", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(Namespace::Local, "a", Some(vec!["i32".to_string()]));
    symbols.set_parameter_types_for_test(Namespace::Local, "a", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Local, "a", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_type_parameter_names_for_test(Namespace::Local, "a", Some(vec!["T".to_string()]));
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Local,
        "a",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Local,
        "a",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Local,
        "a",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Local, "a", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Local, "a", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_variant_payload_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
    symbols.set_variant_payload_type_for_test(Namespace::Local, "a", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Local,
        "a",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Local,
        "a",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(Namespace::Local, "a", Some(vec!["Json".to_string()]));
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Local,
        "a",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local type metadata should fail");

    for expected in [
        "resolver local symbol 'a' has parameter names metadata, expected none",
        "resolver local symbol 'a' has parameter types metadata, expected none",
        "resolver local symbol 'a' has typed parameter types metadata, expected none",
        "resolver local symbol 'a' has typed return type metadata, expected none",
        "resolver local symbol 'a' has type parameter count metadata, expected none",
        "resolver local symbol 'a' has type parameter names metadata, expected none",
        "resolver local symbol 'a' has type parameter bounds metadata, expected none",
        "resolver local symbol 'a' has typed type parameter bound refs metadata, expected none",
        "resolver local symbol 'a' has field count metadata, expected none",
        "resolver local symbol 'a' has field types metadata, expected none",
        "resolver local symbol 'a' has typed field types metadata, expected none",
        "resolver local symbol 'a' has variant names metadata, expected none",
        "resolver local symbol 'a' has variant owner metadata, expected none",
        "resolver local symbol 'a' has variant payload count metadata, expected none",
        "resolver local symbol 'a' has variant payload type metadata, expected none",
        "resolver local symbol 'a' has typed variant payload type metadata, expected none",
        "resolver local symbol 'a' has behavior methods metadata, expected none",
        "resolver local symbol 'a' has typed behavior methods metadata, expected none",
        "resolver local symbol 'a' has behavior parents metadata, expected none",
        "resolver local symbol 'a' has typed behavior parents metadata, expected none",
        "resolver local symbol 'a' has behavior impls metadata, expected none",
        "resolver local symbol 'a' has typed behavior impls metadata, expected none",
        "resolver local symbol 'a' has behavior requires metadata, expected none",
        "resolver local symbol 'a' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver local metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_requires_resolver_var_decl_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    value = 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver var local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver var local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_var_decl_local_mutability() {
    let program = parse_program(
        r#"
main = () i32 {
    value ::= 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("value", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver var local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability immutable, expected mutable")),
        "expected resolver var local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    0
}
"#,
    );
    let symbols_program = parse_program(
        r#"
main = () i32 {
    value = 1
    0
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra local symbol 'value'")),
        "expected extra resolver local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_mutability_by_scope() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let inner_scope = symbols
        .symbols()
        .iter()
        .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
        .map(|symbol| symbol.scope_id)
        .max()
        .expect("inner value local");
    symbols.set_local_mutability_in_scope_for_test("value", inner_scope, Some(true));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver scoped local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability mutable, expected immutable")),
        "expected scoped resolver local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_pattern_locals() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    value ?
        | Some(inner) { inner }
        | None { 0 }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "inner");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver pattern local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'inner'")),
        "expected missing resolver pattern local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_top_level_expr_locals() {
    let program = parse_program(
        r#"
value := 1
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver top-level expr local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver top-level expr local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_closure_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (input: i32) i32 {
        inner = input
        inner
    }
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "inner");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver closure local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'inner'")),
        "expected missing resolver closure local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_closure_parameter_mutability() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (mut input: i32) i32 {
        input = input + 1
        input
    }
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("input", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver closure parameter mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'input' has mutability immutable, expected mutable")),
        "expected resolver closure parameter mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_struct_field_default_locals() {
    let program = parse_program(
        r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver struct field default local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver struct field default local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_behavior_default_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str {
        value = "{}"
        value
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver behavior default local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver behavior default local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_visibility() {
    let program = parse_program(
        r#"
pub exported = () i32 { 1 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Value, "exported", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'exported' has visibility private, expected public")),
        "expected resolver function visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_return_type() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "main", Some("bool".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function return mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'main' has return type 'bool', expected 'i32'")),
        "expected resolver function return diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_return_metadata() {
    let program = parse_program(
        r#"
factory = () (i32) i32 {
    (value: i32) i32 { value }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "factory", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function type return metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'factory' has return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver function type return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_typed_signature_metadata() {
    let program = parse_program(
        r#"
apply = (callback: (i32) i32) (i32) i32 {
    callback
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_types_for_test(Namespace::Value, "apply", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Value, "apply", Some(AstType::I32));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed function signature metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
            )),
            "expected resolver typed parameter diagnostic, got {err:?}"
        );
    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver typed return diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_counts() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_count_for_test(Namespace::Value, "identity", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic arity mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'identity' has type parameter count 0, expected 1")),
        "expected resolver function generic arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_names() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_names_for_test(
        Namespace::Value,
        "identity",
        Some(vec!["U".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic parameter name mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
        )),
        "expected resolver function generic parameter name diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
encode<T: Json> = (value: T) str { "encoded" }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Value,
        "encode",
        Some(vec![("T".to_string(), "Other".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'encode' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver function generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs() {
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
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Value,
        "identity",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: vec![AstType::Str],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic bound ref mismatch should fail");

    let expected = "resolver value symbol 'identity' has type parameter bound refs '(T: Json<str>)', expected '(T: Json<T>)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver function generic bound ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_absent_declaration_metadata() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Value, "main", Some("std".to_string()));
    symbols.set_field_count_for_test(Namespace::Value, "main", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Value,
        "main",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Value,
        "main",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Value, "main", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Value,
        "main",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Value, "main", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Value,
        "main",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::Named("Self".to_string())],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function declaration metadata should fail");

    for expected in [
        "resolver value symbol 'main' has source 'std', expected none",
        "resolver value symbol 'main' has field count metadata, expected none",
        "resolver value symbol 'main' has field types metadata, expected none",
        "resolver value symbol 'main' has typed field types metadata, expected none",
        "resolver value symbol 'main' has variant names metadata, expected none",
        "resolver value symbol 'main' has variant payload type metadata, expected none",
        "resolver value symbol 'main' has typed variant payload type metadata, expected none",
        "resolver value symbol 'main' has behavior methods metadata, expected none",
        "resolver value symbol 'main' has typed behavior methods metadata, expected none",
        "resolver value symbol 'main' has behavior parents metadata, expected none",
        "resolver value symbol 'main' has typed behavior parents metadata, expected none",
        "resolver value symbol 'main' has behavior impls metadata, expected none",
        "resolver value symbol 'main' has typed behavior impls metadata, expected none",
        "resolver value symbol 'main' has behavior requires metadata, expected none",
        "resolver value symbol 'main' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver function declaration metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_counts() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_count_for_test(Namespace::Type, "Box", Some(0));
    symbols.set_type_parameter_count_for_test(Namespace::Behavior, "Serializable", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic arity mismatches should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has type parameter count 0, expected 1")),
        "expected resolver type generic arity diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver behavior symbol 'Serializable' has type parameter count 0, expected 1"
        )),
        "expected resolver behavior generic arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_names() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_names_for_test(Namespace::Type, "Box", Some(vec!["U".to_string()]));
    symbols.set_type_parameter_names_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec!["U".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic parameter name mismatches should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has type parameter names '(U)', expected '(T)'")),
        "expected resolver type generic parameter name diagnostic, got {err:?}"
    );
    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver behavior generic parameter name diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_visibility() {
    let program = parse_program(
        r#"
pub Box<T>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Type, "Box", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has visibility private, expected public")),
        "expected resolver type visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_visibility() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Behavior, "Json", true);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver behavior symbol 'Json' has visibility public, expected private")),
        "expected resolver behavior visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
Box<T: Json>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Type,
        "Box",
        Some(vec![("T".to_string(), "Other".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver type generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds() {
    let program = parse_program(
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
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec![("T".to_string(), "Json<i32>".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter bounds '(T: Json<i32>)', expected '(T: Json<T>)'"
            )),
            "expected resolver behavior generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_like_absent_value_metadata() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Type, "Box", Some("std".to_string()));
    symbols.set_parameter_count_for_test(Namespace::Type, "Box", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Type, "Box", Some("i32".to_string()));
    symbols.set_return_type_for_test(Namespace::Type, "Box", Some(AstType::I32));
    symbols.set_import_source_for_test(Namespace::Behavior, "Json", Some("std".to_string()));
    symbols.set_parameter_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["value".to_string()]),
    );
    symbols.set_parameter_type_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Self".to_string()]),
    );
    symbols.set_parameter_types_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![AstType::SelfType]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type-like value metadata should fail");

    for expected in [
        "resolver type symbol 'Box' has source 'std', expected none",
        "resolver type symbol 'Box' has parameter count metadata, expected none",
        "resolver type symbol 'Box' has return type metadata, expected none",
        "resolver type symbol 'Box' has typed return type metadata, expected none",
        "resolver behavior symbol 'Json' has source 'std', expected none",
        "resolver behavior symbol 'Json' has parameter names metadata, expected none",
        "resolver behavior symbol 'Json' has parameter types metadata, expected none",
        "resolver behavior symbol 'Json' has typed parameter types metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver type-like value metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_method_signatures() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self, i32) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string(), "bool".to_string()],
            "str".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior method signature mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has methods '(encode(Self, bool) str)', expected '(encode(Self, i32) str)'"
            )),
            "expected resolver behavior method signature diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![(
            "map".to_string(),
            vec!["Self".to_string(), "i32".to_string()],
            "(i32) i32".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior function type method signature mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, i32) (i32) i32)', expected '(map(Self, (i32) i32) (i32) i32)'"
            )),
            "expected resolver behavior function type method signature diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_method_types() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "map".to_string(),
            parameter_names: vec!["__arg0".to_string(), "__arg1".to_string()],
            parameter_types: vec![AstType::SelfType, AstType::I32],
            return_type: AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            },
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed behavior method metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has typed methods '(map(__arg0: Self, __arg1: i32) (i32) i32)', expected '(map(__arg0: Self, __arg1: (i32) i32) (i32) i32)'"
            )),
            "expected resolver typed behavior method diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_method_signatures() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior method signature mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Json' has methods '(encode(Self) str)', expected '(encode(Self) T)'"
            )),
            "expected resolver generic behavior method signature diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures()
{
    let program = parse_program(
        r#"
Mapper<T>: behavior {
    map: (Self, (T) T) (T) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![(
            "map".to_string(),
            vec!["Self".to_string(), "T".to_string()],
            "(T) T".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior function type method mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, T) (T) T)', expected '(map(Self, (T) T) (T) T)'"
            )),
            "expected resolver generic behavior function type method diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_absent_type_metadata() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_count_for_test(Namespace::Behavior, "Json", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Behavior, "Json", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Behavior,
        "Json",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Behavior, "Json", Some(AstType::I32));
    symbols.set_behavior_impl_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Debug".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Debug".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior type metadata should fail");

    for expected in [
        "resolver behavior symbol 'Json' has field count metadata, expected none",
        "resolver behavior symbol 'Json' has field types metadata, expected none",
        "resolver behavior symbol 'Json' has typed field types metadata, expected none",
        "resolver behavior symbol 'Json' has variant names metadata, expected none",
        "resolver behavior symbol 'Json' has variant payload type metadata, expected none",
        "resolver behavior symbol 'Json' has typed variant payload type metadata, expected none",
        "resolver behavior symbol 'Json' has behavior impls metadata, expected none",
        "resolver behavior symbol 'Json' has typed behavior impls metadata, expected none",
        "resolver behavior symbol 'Json' has behavior requires metadata, expected none",
        "resolver behavior symbol 'Json' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver behavior type metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_names_for_test(Namespace::Behavior, "PrettyJson", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior parent metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver behavior symbol 'PrettyJson' has parents 'none', expected to include 'Json'"
        )),
        "expected resolver behavior parent metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_parent_names() {
    let program = parse_program(
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
    symbols.set_behavior_parent_names_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior parent metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'PrettyJson' has parents 'Json<i32>', expected to include 'Json<str>'"
            )),
            "expected resolver generic behavior parent metadata diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_parent_refs() {
    let program = parse_program(
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
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior parent ref mismatch should fail");

    let expected =
            "resolver behavior symbol 'PrettyJson' has parent refs 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior parent ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_accepts_resolver_behavior_parent_child_type_param_refs() {
    let program = parse_program(
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
    let mut tc = TypeChecker::new();

    tc.check_program_with_symbols(&program, &symbols)
        .expect("resolver parent type arg using child type parameter should validate");
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_names_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior parent metadata should fail");

    let expected =
        "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior parent metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior parent ref metadata should fail");

    let expected =
        "resolver behavior symbol 'PrettyJson' has parent refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior parent ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_impl_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_names_for_test(Namespace::Type, "Point", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior impl metadata mismatch should fail");

    let expected =
        "resolver type symbol 'Point' has behavior impls 'none', expected to include 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver behavior impl metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_impl_names() {
    let program = parse_program(
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
    symbols.set_behavior_impl_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior impl metadata mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior impls 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior impl metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_impl_refs() {
    let program = parse_program(
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
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior impl ref mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior impl refs 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior impl ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_required_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(Namespace::Type, "Point", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior requires metadata mismatch should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires 'none', expected to include 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_required_names() {
    let program = parse_program(
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
    symbols.set_behavior_required_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior requires metadata mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior requires 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_required_refs() {
    let program = parse_program(
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
    symbols.set_behavior_required_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior requires ref mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior requires refs 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior requires ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior impl metadata should fail");

    let expected = "resolver type symbol 'Point' has behavior impls 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior impl metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior impl ref metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior impl refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior impl ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_required_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior requires metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_required_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior requires ref metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior requires ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_counts() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_count_for_test(Namespace::Type, "Point", Some(1));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Point' has field count 1, expected 2")),
        "expected resolver struct field count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32, y: f64 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field type mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Point' has fields '(x: i32, y: i32)', expected '(x: i32, y: f64)'"
            )),
            "expected resolver struct field type diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_function_type_fields() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct function type field mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver struct function type field diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_typed_field_metadata() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed struct field metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver typed struct field diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_struct_field_types() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Box",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic struct field mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver type symbol 'Box' has fields '(value: i32)', expected '(value: T)'"
        )),
        "expected resolver generic struct field diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_and_enum_absent_kind_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Point", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Type,
        "Point",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Type, "Point", Some(AstType::I32));
    symbols.set_field_count_for_test(Namespace::Type, "Option", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct/enum kind metadata should fail");

    for expected in [
        "resolver type symbol 'Point' has variant names metadata, expected none",
        "resolver type symbol 'Point' has variant payload type metadata, expected none",
        "resolver type symbol 'Point' has typed variant payload type metadata, expected none",
        "resolver type symbol 'Option' has field count metadata, expected none",
        "resolver type symbol 'Option' has field types metadata, expected none",
        "resolver type symbol 'Option' has typed field types metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver struct/enum kind metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_counts() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_count_for_test(Namespace::Variant, "Some", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload count 0, expected 1")),
        "expected resolver enum variant payload count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_visibility() {
    let program = parse_program(
        r#"
pub Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Variant, "Some", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has visibility private, expected public")),
        "expected resolver enum variant visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_types() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Some",
        Some("bool".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload type mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload type 'bool', expected 'i32'")),
        "expected resolver enum variant payload type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("i32".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_typed_payload_metadata() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_for_test(Namespace::Variant, "Wrap", Some(AstType::I32));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed enum payload metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver typed enum payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback<T>: Wrap((T) T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Wrap' has payload type 'T', expected '(T) T'")),
        "expected resolver generic enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_payload_types() {
    let program = parse_program(
        r#"
Result<T, E>: Ok(T), Err(E)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Err",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Err' has payload type 'T', expected 'E'")),
        "expected resolver generic enum payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_variant_absent_other_metadata() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Variant, "Some", Some("std".to_string()));
    symbols.set_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_parameter_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["value".to_string()]),
    );
    symbols.set_parameter_type_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Variant, "Some", Some(vec![AstType::I32]));
    symbols.set_return_type_name_for_test(Namespace::Variant, "Some", Some("i32".to_string()));
    symbols.set_return_type_for_test(Namespace::Variant, "Some", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_type_parameter_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["T".to_string()]),
    );
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Variant, "Some", Some(vec!["Other".to_string()]));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver variant non-variant metadata should fail");

    for expected in [
            "resolver variant symbol 'Some' has source 'std', expected none",
            "resolver variant symbol 'Some' has parameter count metadata, expected none",
            "resolver variant symbol 'Some' has parameter names metadata, expected none",
            "resolver variant symbol 'Some' has parameter types metadata, expected none",
            "resolver variant symbol 'Some' has typed parameter types metadata, expected none",
            "resolver variant symbol 'Some' has return type metadata, expected none",
            "resolver variant symbol 'Some' has typed return type metadata, expected none",
            "resolver variant symbol 'Some' has type parameter count metadata, expected none",
            "resolver variant symbol 'Some' has type parameter names metadata, expected none",
            "resolver variant symbol 'Some' has type parameter bounds metadata, expected none",
            "resolver variant symbol 'Some' has typed type parameter bound refs metadata, expected none",
            "resolver variant symbol 'Some' has field count metadata, expected none",
            "resolver variant symbol 'Some' has field types metadata, expected none",
            "resolver variant symbol 'Some' has typed field types metadata, expected none",
            "resolver variant symbol 'Some' has variant names metadata, expected none",
            "resolver variant symbol 'Some' has behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has behavior requires metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver variant metadata diagnostic '{expected}', got {err:?}"
            );
        }
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", Some(vec!["Some".to_string()]));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant names mismatch should fail");

    let expected = "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant names diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_owner_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_owner_name_for_test(Namespace::Variant, "Some", Some("Result".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant owner mismatch should fail");

    let expected = "resolver variant symbol 'Some' has owner 'Result', expected 'Option'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant owner diagnostic, got {err:?}"
    );
}
