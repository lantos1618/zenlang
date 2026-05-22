use super::*;

#[test]
fn c_call_assertion_ignores_struct_return_definitions() {
    let c_source = r#"
typedef struct Box_Option_i32 Box_Option_i32;

Box_Option_i32 Box_copy_Option_i32(Box_Option_i32 self) {
    return self;
}
"#;

    assert!(
        !has_c_call_outside_signature(c_source, "Box_copy_Option_i32"),
        "definition-only generated C should not count as a call"
    );
}

#[test]
fn generated_c_call_definition_scan_reports_missing_generated_calls() {
    let c_source = r#"
#include <stdio.h>

int32_t inner_i32(int32_t value) {
    return value;
}

int32_t outer_i32(int32_t value) {
    printf("%d", value);
    missing_stmt_i32(value);
    return missing_i32(value) + inner_i32(value) + id(12LL);
}
"#;

    assert_eq!(
        undefined_generated_c_calls(c_source),
        vec![
            "missing_stmt_i32".to_string(),
            "missing_i32".to_string(),
            "id".to_string()
        ]
    );
}

#[test]
fn generated_c_definition_count_ignores_prototypes() {
    let c_source = r#"
int32_t inner_i32(int32_t value);

int32_t inner_i32(int32_t value) {
    return value;
}
"#;

    assert_c_function_definition_count(c_source, "inner_i32", 1);
}

// ── Individual test cases ───────────────────────────────────────────
