use std::process::Command;

#[test]
fn emit_json_diagnostics_command_outputs_machine_readable_errors() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_type.zen");
    let source = r#"
main = () i32 {
    true
}
"#;
    std::fs::write(&zen_path, source).expect("write bad source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on errors: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
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

#[test]
fn emit_json_diagnostics_includes_structured_return_keyword_fix() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("return_keyword.zen");
    let source = r#"
main = () i32 {
    return 1
}
"#;
    let return_start = source.find("return").expect("source contains return") as u32;
    let return_end = return_start + "return".len() as u32;
    std::fs::write(&zen_path, source).expect("write removed return source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on removed return syntax: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["code"], "E2000");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("return keyword has been removed"),
        "unexpected diagnostic payload: {diagnostic}"
    );

    let suggestions = diagnostic["suggested_fixes"]
        .as_array()
        .expect("diagnostic should carry structured suggested fixes");
    assert_eq!(
        suggestions.len(),
        1,
        "unexpected suggestions: {suggestions:?}"
    );

    let fix = &suggestions[0];
    assert_eq!(fix["kind"], "replace_removed_return_with_final_expression");
    assert_eq!(
        fix["title"],
        "Remove `return` and use the value as the final expression"
    );

    let edit = &fix["edits"][0];
    assert_eq!(
        fix["edits"].as_array().expect("fix edits array").len(),
        1,
        "return fix should carry exactly one text edit: {fix}"
    );
    assert!(edit["span"]["path"]
        .as_str()
        .expect("edit span path")
        .ends_with("return_keyword.zen"));
    assert_eq!(edit["span"]["start"], return_start);
    assert_eq!(edit["span"]["end"], return_end);
    assert_eq!(edit["span"]["line"], 3);
    assert_eq!(edit["span"]["column"], 5);
    assert_eq!(edit["replacement"], "");
}

#[test]
fn emit_json_diagnostics_includes_structured_infix_as_cast_fix() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("as_cast.zen");
    let source = r#"
main = (x: i32) i64 {
    x + 1 as i64
}
"#;
    let expression = "x + 1 as i64";
    let expression_start = source.find(expression).expect("source contains as-cast") as u32;
    let expression_end = expression_start + expression.len() as u32;
    std::fs::write(&zen_path, source).expect("write removed as-cast source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on removed as-cast syntax: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["code"], "E2000");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("`as` cast syntax has been removed"),
        "unexpected diagnostic payload: {diagnostic}"
    );

    let suggestions = diagnostic["suggested_fixes"]
        .as_array()
        .expect("diagnostic should carry structured suggested fixes");
    assert_eq!(
        suggestions.len(),
        1,
        "unexpected suggestions: {suggestions:?}"
    );

    let fix = &suggestions[0];
    assert_eq!(fix["kind"], "replace_infix_as_cast_with_prefix_cast");
    assert_eq!(
        fix["title"],
        "Rewrite infix `as` cast to prefix `cast(value, Type)`"
    );

    let edit = &fix["edits"][0];
    assert_eq!(
        fix["edits"].as_array().expect("fix edits array").len(),
        1,
        "as-cast fix should carry exactly one text edit: {fix}"
    );
    assert!(edit["span"]["path"]
        .as_str()
        .expect("edit span path")
        .ends_with("as_cast.zen"));
    assert_eq!(edit["span"]["start"], expression_start);
    assert_eq!(edit["span"]["end"], expression_end);
    assert_eq!(edit["span"]["line"], 3);
    assert_eq!(edit["span"]["column"], 5);
    assert_eq!(edit["replacement"], "cast(value, Type)");
}

#[test]
fn emit_json_diagnostics_spans_full_gated_behavior_derive_association() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("derive_gate.zen");
    let source = r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.derive(Json)
"#;
    let association = "Point.derive(Json)";
    let association_start = source
        .find(association)
        .expect("source contains derive association") as u32;
    let association_end = association_start + association.len() as u32;
    std::fs::write(&zen_path, source).expect("write gated derive source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on gated derive association: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["code"], "E2000");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("generated behavior association `Type.derive(...)` is gated"),
        "unexpected diagnostic payload: {diagnostic}"
    );
    assert_eq!(
        diagnostic["notes"][0],
        "Use an explicit `Type.implements(Behavior) { ... }` block until generated fallback derives are implemented"
    );
    assert_eq!(diagnostic["context"][0]["kind"], "feature_gate");
    assert_eq!(
        diagnostic["context"][0]["message"],
        "reserved generated/fallback behavior association"
    );
    assert!(diagnostic["span"]["path"]
        .as_str()
        .expect("diagnostic span path")
        .ends_with("derive_gate.zen"));
    assert_eq!(diagnostic["span"]["start"], association_start);
    assert_eq!(diagnostic["span"]["end"], association_end);
    assert_eq!(diagnostic["span"]["line"], 8);
    assert_eq!(diagnostic["span"]["column"], 1);
}

#[test]
fn emit_json_diagnostics_spans_full_gated_generic_association_target() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_association_gate.zen");
    let source = r#"
Box<T>: {
    value: T
}

Json<T>: behavior {
    to_json: (T) StaticString
}

Box<T>.derive(Json<T>)
"#;
    let association = "Box<T>.derive(Json<T>)";
    let association_start = source
        .find(association)
        .expect("source contains generic association") as u32;
    let association_end = association_start + association.len() as u32;
    std::fs::write(&zen_path, source).expect("write gated generic association source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on gated generic association: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["code"], "E2000");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("generic association target `Type<T>.derive` is gated"),
        "unexpected diagnostic payload: {diagnostic}"
    );
    assert_eq!(
        diagnostic["notes"][0],
        "Use a non-generic explicit behavior association until generic behavior target templates are implemented"
    );
    assert_eq!(diagnostic["context"][0]["kind"], "feature_gate");
    assert_eq!(
        diagnostic["context"][0]["message"],
        "reserved generic behavior association target"
    );
    assert!(diagnostic["span"]["path"]
        .as_str()
        .expect("diagnostic span path")
        .ends_with("generic_association_gate.zen"));
    assert_eq!(diagnostic["span"]["start"], association_start);
    assert_eq!(diagnostic["span"]["end"], association_end);
    assert_eq!(diagnostic["span"]["line"], 10);
    assert_eq!(diagnostic["span"]["column"], 1);
}
