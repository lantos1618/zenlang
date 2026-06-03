use super::support::{assert_diagnostic_code_and_message, frontend_diagnostics_for_modules};

#[test]
fn contextual_integer_literal_overflow_is_reported() {
    let diagnostics = frontend_diagnostics_for_modules(
        &[],
        r#"
take_byte = (n: u8) void {
}

main = () i32 {
    take_byte(256)
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3074",
        "integer literal `256` does not fit in `u8`",
        "contextual integer literal overflow",
    );
}

#[test]
fn contextual_numeric_literal_overflow_is_reported_for_primitive_targets() {
    let diagnostics = frontend_diagnostics_for_modules(
        &[],
        r#"
take_i8 = (n: i8) void {
}
take_u16 = (n: u16) void {
}
take_u32 = (n: u32) void {
}
take_u64 = (n: u64) void {
}
take_f32 = (n: f32) void {
}

main = () i32 {
    take_i8(128)
    take_u16(65536)
    take_u32(4294967296)
    take_u64(18446744073709551616)
    take_f32(1000000000000000000000000000000000000000.0)
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3074",
        "integer literal `128` does not fit in `i8`",
        "contextual i8 literal overflow",
    );
    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3074",
        "integer literal `65536` does not fit in `u16`",
        "contextual u16 literal overflow",
    );
    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3074",
        "integer literal `4294967296` does not fit in `u32`",
        "contextual u32 literal overflow",
    );
    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3074",
        "integer literal `18446744073709551616` does not fit in `u64`",
        "contextual u64 literal overflow",
    );
    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3074",
        "does not fit in `f32`",
        "contextual f32 literal overflow",
    );
}
