use super::super::support::{run_zen_in, write_file};

#[test]
fn check_command_deduplicates_typechecker_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "traits.zen",
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub Point: {
    x: i32
}
"#,
    );

    write_file(
        &tmp,
        "main.zen",
        r#"
{ Json, Point } = traits

Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
    );

    let output = run_zen_in(&tmp, &["check", "main.zen"]);

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostic = "type `Point` does not implement required behavior `Json_StaticString`";
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
