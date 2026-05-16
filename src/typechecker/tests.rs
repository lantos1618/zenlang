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

mod resolver_behavior_impls_requires;
mod resolver_behavior_parents;
mod resolver_declarations;
mod resolver_impl_values;
mod resolver_import_metadata;
mod resolver_locals;
mod resolver_module_graph;
mod resolver_struct_enum_metadata;
mod resolver_type_behavior_metadata;
mod resolver_value_metadata;
