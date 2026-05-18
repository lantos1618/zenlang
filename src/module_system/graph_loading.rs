use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::{Declaration, Program};
use crate::error::{CompileError, FileTable, Span};
use crate::resolver::{Namespace, Resolver, SymbolTable};

use super::root_prefix::parse_module_root_prefix;
use super::{ImportBinding, ModuleId, ModuleSystem, ResolvedModule, ResolvedModuleGraph};

impl ModuleSystem {
    /// Load a module graph with validated import bindings.
    ///
    /// Unlike `load_with_imports`, this does not merge imported declarations
    /// into the entry module AST. It records imports as bindings between module
    /// IDs so resolver/typechecker integration can move away from AST cloning.
    pub fn load_module_graph(
        &mut self,
        path: &Path,
        files: &mut FileTable,
    ) -> Result<ResolvedModuleGraph, Vec<CompileError>> {
        let mut graph = ResolvedModuleGraph {
            entry: ModuleId(0),
            modules: Default::default(),
            paths: Default::default(),
        };
        let mut loading = HashSet::new();
        let entry = self.load_graph_module(path, files, &mut graph, &mut loading)?;
        graph.entry = entry;
        Ok(graph)
    }

    fn load_graph_module(
        &mut self,
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
            return Err(vec![CompileError::Resolution(
                format!("circular import detected: {}", canonical.display()),
                None,
            )]);
        }

        if let Some(id) = graph.paths.get(&key) {
            return Ok(*id);
        }

        loading.insert(canonical.clone());

        let program = self.load_file(path, files)?;
        let symbols = Resolver::new()
            .resolve_program(&program)
            .map_err(|diagnostics| {
                diagnostics
                    .into_iter()
                    .map(|diagnostic| CompileError::Resolution(diagnostic.message, diagnostic.span))
                    .collect::<Vec<_>>()
            })?;
        let info = self.register_module_info(&key, &canonical);
        let id = info.id;
        graph.paths.insert(key.clone(), id);

        let base_dir = canonical
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let imports = self.resolve_graph_imports(&program, &base_dir, files, graph, loading)?;

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
        &mut self,
        program: &Program,
        base_dir: &Path,
        files: &mut FileTable,
        graph: &mut ResolvedModuleGraph,
        loading: &mut HashSet<PathBuf>,
    ) -> Result<Vec<ImportBinding>, Vec<CompileError>> {
        let imports: Vec<(Vec<String>, Vec<String>, Span)> = program
            .declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Import {
                    names,
                    module_path,
                    span,
                } => Some((names.clone(), module_path.clone(), *span)),
                _ => None,
            })
            .collect();

        let mut bindings = Vec::new();

        for (names, module_path, span) in imports {
            if module_path.is_empty() {
                return Err(vec![CompileError::Resolution(
                    "empty import path".into(),
                    Some(span),
                )]);
            }

            self.reject_duplicate_requested_imports(&names, &module_path, span)?;

            let first = &module_path[0];
            let root_prefix = parse_module_root_prefix(first);
            if first == "@builtin"
                || (root_prefix.is_some_and(|prefix| prefix.is_std()) && module_path.len() == 1)
            {
                continue;
            }

            let file_path = if root_prefix.is_some_and(|prefix| prefix.is_std()) {
                self.resolve_stdlib_file_path(&module_path[1..])?
                    .ok_or_else(|| {
                        vec![CompileError::Resolution(
                            format!("cannot find stdlib module '{}'", module_path.join(".")),
                            Some(span),
                        )]
                    })?
            } else {
                self.local_import_file_path(base_dir, &module_path, span)?
            };

            let dep_id = self.load_graph_module(&file_path, files, graph, loading)?;
            let dep_module = graph
                .module(dep_id)
                .expect("graph module exists immediately after load");
            self.collect_import_bindings(
                dep_module,
                &names,
                &module_path.join("."),
                span,
                &mut bindings,
            )?;
        }

        Ok(bindings)
    }

    fn collect_import_bindings(
        &self,
        dep_module: &ResolvedModule,
        names: &[String],
        module_name: &str,
        import_span: Span,
        bindings: &mut Vec<ImportBinding>,
    ) -> Result<(), Vec<CompileError>> {
        for name in names {
            match exported_module_symbol(&dep_module.symbols, name) {
                ExportedModuleSymbol::Public => {
                    bindings.push(ImportBinding {
                        local_name: name.clone(),
                        source_module: dep_module.info.id,
                        source_symbol: name.clone(),
                        span: import_span,
                    });
                }
                ExportedModuleSymbol::Private => {
                    return Err(vec![CompileError::Resolution(
                        format!(
                            "symbol '{}' in module '{}' is not exported",
                            name, module_name
                        ),
                        Some(import_span),
                    )]);
                }
                ExportedModuleSymbol::Missing => {
                    return Err(vec![CompileError::Resolution(
                        format!("module '{}' does not export '{}'", module_name, name),
                        Some(import_span),
                    )]);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportedModuleSymbol {
    Public,
    Private,
    Missing,
}

fn exported_module_symbol(symbols: &SymbolTable, name: &str) -> ExportedModuleSymbol {
    let mut found_private = false;

    for namespace in [Namespace::Value, Namespace::Type, Namespace::Behavior] {
        let Some(symbol) = symbols.lookup(namespace, name) else {
            continue;
        };
        if symbol.is_public {
            return ExportedModuleSymbol::Public;
        }
        found_private = true;
    }

    if found_private {
        ExportedModuleSymbol::Private
    } else {
        ExportedModuleSymbol::Missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer, parser};

    fn resolve_symbols(source: &str) -> SymbolTable {
        let tokens = lexer::tokenize(source, 0).expect("lex source");
        let program = parser::parse(tokens, 0).expect("parse source");
        Resolver::new()
            .resolve_program(&program)
            .expect("resolve source")
    }

    #[test]
    fn exported_module_symbol_reads_resolver_public_visibility() {
        let symbols = resolve_symbols(
            r#"
hidden = () i32 { 1 }
pub Model: { value: i32 }
pub Json<T>: behavior {
    encode: (Self) T
}
"#,
        );

        assert_eq!(
            exported_module_symbol(&symbols, "hidden"),
            ExportedModuleSymbol::Private
        );
        assert_eq!(
            exported_module_symbol(&symbols, "Model"),
            ExportedModuleSymbol::Public
        );
        assert_eq!(
            exported_module_symbol(&symbols, "Json"),
            ExportedModuleSymbol::Public
        );
        assert_eq!(
            exported_module_symbol(&symbols, "Missing"),
            ExportedModuleSymbol::Missing
        );
    }

    #[test]
    fn exported_module_symbol_accepts_public_symbol_over_private_symbol_in_other_namespace() {
        let symbols = resolve_symbols(
            r#"
Name = () i32 { 1 }
pub Name: { value: i32 }
"#,
        );

        assert_eq!(
            exported_module_symbol(&symbols, "Name"),
            ExportedModuleSymbol::Public
        );
    }
}
