mod generic_specializations;
mod import_signatures;
mod imported_methods;
use super::*;

fn module_graph_from_sources(imports: &[(&str, &str)], main_source: &str) -> ResolvedModuleGraph {
    let tmp = tempfile::tempdir().expect("temp dir");
    for (module_name, source) in imports {
        let path = tmp.path().join(format!("{module_name}.zen"));
        std::fs::write(&path, source).expect("write imported module");
    }

    let main_path = tmp.path().join("main.zen");
    std::fs::write(&main_path, main_source).expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph")
}

fn check_module_graph_sources(
    imports: &[(&str, &str)],
    main_source: &str,
) -> Result<TypedProgram, Vec<Diagnostic>> {
    let graph = module_graph_from_sources(imports, main_source);
    TypeChecker::new().check_module_graph_entry(&graph)
}
