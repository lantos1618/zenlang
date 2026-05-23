use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::{Declaration, Program};
use crate::error::{CompileError, FileTable, Span};

use super::root_prefix::parse_module_root_prefix;
use super::ModuleSystem;

mod imported_declarations;
mod stdlib_gates;
mod stdlib_paths;

use stdlib_gates::GatedStdlibModule;
pub(super) use stdlib_paths::find_stdlib_root;

impl ModuleSystem {
    /// Walk Import declarations in `program` and load their dependencies.
    ///
    /// Import routing:
    /// - `{ io } = std` or `{ io } = std.io` - stdlib, skip (handled by codegen)
    /// - `{ cast } = @builtin` - intrinsic, skip
    /// - `{ add } = math` - resolve `math.zen` relative to `base_dir`
    /// - `{ Foo } = utils.math` - resolve `utils/math.zen` relative to `base_dir`
    pub(super) fn resolve_imports(
        &mut self,
        program: &mut Program,
        base_dir: &Path,
        files: &mut FileTable,
    ) -> Result<(), Vec<CompileError>> {
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

        let mut imported_decls: Vec<Declaration> = Vec::new();

        for (names, module_path, span) in imports {
            if module_path.is_empty() {
                return Err(vec![CompileError::Resolution(
                    "empty import path".into(),
                    Some(span),
                )]);
            }

            let first = &module_path[0];

            if parse_module_root_prefix(first).is_some_and(|prefix| prefix.is_std()) {
                self.reject_gated_stdlib_import(&names, &module_path[1..], Some(span))?;

                if module_path.len() == 1 {
                    continue;
                }

                let Some(file_path) = self.resolve_stdlib_file_path(&module_path[1..])? else {
                    return Err(vec![CompileError::Resolution(
                        format!("cannot find stdlib module '{}'", module_path.join(".")),
                        Some(span),
                    )]);
                };

                let dep_program = self.load_with_imports(&file_path, files)?;
                self.collect_imported_declarations(
                    &dep_program,
                    &names,
                    &module_path.join("."),
                    span,
                    &mut imported_decls,
                )?;
                continue;
            }

            if first == "@builtin" {
                continue;
            }

            let file_path = self.local_import_file_path(base_dir, &module_path, span)?;
            self.reject_duplicate_requested_imports(&names, &module_path, span)?;

            let dep_program = self.load_with_imports(&file_path, files)?;
            self.collect_imported_declarations(
                &dep_program,
                &names,
                &module_path.join("."),
                span,
                &mut imported_decls,
            )?;
        }

        program.declarations.extend(imported_decls);
        Ok(())
    }

    pub(super) fn local_import_file_path(
        &self,
        base_dir: &Path,
        module_path: &[String],
        span: Span,
    ) -> Result<PathBuf, Vec<CompileError>> {
        let rel_path: PathBuf = module_path.iter().collect();
        let mut file_path = base_dir.join(&rel_path);
        if file_path.extension().is_none() {
            file_path.set_extension("zen");
        }

        if !file_path.exists() {
            return Err(vec![CompileError::Resolution(
                format!(
                    "cannot find imported module '{}' (looked for {})",
                    module_path.join("."),
                    file_path.display()
                ),
                Some(span),
            )]);
        }

        Ok(file_path)
    }

    pub(super) fn reject_duplicate_requested_imports(
        &self,
        names: &[String],
        module_path: &[String],
        span: Span,
    ) -> Result<(), Vec<CompileError>> {
        let mut requested_names = HashSet::new();
        for name in names {
            if !requested_names.insert(name.as_str()) {
                return Err(vec![CompileError::Resolution(
                    format!(
                        "duplicate import '{}' from module '{}'",
                        name,
                        module_path.join(".")
                    ),
                    Some(span),
                )]);
            }
        }
        Ok(())
    }

    /// Resolve an import module path to a filesystem path and load it.
    /// e.g. ["std"] or ["std", "io"] or ["@std", "io"]
    pub fn resolve_import(
        &mut self,
        module_path: &[String],
        files: &mut FileTable,
    ) -> Result<Program, Vec<CompileError>> {
        if module_path.is_empty() {
            return Err(vec![CompileError::Resolution(
                "empty module path".into(),
                None,
            )]);
        }

        let first = &module_path[0];

        if parse_module_root_prefix(first).is_some_and(|prefix| prefix.is_std()) {
            return self.resolve_stdlib_import(&module_path[1..], files);
        }

        if first == "@builtin" {
            return Ok(Program {
                declarations: Vec::new(),
                file_id: 0,
            });
        }

        Err(vec![CompileError::Resolution(
            format!("unknown module: {}", module_path.join(".")),
            None,
        )])
    }

    fn resolve_stdlib_import(
        &mut self,
        sub_path: &[String],
        files: &mut FileTable,
    ) -> Result<Program, Vec<CompileError>> {
        if sub_path.is_empty() {
            return Ok(Program {
                declarations: Vec::new(),
                file_id: 0,
            });
        }

        self.reject_gated_stdlib_module(sub_path, None)?;

        if let Some(file_path) = self.resolve_stdlib_file_path(sub_path)? {
            return self.load_file(&file_path, files);
        }

        Ok(Program {
            declarations: Vec::new(),
            file_id: 0,
        })
    }

    pub(super) fn reject_gated_stdlib_module(
        &self,
        sub_path: &[String],
        span: Option<Span>,
    ) -> Result<(), Vec<CompileError>> {
        if let Some(gated) = GatedStdlibModule::from_sub_path(sub_path) {
            return Err(vec![CompileError::Resolution(
                gated.gate_message().into(),
                span,
            )]);
        }
        Ok(())
    }

    pub(super) fn reject_gated_stdlib_import(
        &self,
        names: &[String],
        sub_path: &[String],
        span: Option<Span>,
    ) -> Result<(), Vec<CompileError>> {
        if let Some(gated) = GatedStdlibModule::from_import(names, sub_path) {
            return Err(vec![CompileError::Resolution(
                gated.gate_message().into(),
                span,
            )]);
        }
        Ok(())
    }
}
