use super::*;

#[test]
fn unknown_module_function_is_an_error() {
    let errors = typecheck_errors(
        r#"
{ io } = std

main = () i32 {
    io.missing("bad")
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &errors,
        "E3023",
        "undefined module function `io.missing`",
        "unknown module function",
    );
    assert_no_diagnostic_message(&errors, "assuming void", "unknown module function");
}
