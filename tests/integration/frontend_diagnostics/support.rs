use std::path::{Path, PathBuf};

use crate::compile_to_c;

pub(crate) fn write_tmp_module(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    path
}

pub(crate) fn compile_to_c_panic_message(zen_path: &Path) -> String {
    let panic = std::panic::catch_unwind(|| compile_to_c(zen_path))
        .expect_err("compile_to_c should reject frontend diagnostics");
    panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>")
        .to_string()
}
