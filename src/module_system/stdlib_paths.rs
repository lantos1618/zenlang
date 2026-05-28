use std::path::PathBuf;

use crate::error::CompileError;

pub(in crate::module_system) fn resolve_stdlib_file_path(
    sub_path: &[String],
) -> Result<Option<PathBuf>, Vec<CompileError>> {
    let Some(root) = find_stdlib_root() else {
        return Err(vec![CompileError::Resolution(
            "stdlib not found".into(),
            None,
        )]);
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

    Ok(None)
}

pub(in crate::module_system) fn find_stdlib_root() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from));
    [
        exe_dir.as_ref().map(|dir| dir.join("stdlib")),
        exe_dir
            .as_ref()
            .and_then(|dir| dir.parent()?.parent().map(|root| root.join("stdlib"))),
        std::env::current_dir().ok().map(|dir| dir.join("stdlib")),
    ]
    .into_iter()
    .flatten()
    .find(|stdlib| stdlib.is_dir())
}
