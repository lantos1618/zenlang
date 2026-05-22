use super::*;

#[test]
fn absent_metadata_entry_formats_message() {
    let entry = AbsentMetadataEntry {
        present: true,
        code: "ABSENT",
        label: "parameter count",
    };

    assert_eq!(entry.code, "ABSENT");
    assert_eq!(
        entry.message("value", "main"),
        "resolver value symbol 'main' has parameter count metadata, expected none"
    );
}

#[test]
fn resolver_named_list_display_formats_known_and_missing_items() {
    let fields = vec![("value".to_string(), "i32".to_string())];
    assert_eq!(
        format_resolver_named_list(Some(&fields), |ty: &String| ty.clone()),
        "(value: i32)"
    );
    assert_eq!(
        format_resolver_named_list::<String>(None, |ty: &String| ty.clone()),
        "unknown"
    );
}
