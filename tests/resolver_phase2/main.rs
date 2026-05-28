use zen::error::FileTable;
use zen::resolver::{
    BehaviorRefMetadata, Namespace, Resolver, Symbol, SymbolTable, TypeParameterBoundRefMetadata,
};
use zen::{lexer, parser};

fn parse_program(src: &str) -> zen::ast::Program {
    let mut files = FileTable::default();
    let file_id = files.add_file("test.zen".to_string(), src);
    let tokens = lexer::tokenize(src, file_id).expect("tokenize");
    parser::parse(tokens, file_id).expect("parse")
}

fn scoped_symbol<'a>(table: &'a SymbolTable, namespace: Namespace, name: &str) -> &'a Symbol {
    table
        .symbols()
        .iter()
        .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        .expect("scoped symbol")
}

fn symbol<'a>(table: &'a SymbolTable, namespace: Namespace, name: &str) -> &'a Symbol {
    table
        .lookup(namespace, name)
        .unwrap_or_else(|| panic!("{namespace:?} symbol `{name}`"))
}

fn assert_string_metadata(actual: Option<&[String]>, expected: &[&str]) {
    let actual = actual.map(|items| items.iter().map(String::as_str).collect::<Vec<_>>());
    assert_eq!(actual.as_deref(), Some(expected));
}

fn assert_type_parameter_bound_metadata(
    actual: Option<&[TypeParameterBoundRefMetadata]>,
    expected: &[(&str, &str)],
) {
    let actual = actual.map(|items| {
        items
            .iter()
            .map(|bound| {
                (
                    bound.type_parameter.clone(),
                    behavior_ref_display(&bound.behavior, &bound.type_args),
                )
            })
            .collect::<Vec<_>>()
    });
    let expected = expected
        .iter()
        .map(|(param, behavior)| ((*param).to_string(), (*behavior).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(actual, Some(expected));
}

fn assert_type_metadata(actual: Option<&[zen::ast::AstType]>, expected: &[&str]) {
    let actual = actual.map(|items| {
        items
            .iter()
            .map(zen::ast::AstType::display_name)
            .collect::<Vec<_>>()
    });
    assert_string_metadata(actual.as_deref(), expected);
}

fn assert_type_name(actual: Option<&zen::ast::AstType>, expected: Option<&str>) {
    assert_eq!(
        actual.map(zen::ast::AstType::display_name).as_deref(),
        expected
    );
}

fn assert_field_type_metadata(
    actual: Option<&[(String, zen::ast::AstType)]>,
    expected: &[(&str, &str)],
) {
    let actual = actual.map(|items| {
        items
            .iter()
            .map(|(name, ty)| (name.clone(), ty.display_name()))
            .collect::<Vec<_>>()
    });
    let expected = expected
        .iter()
        .map(|(name, ty)| ((*name).to_string(), (*ty).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(actual, Some(expected));
}

fn assert_method_signature_metadata(
    actual: Option<&[zen::resolver::BehaviorMethodTypeMetadata]>,
    expected: &[(&str, &[&str], &str)],
) {
    let actual = actual.map(|items| {
        items
            .iter()
            .map(|method| {
                (
                    method.name.as_str(),
                    method
                        .parameter_types
                        .iter()
                        .map(zen::ast::AstType::display_name)
                        .collect::<Vec<_>>(),
                    method.return_type.display_name(),
                )
            })
            .collect::<Vec<_>>()
    });
    let expected = expected
        .iter()
        .map(|(name, params, ret)| {
            (
                *name,
                params
                    .iter()
                    .map(|param| (*param).to_string())
                    .collect::<Vec<_>>(),
                (*ret).to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, Some(expected));
}

fn assert_behavior_refs(
    actual: Option<&[BehaviorRefMetadata]>,
    expected: &[(&str, Vec<zen::ast::AstType>)],
) {
    let expected = expected
        .iter()
        .map(|(name, type_args)| BehaviorRefMetadata {
            name: (*name).to_string(),
            type_args: type_args.clone(),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, Some(expected.as_slice()));
}

fn behavior_ref_display(behavior: &str, type_args: &[zen::ast::AstType]) -> String {
    if type_args.is_empty() {
        behavior.to_string()
    } else {
        let args = type_args
            .iter()
            .map(zen::ast::AstType::display_name)
            .collect::<Vec<_>>()
            .join(", ");
        format!("{behavior}<{args}>")
    }
}

fn resolved_symbols(src: &str) -> SymbolTable {
    Resolver
        .resolve_program(&parse_program(src))
        .expect("resolve")
}

fn resolver_errors(src: &str, reason: &str) -> Vec<zen::error::Diagnostic> {
    Resolver
        .resolve_program(&parse_program(src))
        .expect_err(reason)
}

fn assert_resolver_error_contains(errors: &[zen::error::Diagnostic], needle: &str) {
    assert!(
        errors.iter().any(|d| d.message.contains(needle)),
        "expected resolver diagnostic containing `{needle}`, got {errors:?}"
    );
}

mod behavior_relations;
mod core_symbols;
mod enum_metadata;
mod expr_locals;
mod generic_behavior_associations;
mod generic_behavior_metadata;
mod impls;
mod struct_metadata;
mod value_metadata;
