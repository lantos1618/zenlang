use std::process::Command;

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
