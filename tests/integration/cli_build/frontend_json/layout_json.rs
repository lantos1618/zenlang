use super::golden_support::checked_source_json;

#[test]
fn emit_json_layout_outputs_checked_type_layouts() {
    let json = checked_source_json(
        "layout",
        "layout_subject.zen",
        r#"
Point: {
    x: i32,
    label: StaticString
}

main = () i32 { 0 }
"#,
        "program input",
    );

    assert_eq!(json["format"], "zen.layout.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "checked");
    assert_eq!(json["target"]["pointer_size"], 8);
    assert_eq!(json["target"]["usize_size"], 8);

    let layouts = json["layouts"].as_object().expect("layouts object");
    assert_eq!(layouts["i32"]["size"], 4);
    assert_eq!(layouts["i32"]["alignment"], 4);
    assert_eq!(layouts["StaticString"]["size"], 16);
    assert_eq!(layouts["StaticString"]["alignment"], 8);

    let point = &layouts["Point"];
    assert_eq!(point["kind"], "struct");
    assert_eq!(point["size"], 24);
    assert_eq!(point["alignment"], 8);
    let fields = point["fields"].as_array().expect("Point fields");
    assert_eq!(fields[0]["name"], "x");
    assert_eq!(fields[0]["offset"], 0);
    assert_eq!(fields[0]["type"], "i32");
    assert_eq!(fields[1]["name"], "label");
    assert_eq!(fields[1]["offset"], 8);
    assert_eq!(fields[1]["type"], "StaticString");
}

#[test]
fn emit_json_layout_outputs_compound_type_layout_entries() {
    let json = checked_source_json(
        "layout",
        "compound_layout_subject.zen",
        r#"
Handles: {
    ptr: Ptr<i32>,
    raw: RawPtr<i32>,
    slice: Slice<i32>,
    fixed: [i32; 4],
}

Choice:
    Empty,
    WithPayload(Handles)

main = () i32 { 0 }
"#,
        "compound program input",
    );

    assert_eq!(json["format"], "zen.layout.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "checked");

    let layouts = json["layouts"].as_object().expect("layouts object");
    for (name, kind, size, alignment) in [
        ("Ptr<i32>", "pointer", 8, 8),
        ("RawPtr<i32>", "pointer", 8, 8),
        ("[i32]", "slice", 16, 8),
        ("[i32; 4]", "array", 16, 4),
    ] {
        let layout = layouts
            .get(name)
            .unwrap_or_else(|| panic!("missing compound layout entry for {name}"));
        assert_eq!(layout["kind"], kind, "unexpected layout kind for {name}");
        assert_eq!(layout["size"], size, "unexpected layout size for {name}");
        assert_eq!(
            layout["alignment"], alignment,
            "unexpected layout alignment for {name}"
        );
    }

    let handles = &layouts["Handles"];
    assert_eq!(handles["kind"], "struct");
    let fields = handles["fields"].as_array().expect("Handles fields");
    assert_eq!(fields[0]["type"], "Ptr<i32>");
    assert_eq!(fields[1]["type"], "RawPtr<i32>");
    assert_eq!(fields[2]["type"], "[i32]");
    assert_eq!(fields[3]["type"], "[i32; 4]");

    let choice = &layouts["Choice"];
    assert_eq!(choice["kind"], "enum");
    let variants = choice["variants"].as_array().expect("Choice variants");
    assert_eq!(variants[0]["name"], "Empty");
    assert_eq!(variants[0]["tag"], 0);
    assert_eq!(variants[1]["name"], "WithPayload");
    assert_eq!(variants[1]["tag"], 1);
    let payload = variants[1]["payload_fields"]
        .as_array()
        .expect("WithPayload payload fields");
    assert_eq!(payload[0]["type"], "Handles");
}
