use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::{Declaration, Program};
use crate::error::{CompileError, FileTable, Span};

use super::root_prefix::parse_module_root_prefix;
use super::ModuleSystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatedStdlibModule {
    ActorFramework,
    AllocatorFramework,
    AsyncRuntime,
    SyncRuntime,
}

impl GatedStdlibModule {
    const CONCURRENCY_SEGMENT: &'static str = "concurrency";
    const ACTOR_SEGMENT: &'static str = "actor";
    const ASYNC_SEGMENT: &'static str = "async";
    const SYNC_SEGMENT: &'static str = "sync";
    const MEMORY_SEGMENT: &'static str = "memory";
    const ALLOCATOR_SEGMENT: &'static str = "allocator";

    fn from_sub_path(sub_path: &[String]) -> Option<Self> {
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::CONCURRENCY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::ACTOR_SEGMENT)
        {
            return Some(Self::ActorFramework);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::CONCURRENCY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::ASYNC_SEGMENT)
        {
            return Some(Self::AsyncRuntime);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::CONCURRENCY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::SYNC_SEGMENT)
        {
            return Some(Self::SyncRuntime);
        }
        if sub_path
            .first()
            .is_some_and(|segment| segment == Self::MEMORY_SEGMENT)
            && sub_path
                .get(1)
                .is_some_and(|segment| segment == Self::ALLOCATOR_SEGMENT)
        {
            return Some(Self::AllocatorFramework);
        }
        None
    }

    fn gate_message(self) -> &'static str {
        match self {
            Self::ActorFramework => {
                "std actor framework modules are gated until mailbox, scheduling, supervisor, and allocator semantics are implemented"
            }
            Self::AllocatorFramework => {
                "std allocator modules are gated until allocator ownership and effect semantics are implemented"
            }
            Self::AsyncRuntime => {
                "std async runtime modules are gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::SyncRuntime => {
                "std sync runtime modules are gated until channel, mailbox, and blocking semantics are implemented"
            }
        }
    }
}

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
                if module_path.len() == 1 {
                    continue;
                }

                self.reject_gated_stdlib_module(&module_path[1..], Some(span))?;

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

    fn collect_imported_declarations(
        &self,
        dep_program: &Program,
        names: &[String],
        module_name: &str,
        import_span: Span,
        imported_decls: &mut Vec<Declaration>,
    ) -> Result<(), Vec<CompileError>> {
        for name in names {
            let mut found_private = false;
            let mut found_public = false;

            for decl in &dep_program.declarations {
                if decl.name() == Some(name.as_str()) {
                    if decl.is_public() {
                        found_public = true;
                        imported_decls.push(decl.clone());
                    } else {
                        found_private = true;
                    }
                }

                if let Declaration::Method {
                    type_name, public, ..
                } = decl
                {
                    if type_name == name && *public {
                        imported_decls.push(decl.clone());
                    }
                }
            }

            if !found_public {
                if found_private {
                    return Err(vec![CompileError::Resolution(
                        format!(
                            "symbol '{}' in module '{}' is not exported",
                            name, module_name
                        ),
                        Some(import_span),
                    )]);
                }
                return Err(vec![CompileError::Resolution(
                    format!("module '{}' does not export '{}'", module_name, name),
                    Some(import_span),
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

    pub(super) fn resolve_stdlib_file_path(
        &self,
        sub_path: &[String],
    ) -> Result<Option<PathBuf>, Vec<CompileError>> {
        let root = match &self.stdlib_root {
            Some(r) => r.clone(),
            None => {
                return Err(vec![CompileError::Resolution(
                    "stdlib not found".into(),
                    None,
                )]);
            }
        };

        if sub_path.is_empty() {
            return Ok(None);
        }

        let mut dir = root;
        for seg in sub_path {
            dir.push(seg);
        }

        let file_path = dir.with_extension("zen");
        if file_path.exists() {
            return Ok(Some(file_path));
        }

        let mod_path = dir.join("mod.zen");
        if mod_path.exists() {
            return Ok(Some(mod_path));
        }

        let parent_file = dir.with_extension("zen");
        if parent_file.exists() {
            return Ok(Some(parent_file));
        }

        Ok(None)
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
}

/// Find the stdlib root by looking for a `stdlib/` directory.
pub(super) fn find_stdlib_root() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let stdlib = dir.join("stdlib");
            if stdlib.is_dir() {
                return Some(stdlib);
            }
            if let Some(parent) = dir.parent() {
                if let Some(grandparent) = parent.parent() {
                    let stdlib = grandparent.join("stdlib");
                    if stdlib.is_dir() {
                        return Some(stdlib);
                    }
                }
            }
        }
    }

    let cwd = std::env::current_dir().ok()?;
    let stdlib = cwd.join("stdlib");
    if stdlib.is_dir() {
        return Some(stdlib);
    }

    None
}
