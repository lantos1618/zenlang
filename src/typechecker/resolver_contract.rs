use super::*;

include!("resolver_contract/imports_modules.rs");
include!("resolver_contract/imports_graph_seeding.rs");
include!("resolver_contract/imports_dependencies.rs");
include!("resolver_contract/imports_behavior_extends.rs");
include!("resolver_contract/imports_behavior_dependencies.rs");
include!("resolver_contract/imports_source_dependencies.rs");
include!("resolver_contract/imports_source_dependency_types.rs");
include!("resolver_contract/imports_source_dependency_callables.rs");
include!("resolver_contract/imported_method_seeding.rs");

fn module_declaration<'a>(module: &'a ResolvedModule, name: &str) -> Option<&'a Declaration> {
    module
        .program
        .declarations
        .iter()
        .find(|decl| decl.name() == Some(name))
}

fn public_type_declaration<'a>(module: &'a ResolvedModule, name: &str) -> Option<&'a Declaration> {
    module_declaration(module, name).filter(|decl| {
        decl.is_public() && matches!(decl, Declaration::Struct { .. } | Declaration::Enum { .. })
    })
}
