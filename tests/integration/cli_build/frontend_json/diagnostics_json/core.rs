use super::emit_diagnostics_json;

#[test]
fn emit_json_diagnostics_command_outputs_machine_readable_errors() {
    let json = emit_diagnostics_json(
        r#"
main = () i32 {
    true
}
"#,
        "bad_type.zen",
        "ordinary type error",
    );

    assert_eq!(json["format"], "zen.diagnostics.v0");
    assert_eq!(json["semantic_status"], "diagnostic");
    assert_eq!(json["files"].as_array().expect("files array").len(), 1);

    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(diagnostic["code"], "E3030");
    assert!(
        diagnostic["suggested_fixes"]
            .as_array()
            .expect("suggested_fixes array")
            .is_empty(),
        "ordinary type diagnostics should not carry return keyword fixes: {diagnostic}"
    );
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "unexpected diagnostic payload: {diagnostic}"
    );

    let span = &diagnostic["span"];
    assert!(span["path"]
        .as_str()
        .expect("span path")
        .ends_with("bad_type.zen"));
    assert_eq!(span["line"], 3);
    assert_eq!(span["column"], 5);
}
