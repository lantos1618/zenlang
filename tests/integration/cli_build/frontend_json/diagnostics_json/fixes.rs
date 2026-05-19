use super::emit_diagnostics_json;

#[test]
fn emit_json_diagnostics_includes_structured_return_keyword_fix() {
    let source = r#"
main = () i32 {
    return 1
}
"#;
    let return_start = source.find("return").expect("source contains return") as u32;
    let return_end = return_start + "return".len() as u32;
    let json = emit_diagnostics_json(source, "return_keyword.zen", "removed return syntax");

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
    let source = r#"
main = (x: i32) i64 {
    x + 1 as i64
}
"#;
    let expression = "x + 1 as i64";
    let expression_start = source.find(expression).expect("source contains as-cast") as u32;
    let expression_end = expression_start + expression.len() as u32;
    let json = emit_diagnostics_json(source, "as_cast.zen", "removed as-cast syntax");

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
fn emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix() {
    let source = r#"
main = (flag: bool) i32 {
    flag ?
        | true { 1 }
}
"#;
    let json = emit_diagnostics_json(source, "missing_bool_arm.zen", "missing bool match arm");

    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["code"], "E4006");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("non-exhaustive bool match: missing `false`"),
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
    assert_eq!(fix["kind"], "add_missing_bool_match_arm");
    assert_eq!(fix["title"], "Add missing bool match arm");

    let edit = &fix["edits"][0];
    assert_eq!(
        fix["edits"].as_array().expect("fix edits array").len(),
        1,
        "missing bool arm fix should carry exactly one text edit: {fix}"
    );
    assert!(edit["span"]["path"]
        .as_str()
        .expect("edit span path")
        .ends_with("missing_bool_arm.zen"));
    assert_eq!(edit["span"]["start"], edit["span"]["end"]);
    assert_eq!(edit["replacement"], "\n        | false { <expression> }");
}
