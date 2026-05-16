use std::process::Command;

#[test]
fn check_command_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_resolver_ref.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    missing_local
}
"#,
    )
    .expect("write test file");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown value symbol 'missing_local'"),
        "expected resolver diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_reports_imported_module_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    missing_dep_local
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("unknown value symbol 'missing_dep_local'"),
        "expected imported module resolver diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_deduplicates_typechecker_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub Point: {
    x: i32
}
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Json, Point } = traits

Point.requires(Json<str>)

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostic = "type `Point` does not implement required behavior `Json_str`";
    assert!(
        stderr.contains(diagnostic),
        "expected missing behavior diagnostic, stderr={stderr}"
    );
    assert_eq!(
        stderr.matches(diagnostic).count(),
        1,
        "expected missing behavior diagnostic once, stderr={stderr}"
    );
}

#[test]
fn emit_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit");

    assert!(
        !output.status.success(),
        "zen emit unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
