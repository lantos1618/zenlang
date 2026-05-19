use super::emit_diagnostics_json;

#[test]
fn emit_json_diagnostics_spans_full_gated_behavior_derive_association() {
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
    let json = emit_diagnostics_json(source, "derive_gate.zen", "gated derive association");

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
    let json = emit_diagnostics_json(
        source,
        "generic_association_gate.zen",
        "gated generic association",
    );

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
