use std::path::{Path, PathBuf};
use std::process;

use zen::codegen::c::CBackend;
use zen::codegen::Backend;

pub(super) fn compile_file_to_c_source(path: &Path) -> String {
    let path_str = path.to_str().unwrap_or_else(|| {
        eprintln!("error: non-utf8 source path: {}", path.display());
        process::exit(1);
    });
    let typed = super::graph_frontend(path_str);
    typed_program_to_c_source(&typed)
}

pub(super) fn typed_program_to_c_source(typed: &zen::ast::typed::TypedProgram) -> String {
    let backend = CBackend;
    match backend.generate(typed) {
        Ok(c_source) => c_source,
        Err(e) => {
            eprintln!("codegen error: {}", e);
            process::exit(1);
        }
    }
}

pub(super) fn compile_file_to_binary(
    path_str: &str,
    output_dir: Option<&Path>,
    output_name: Option<&str>,
) -> PathBuf {
    let c_source = compile_file_to_c_source(Path::new(path_str));

    let stem = Path::new(path_str)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");
    let output_stem = output_name.unwrap_or(stem);
    let c_path = output_dir
        .map(|dir| dir.join(format!("{output_stem}.c")))
        .unwrap_or_else(|| Path::new(&format!("{output_stem}.c")).to_path_buf());
    let bin_path = output_dir
        .map(|dir| dir.join(output_stem))
        .unwrap_or_else(|| Path::new(output_stem).to_path_buf());

    if let Err(e) = std::fs::write(&c_path, &c_source) {
        eprintln!("error writing {}: {}", c_path.display(), e);
        process::exit(1);
    }
    println!("  emitted {}", c_path.display());

    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let status = process::Command::new(&cc)
        .arg(&c_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("-lm")
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("  compiled → {}", bin_path.display());
            bin_path
        }
        Ok(s) => {
            eprintln!("  {} exited with {}", cc, s);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("  failed to run {}: {}", cc, e);
            eprintln!("  (C source saved to {})", c_path.display());
            process::exit(1);
        }
    }
}
