use super::*;

fn tracked_stdlib_zen_files() -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["ls-files", "stdlib"])
        .current_dir(repo_root())
        .output()
        .expect("list tracked stdlib sources");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("git ls-files output is utf-8")
        .lines()
        .filter(|path| path.ends_with(".zen"))
        .map(str::to_owned)
        .collect()
}

#[test]
fn stdlib_builtin_intrinsic_calls_stay_behind_compiler_facade() {
    for path in tracked_stdlib_zen_files() {
        if path == "stdlib/compiler.zen" {
            continue;
        }
        let source = read(&path);
        assert!(
            !source.contains("@builtin."),
            "{path} should import compiler facade helpers instead of calling @builtin directly"
        );
    }
}

#[test]
fn stdlib_syscalls_use_named_syscall_constants() {
    for path in tracked_stdlib_zen_files() {
        let source = read(&path);
        let executable_source = source
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("//") {
                    ""
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut offset = 0;
        while let Some(found) = executable_source[offset..].find("compiler.syscall") {
            let start = offset + found;
            let Some(open_paren) = executable_source[start..].find('(') else {
                break;
            };
            let args_start = start + open_paren + 1;
            let args = executable_source[args_start..].trim_start();
            let first_arg = args.split(',').next().unwrap_or("").trim();
            let line_no = executable_source[..start].lines().count() + 1;
            assert!(
                first_arg.starts_with("SYS_"),
                "{path}:{} should pass a named SYS_* constant as the syscall number, not `{first_arg}`",
                line_no
            );
            offset = args_start;
        }
    }
}

#[test]
fn public_stdlib_text_uses_static_string_name() {
    for path in [
        "README.md",
        "docs/learn_zen_in_y_minutes.md",
        "docs/V1_SPEC.md",
        "stdlib/compiler.zen",
        "stdlib/io/io.zen",
    ] {
        let source = read(path);
        assert!(
            !source.contains("StringLiteral"),
            "{path} should use the language-facing StaticString spelling"
        );
    }
}

#[test]
fn std_facade_does_not_reexport_experimental_testing_sketch() {
    let source = read("stdlib/std.zen");
    assert!(
        !source.contains("@std.testing"),
        "stdlib/std.zen should not promote the experimental testing sketch"
    );
}

#[test]
fn root_tests_directory_does_not_accumulate_legacy_zen_fixtures() {
    let root_tests = repo_root().join("tests");
    let mut legacy = std::fs::read_dir(&root_tests)
        .expect("read tests directory")
        .filter_map(|entry| {
            let path = entry
                .expect("tests directory entry should be readable")
                .path();
            path.is_file()
                .then_some(path)
                .filter(|path| path.extension().is_some_and(|ext| ext == "zen"))
        })
        .collect::<Vec<_>>();
    legacy.sort();

    assert!(
        legacy.is_empty(),
        "root tests/*.zen fixtures are legacy smoke files; keep runtime fixtures under tests/zen:\n{}",
        legacy
            .iter()
            .map(|path| path.strip_prefix(repo_root()).unwrap_or(path).display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn stdlib_async_operation_state_lives_in_one_helper_module() {
    assert!(
        !repo_root()
            .join("stdlib/memory/async_allocator.zen")
            .exists(),
        "async operation state helpers should not masquerade as an allocator module"
    );

    let helpers = read("stdlib/memory/async_helpers.zen");
    for required in ["AsyncOp:", "Promise:", "async_op_new", "Promise.new"] {
        assert!(
            helpers.contains(required),
            "stdlib/memory/async_helpers.zen should own async operation state helper: {required}"
        );
    }
}
