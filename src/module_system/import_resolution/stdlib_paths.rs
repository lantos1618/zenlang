use std::path::PathBuf;

use crate::error::CompileError;
use crate::module_system::ModuleSystem;

impl ModuleSystem {
    pub(in crate::module_system) fn resolve_stdlib_file_path(
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
}

/// Find the stdlib root by looking for a `stdlib/` directory.
pub(in crate::module_system) fn find_stdlib_root() -> Option<PathBuf> {
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
