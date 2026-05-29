use super::*;

include!("import_seeding/imports_modules.rs");
include!("import_seeding/imports_graph_seeding.rs");
include!("import_seeding/imports_dependencies.rs");
include!("import_seeding/imports_behavior_extends.rs");
include!("import_seeding/imports_behavior_dependencies.rs");
include!("import_seeding/imports_source_dependencies.rs");
include!("import_seeding/imports_source_dependency_types.rs");
include!("import_seeding/imports_source_dependency_callables.rs");
include!("import_seeding/imported_method_seeding.rs");

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
