use std::path::{Path, PathBuf};
use std::process::Command;

use zen::codegen::c::CBackend;
use zen::codegen::Backend;
use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

/// Root of the test fixtures.
pub fn test_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/zen")
}

/// Run the full pipeline for a `.zen` file and return generated C.
pub fn compile_to_c(zen_path: &Path) -> String {
    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();
    let graph = module_system
        .load_module_graph(zen_path, &mut files)
        .unwrap_or_else(|errs| {
            panic!(
                "load/parse error in {}:\n  {}",
                zen_path.display(),
                errs.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        });

    let mut checker = TypeChecker::new();
    let typed = checker
        .check_module_graph_entry(&graph)
        .unwrap_or_else(|diags| {
            panic!(
                "typecheck error in {}:\n  {}",
                zen_path.display(),
                diags
                    .iter()
                    .map(|d| d.message.clone())
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        });

    let backend = CBackend;
    backend
        .generate(&typed)
        .unwrap_or_else(|e| panic!("codegen error in {}: {}", zen_path.display(), e))
}

pub fn compile_to_c_with_generated_call_check(zen_path: &Path) -> String {
    let c_source = compile_to_c(zen_path);
    assert_generated_c_calls_resolve_to_definitions(&c_source);
    c_source
}

/// Run the full pipeline for a `.zen` file and return stdout of the compiled binary.
pub fn compile_and_run(zen_path: &Path) -> String {
    let c_source = compile_to_c_with_generated_call_check(zen_path);

    let tmp = tempfile::tempdir().expect("create temp dir");
    let c_path = tmp.path().join("out.c");
    let bin_path = tmp.path().join("out");

    std::fs::write(&c_path, &c_source).expect("write C source");

    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let compile = Command::new(&cc)
        .args([
            c_path.to_str().unwrap(),
            "-o",
            bin_path.to_str().unwrap(),
            "-lm",
        ])
        .output()
        .expect("run cc");

    if !compile.status.success() {
        panic!(
            "cc failed for {}:\n--- C source ---\n{}\n--- stderr ---\n{}",
            zen_path.display(),
            c_source,
            String::from_utf8_lossy(&compile.stderr),
        );
    }

    let run = Command::new(&bin_path).output().expect("run binary");
    assert!(
        run.status.success(),
        "binary exited with {} for {}",
        run.status,
        zen_path.display()
    );

    String::from_utf8(run.stdout).expect("stdout is utf-8")
}

/// Read the expected output for a test name.
pub fn expected_output(name: &str) -> String {
    let path = test_dir()
        .join("expected")
        .join(format!("{}.expected", name));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

/// Run one end-to-end test by name.
pub fn run_test(name: &str) {
    let zen_path = test_dir().join(format!("{}.zen", name));
    let actual = compile_and_run(&zen_path);
    let expected = expected_output(name);
    assert_eq!(
        actual, expected,
        "\n--- test: {} ---\nexpected:\n{}\nactual:\n{}",
        name, expected, actual
    );
}

pub fn assert_c_function_definition(c_source: &str, name: &str) {
    let needle = format!(" {name}(");
    assert!(
        c_source
            .lines()
            .any(|line| line.trim_end().ends_with('{') && line.contains(&needle)),
        "expected generated C definition for `{name}`:\n{c_source}"
    );
}

pub fn assert_c_function_definition_count(c_source: &str, name: &str, expected: usize) {
    let actual = c_function_definitions(c_source)
        .iter()
        .filter(|definition| definition.as_str() == name)
        .count();
    assert_eq!(
        actual, expected,
        "expected {expected} generated C definitions for `{name}`, found {actual}:\n{c_source}"
    );
}

pub fn assert_c_call_resolves_to_definition(c_source: &str, name: &str) {
    assert_c_function_definition(c_source, name);
    assert!(
        has_c_call_outside_signature(c_source, name),
        "expected generated C call to `{name}` outside declarations/definitions:\n{c_source}"
    );
}

pub fn assert_generated_c_calls_resolve_to_definitions(c_source: &str) {
    let undefined = undefined_generated_c_calls(c_source);
    assert!(
        undefined.is_empty(),
        "generated C calls missing emitted definitions: {undefined:?}\n{c_source}"
    );
}

pub fn undefined_generated_c_calls(c_source: &str) -> Vec<String> {
    let definitions = c_function_definitions(c_source);
    let mut undefined = Vec::new();

    for line in c_source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("typedef ")
            || is_any_c_function_signature_line(trimmed)
        {
            continue;
        }

        for call in generated_c_calls_on_line(trimmed) {
            if !definitions.contains(&call) && !undefined.contains(&call) {
                undefined.push(call);
            }
        }
    }

    undefined
}

fn c_function_definitions(c_source: &str) -> Vec<String> {
    c_source
        .lines()
        .filter_map(|line| c_function_definition_name(line.trim()))
        .collect()
}

fn c_function_definition_name(trimmed: &str) -> Option<String> {
    if !trimmed.ends_with('{') {
        return None;
    }

    let paren = trimmed.find('(')?;
    let before = trimmed[..paren].trim_end();
    let name_start = before
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map_or(0, |idx| idx + 1);
    let name = &before[name_start..];

    if is_generated_c_function_name(name) {
        Some(name.to_string())
    } else {
        None
    }
}

fn is_any_c_function_signature_line(trimmed: &str) -> bool {
    if !(trimmed.ends_with(';') || trimmed.ends_with('{')) {
        return false;
    }

    let Some(paren) = trimmed.find('(') else {
        return false;
    };
    let before = &trimmed[..paren];
    let name_start = before
        .trim_end()
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map_or(0, |idx| idx + 1);
    let return_type = before[..name_start].trim();
    let name = before[name_start..].trim();

    !return_type.is_empty()
        && is_generated_c_function_name(name)
        && !before.contains('=')
        && !before.contains("return")
}

fn generated_c_calls_on_line(trimmed: &str) -> Vec<String> {
    let mut calls = Vec::new();
    let bytes = trimmed.as_bytes();
    let mut index = 0;

    while let Some(relative) = trimmed[index..].find('(') {
        let paren = index + relative;
        let mut start = paren;
        while start > 0 {
            let ch = bytes[start - 1] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                start -= 1;
            } else {
                break;
            }
        }

        let name = &trimmed[start..paren];
        if is_generated_c_function_name(name) && !calls.iter().any(|call| call == name) {
            calls.push(name.to_string());
        }

        index = paren + 1;
    }

    calls
}

fn is_generated_c_function_name(name: &str) -> bool {
    name.contains('_')
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub fn has_c_call_outside_signature(c_source: &str, name: &str) -> bool {
    let call = format!("{name}(");
    c_source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains(&call) && !is_c_function_signature_line(trimmed, name)
    })
}

fn is_c_function_signature_line(trimmed: &str, name: &str) -> bool {
    let needle = format!(" {name}(");
    let Some(call_start) = trimmed.find(&needle) else {
        return false;
    };
    let prefix = &trimmed[..call_start];
    !prefix.contains('=')
        && !prefix.contains("return")
        && (trimmed.ends_with(';') || trimmed.ends_with('{'))
}
