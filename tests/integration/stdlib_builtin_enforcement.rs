// Enforcement test: no raw `@builtin.*` calls outside `stdlib/compiler.zen`.
//
// The stdlib rule is: `@builtin.*` intrinsics may only appear in
// `stdlib/compiler.zen`. Every other stdlib module calls typed wrappers
// imported from `std.compiler`. This test walks the entire `stdlib/` tree and
// asserts that rule, with an allowlist for documented exceptions.
//
// Documented exceptions:
//   stdlib/build.zen                         — @builtin.build is the comptime DSL import
//   stdlib/io/io.zen                         — namespace-injection module, must be self-contained
//   stdlib/concurrency/coroutine.zen         — global-var initializers require @builtin.null_ptr() (C constant)
//   stdlib/concurrency/actor/actor.zen       — void bare-statement stores (Zen parser footgun)
//   stdlib/concurrency/sync/channel.zen      — void bare-statement stores (Zen parser footgun)
//   stdlib/concurrency/actor/async_actor.zen — global-var initializers (same as coroutine.zen)
//   stdlib/concurrency/actor/coactor.zen     — global-var initializers (same as coroutine.zen)
//   stdlib/memory/async_pool.zen             — global-var initializers (same as coroutine.zen)

use std::path::Path;

fn stdlib_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("stdlib")
}

/// Paths that are fully exempt from the `@builtin.` ban.
fn fully_exempt(relative: &str) -> bool {
    matches!(relative, "compiler.zen" | "build.zen" | "io/io.zen")
}

/// Paths where @builtin is allowed only for global-var initializer uses
/// (`@builtin.null_ptr()` at module level) or void-as-statement store calls
/// (`@builtin.store(...)`). We still check that NO OTHER @builtin intrinsics
/// appear in these files.
fn partially_exempt(relative: &str) -> bool {
    matches!(
        relative,
        "concurrency/coroutine.zen"
            | "concurrency/actor/actor.zen"
            | "concurrency/sync/channel.zen"
            | "concurrency/actor/async_actor.zen"
            | "concurrency/actor/coactor.zen"
            | "memory/async_pool.zen"
    )
}

/// For partially-exempt files, the allowed @builtin patterns.
fn allowed_in_partial(line: &str) -> bool {
    // Global-var initializer: @builtin.null_ptr()
    line.contains("@builtin.null_ptr()") ||
    // Void bare-statement store (parser footgun workaround)
    line.contains("@builtin.store(")
}

#[test]
fn stdlib_no_raw_builtin_outside_compiler_zen() {
    let root = stdlib_root();
    let mut violations: Vec<String> = Vec::new();

    collect_violations(&root, &root, &mut violations);

    if !violations.is_empty() {
        let msg = format!(
            "{} @builtin.* violation(s) found in stdlib:\n{}",
            violations.len(),
            violations.join("\n")
        );
        panic!("{}", msg);
    }
}

fn collect_violations(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_violations(root, &path, out);
        } else if path.extension().is_some_and(|e| e == "zen") {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            check_file(&path, &relative, out);
        }
    }
}

fn check_file(path: &Path, relative: &str, out: &mut Vec<String>) {
    if fully_exempt(relative) {
        return;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            out.push(format!("  cannot read {relative}: {e}"));
            return;
        }
    };

    let partial = partially_exempt(relative);
    for (lineno, line) in content.lines().enumerate() {
        // Skip comment lines
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if !trimmed.contains("@builtin.") {
            continue;
        }
        if partial && allowed_in_partial(trimmed) {
            continue;
        }
        out.push(format!("  {relative}:{}: {}", lineno + 1, trimmed));
    }
}
