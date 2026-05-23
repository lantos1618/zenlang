use std::path::{Path, PathBuf};

use zen::error::{Diagnostic, FileTable};
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

pub(crate) fn write_tmp_module(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    path
}

pub(crate) fn frontend_diagnostics(zen_path: &Path) -> Vec<Diagnostic> {
    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();
    let graph = match module_system.load_module_graph(zen_path, &mut files) {
        Ok(graph) => graph,
        Err(errs) => return errs.into_iter().map(Diagnostic::from).collect(),
    };

    let mut checker = TypeChecker::new();
    checker
        .check_module_graph_entry(&graph)
        .expect_err("frontend diagnostics fixture should fail typechecking")
}

pub(crate) fn assert_diagnostic_code_and_message(
    diagnostics: &[Diagnostic],
    code: &str,
    message: &str,
    label: &str,
) {
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code && diagnostic.message.contains(message)),
        "expected {label} diagnostic {code} containing `{message}`, got {diagnostics:?}"
    );
}

pub(crate) fn assert_no_diagnostic_message(diagnostics: &[Diagnostic], message: &str, label: &str) {
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains(message)),
        "{label} should not include diagnostic containing `{message}`, got {diagnostics:?}"
    );
}
