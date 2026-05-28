use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::{Declaration, Program};
use crate::error::{CompileError, FileTable, Span};
use crate::resolver::{Namespace, Resolver, SymbolTable};
use crate::root_spelling::{AT_BUILTIN_ROOT, AT_STD_ROOT, STD_ROOT};

use super::stdlib_paths::resolve_stdlib_file_path;
use super::{load_file, package_id_for};
use super::{ImportBinding, ModuleId, ModuleInfo, ResolvedModule, ResolvedModuleGraph};

pub fn load_module_graph(
    path: &Path,
    files: &mut FileTable,
) -> Result<ResolvedModuleGraph, Vec<CompileError>> {
    let mut graph = ResolvedModuleGraph {
        entry: ModuleId(0),
        modules: Default::default(),
        paths: Default::default(),
    };
    let mut loading = HashSet::new();
    let entry = load_graph_module(path, files, &mut graph, &mut loading)?;
    graph.entry = entry;
    Ok(graph)
}

fn load_graph_module(
    path: &Path,
    files: &mut FileTable,
    graph: &mut ResolvedModuleGraph,
    loading: &mut HashSet<PathBuf>,
) -> Result<ModuleId, Vec<CompileError>> {
    let canonical = path.canonicalize().map_err(|e| {
        vec![CompileError::Internal(format!(
            "cannot resolve path {}: {}",
            path.display(),
            e
        ))]
    })?;
    let key = canonical.display().to_string();

    if loading.contains(&canonical) {
        return Err(resolution_error(
            format!("circular import detected: {}", canonical.display()),
            None,
        ));
    }

    if let Some(id) = graph.paths.get(&key) {
        return Ok(*id);
    }

    loading.insert(canonical.clone());

    let program = load_file(path, files)?;
    let symbols = Resolver.resolve_program(&program).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| CompileError::Resolution(diagnostic.message, diagnostic.span))
            .collect::<Vec<_>>()
    })?;
    let id = ModuleId(graph.paths.len() as u32);
    let info = ModuleInfo {
        id,
        package_id: package_id_for(&canonical),
        canonical_path: key.clone(),
    };
    graph.paths.insert(key, id);

    let base_dir = canonical
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let imports = resolve_graph_imports(&program, &base_dir, files, graph, loading)?;

    loading.remove(&canonical);
    graph.modules.insert(
        id,
        ResolvedModule {
            info,
            program,
            imports,
            symbols,
        },
    );
    Ok(id)
}

fn resolve_graph_imports(
    program: &Program,
    base_dir: &Path,
    files: &mut FileTable,
    graph: &mut ResolvedModuleGraph,
    loading: &mut HashSet<PathBuf>,
) -> Result<Vec<ImportBinding>, Vec<CompileError>> {
    let mut bindings = Vec::new();

    for decl in &program.declarations {
        let Declaration::Import {
            names,
            module_path,
            span,
        } = decl
        else {
            continue;
        };
        if module_path.is_empty() {
            return Err(resolution_error("empty import path", Some(*span)));
        }

        let module_name = module_path.join(".");
        reject_duplicate_requested_imports(names, &module_name, *span)?;

        let first = &module_path[0];
        let is_std_root = matches!(first.as_str(), STD_ROOT | AT_STD_ROOT);
        if first == AT_BUILTIN_ROOT || is_std_root && module_path.len() == 1 {
            continue;
        }

        let file_path = if is_std_root {
            resolve_stdlib_file_path(&module_path[1..])?.ok_or_else(|| {
                resolution_error(
                    format!("cannot find stdlib module '{module_name}'"),
                    Some(*span),
                )
            })?
        } else {
            local_import_file_path(base_dir, module_path, &module_name, *span)?
        };

        let dep_id = load_graph_module(&file_path, files, graph, loading)?;
        let dep_module = graph
            .module(dep_id)
            .expect("graph module exists immediately after load");
        for name in names {
            match exported_module_symbol_is_public(&dep_module.symbols, name) {
                Some(true) => {
                    bindings.push(ImportBinding {
                        local_name: name.clone(),
                        source_module: dep_module.info.id,
                        source_symbol: name.clone(),
                        span: *span,
                    });
                }
                Some(false) => {
                    return Err(resolution_error(
                        format!("symbol '{name}' in module '{module_name}' is not exported"),
                        Some(*span),
                    ));
                }
                None => {
                    return Err(resolution_error(
                        format!("module '{module_name}' does not export '{name}'"),
                        Some(*span),
                    ));
                }
            }
        }
    }

    Ok(bindings)
}

fn resolution_error(message: impl Into<String>, span: Option<Span>) -> Vec<CompileError> {
    vec![CompileError::Resolution(message.into(), span)]
}

fn local_import_file_path(
    base_dir: &Path,
    module_path: &[String],
    module_name: &str,
    span: Span,
) -> Result<PathBuf, Vec<CompileError>> {
    let rel_path: PathBuf = module_path.iter().collect();
    let mut file_path = base_dir.join(&rel_path);
    if file_path.extension().is_none() {
        file_path.set_extension("zen");
    }
    if file_path.exists() {
        return Ok(file_path);
    }
    Err(resolution_error(
        format!(
            "cannot find imported module '{module_name}' (looked for {})",
            file_path.display()
        ),
        Some(span),
    ))
}

fn reject_duplicate_requested_imports(
    names: &[String],
    module_name: &str,
    span: Span,
) -> Result<(), Vec<CompileError>> {
    let mut requested_names = HashSet::new();
    for name in names {
        if !requested_names.insert(name.as_str()) {
            return Err(resolution_error(
                format!("duplicate import '{name}' from module '{module_name}'"),
                Some(span),
            ));
        }
    }
    Ok(())
}

fn exported_module_symbol_is_public(symbols: &SymbolTable, name: &str) -> Option<bool> {
    let mut found_private = false;
    for namespace in [Namespace::Value, Namespace::Type, Namespace::Behavior] {
        let Some(symbol) = symbols.lookup(namespace, name) else {
            continue;
        };
        if symbol.is_public {
            return Some(true);
        }
        found_private = true;
    }
    found_private.then_some(false)
}
