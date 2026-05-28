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
    assert_message_contains(diagnostic, "return keyword has been removed");
    let (fix, edit) = single_fix_and_edit(diagnostic, "return_keyword.zen");
    assert_eq!(fix["kind"], "replace_removed_return_with_final_expression");
    assert_eq!(
        fix["title"],
        "Remove `return` and use the value as the final expression"
    );
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
    assert_message_contains(diagnostic, "`as` cast syntax has been removed");
    let (fix, edit) = single_fix_and_edit(diagnostic, "as_cast.zen");
    assert_eq!(fix["kind"], "replace_infix_as_cast_with_prefix_cast");
    assert_eq!(
        fix["title"],
        "Rewrite infix `as` cast to prefix `cast(value, Type)`"
    );
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
    assert_message_contains(diagnostic, "non-exhaustive bool match: missing `false`");
    let (fix, edit) = single_fix_and_edit(diagnostic, "missing_bool_arm.zen");
    assert_eq!(fix["kind"], "add_missing_bool_match_arm");
    assert_eq!(fix["title"], "Add missing bool match arm");
    assert_eq!(edit["span"]["start"], edit["span"]["end"]);
    assert_eq!(edit["replacement"], "\n        | false { <expression> }");
}

fn assert_message_contains(diagnostic: &serde_json::Value, expected: &str) {
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains(expected),
        "unexpected diagnostic payload: {diagnostic}"
    );
}

fn single_fix_and_edit<'a>(
    diagnostic: &'a serde_json::Value,
    filename: &str,
) -> (&'a serde_json::Value, &'a serde_json::Value) {
    let suggestions = diagnostic["suggested_fixes"]
        .as_array()
        .expect("diagnostic should carry structured suggested fixes");
    assert_eq!(
        suggestions.len(),
        1,
        "unexpected suggestions: {suggestions:?}"
    );

    let fix = &suggestions[0];
    let edits = fix["edits"].as_array().expect("fix edits array");
    assert_eq!(
        edits.len(),
        1,
        "fix should carry exactly one text edit: {fix}"
    );
    let edit = &edits[0];
    assert!(edit["span"]["path"]
        .as_str()
        .expect("edit span path")
        .ends_with(filename));
    (fix, edit)
}
