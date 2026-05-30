use std::path::{Path, PathBuf};
use std::process;

use zen::codegen::c;

pub(super) fn compile_file_to_c_source(path: &Path) -> String {
    let path_str = path.to_str().unwrap_or_else(|| {
        eprintln!("error: non-utf8 source path: {}", path.display());
        process::exit(1);
    });
    let typed = super::graph_frontend(path_str);
    typed_program_to_c_source(&typed)
}

pub(super) fn typed_program_to_c_source(typed: &zen::ast::typed::TypedProgram) -> String {
    c::generate(typed)
}

pub(super) fn compile_file_to_binary(
    path_str: &str,
    output_dir: Option<&Path>,
    output_name: Option<&str>,
    link_libs: &[String],
    headers: &[String],
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
    let mut command = process::Command::new(&cc);
    command.arg(&c_path).arg("-o").arg(&bin_path).arg("-lm");
    // System libraries from build.zen's `link:` field (Zig's linkSystemLibrary
    // analog): resolve include/lib/rpath flags per library via pkg-config.
    for lib in link_libs {
        command.args(link_flags_for_library(lib));
    }
    // C headers from `headers:`: force-include each so the emitted @extern
    // prototypes are checked against the real declarations (ABI verification —
    // a mismatch is a C "conflicting types" error).
    for header in headers {
        command.arg("-include").arg(header);
    }
    // Escape hatch: extra cc flags (include dirs, `-l`, `-L`, `-include
    // <header>`) appended verbatim, whitespace-separated. `link:` is preferred.
    if let Ok(extra) = std::env::var("ZEN_CC_EXTRA") {
        command.args(extra.split_whitespace());
    }
    let status = command.status();

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

/// Resolve the cc flags needed to link one system library named in build.zen's
/// `link:` field. An entry may carry a minimum-version constraint
/// (`"SDL3 >= 3.2"`), which is checked against pkg-config up front with a clear
/// error. Prefers pkg-config (`--cflags --libs`, plus an rpath to the library
/// directory so the binary runs without LD_LIBRARY_PATH); falls back to a bare
/// `-l<name>` when pkg-config has no entry for it.
fn link_flags_for_library(entry: &str) -> Vec<String> {
    // Split an optional `>= <version>` constraint off the library name.
    let (lib, min_version) = match entry.split_once(">=") {
        Some((name, ver)) => (name.trim(), Some(ver.trim())),
        None => (entry.trim(), None),
    };

    let pkg = |args: &[&str]| -> Option<String> {
        let out = process::Command::new("pkg-config").args(args).output().ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };

    if let Some(min) = min_version {
        let ok = process::Command::new("pkg-config")
            .arg(format!("--atleast-version={min}"))
            .arg(lib)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            let found = pkg(&["--modversion", lib]).unwrap_or_else(|| "not installed".into());
            eprintln!("  link error: `{lib}` needs version >= {min}, found {found}");
            process::exit(1);
        }
    }

    let mut flags = Vec::new();
    // `--exists` succeeds (empty stdout) only when pkg-config knows the library.
    if pkg(&["--exists", lib]).is_some() {
        if let Some(cflags) = pkg(&["--cflags", lib]) {
            flags.extend(cflags.split_whitespace().map(str::to_string));
        }
        if let Some(libs) = pkg(&["--libs", lib]) {
            flags.extend(libs.split_whitespace().map(str::to_string));
        }
        if let Some(libdir) = pkg(&["--variable=libdir", lib]) {
            if !libdir.is_empty() {
                flags.push(format!("-Wl,-rpath,{libdir}"));
            }
        }
    }
    if flags.is_empty() {
        // No pkg-config entry — fall back to a plain link flag.
        flags.push(format!("-l{lib}"));
    }
    flags
}
