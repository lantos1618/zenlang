use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::{CompileError, Span};

use super::ModuleSystem;

impl ModuleSystem {
    pub(in crate::module_system) fn local_import_file_path(
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

    pub(in crate::module_system) fn reject_duplicate_requested_imports(
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
}
