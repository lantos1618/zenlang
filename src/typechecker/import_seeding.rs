use super::*;

include!("import_seeding/imports_core.rs");
include!("import_seeding/imports_behaviors.rs");
include!("import_seeding/imports_source.rs");

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
