mod check_command;
mod dedup;
mod emit_command;
mod usage;

use super::support::{assert_zen_failure_contains, write_file};

fn write_imported_module_type_error_fixture(tmp: &tempfile::TempDir) -> &'static str {
    write_file(
        tmp,
        "math.zen",
        r#"
add = (a: i32, b: i32) i32 {
    a + b
}

broken = () i32 {
    true
}
@export({ add, broken })
"#,
    );

    write_file(
        tmp,
        "main.zen",
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    );
    "main.zen"
}

fn assert_fails_with(output: &std::process::Output, args: &[&str], expected: &str) {
    assert_zen_failure_contains(args, output, expected);
}
