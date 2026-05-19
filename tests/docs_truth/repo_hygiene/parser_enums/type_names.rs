use super::*;

#[test]
fn parser_type_names_use_owned_type_name_enums() {
    let parser_types = read("src/parser/types.rs");
    let type_names_root = read("src/ast/types.rs");
    let type_names_owned = read("src/ast/types/names.rs");
    let type_names = format!("{type_names_root}\n{type_names_owned}");

    for forbidden in [
        r#""i8" =>"#,
        r#""i16" =>"#,
        r#""i32" =>"#,
        r#""i64" =>"#,
        r#""u8" =>"#,
        r#""u16" =>"#,
        r#""u32" =>"#,
        r#""u64" =>"#,
        r#""usize" =>"#,
        r#""f32" =>"#,
        r#""f64" =>"#,
        r#""bool" =>"#,
        r#""void" =>"#,
        r#""str" =>"#,
        r#""StaticString" =>"#,
        r#""Self" =>"#,
        r#""Ptr" if"#,
        r#""MutPtr" if"#,
        r#""RawPtr" if"#,
        r#""Slice" if"#,
        "match base.as_str()",
    ] {
        assert!(
            !parser_types.contains(forbidden),
            "parser type-name resolution should parse through owned parser type-name enums: {forbidden}"
        );
    }

    for forbidden in [
        "Self::I8_NAME => Ok(Self::I8)",
        "Self::I16_NAME => Ok(Self::I16)",
        "Self::I32_NAME => Ok(Self::I32)",
        "Self::I64_NAME => Ok(Self::I64)",
        "Self::U8_NAME => Ok(Self::U8)",
        "Self::U16_NAME => Ok(Self::U16)",
        "Self::U32_NAME => Ok(Self::U32)",
        "Self::U64_NAME => Ok(Self::U64)",
        "Self::USIZE_NAME => Ok(Self::Usize)",
        "Self::F32_NAME => Ok(Self::F32)",
        "Self::F64_NAME => Ok(Self::F64)",
        "Self::BOOL_NAME => Ok(Self::Bool)",
        "Self::VOID_NAME => Ok(Self::Void)",
        "Self::STR_NAME => Ok(Self::Str)",
        "STATIC_STRING_TYPE_NAME => Ok(Self::StaticString)",
        "Self::SELF_NAME => Ok(Self::SelfType)",
        "Self::PTR => Ok(Self::Ptr)",
        "Self::MUT_PTR => Ok(Self::MutPtr)",
        "Self::RAW_PTR => Ok(Self::RawPtr)",
        "Self::SLICE => Ok(Self::Slice)",
    ] {
        assert!(
            !type_names.contains(forbidden),
            "parser type-name FromStr should use enum-owned static tables, not raw match arms: {forbidden}"
        );
    }

    for required in [
        "pub enum BuiltinTypeName",
        "pub enum BuiltinGenericTypeName",
        "pub const ALL: &[BuiltinTypeName]",
        "pub const ALL: &[BuiltinGenericTypeName]",
        "impl FromStr for BuiltinTypeName",
        "impl FromStr for BuiltinGenericTypeName",
        "impl fmt::Display for BuiltinTypeName",
        "impl fmt::Display for BuiltinGenericTypeName",
        ".find(|name| name.as_str() == value)",
        "name.parse::<BuiltinTypeName>()",
        "base.parse::<BuiltinGenericTypeName>()",
    ] {
        assert!(
            type_names.contains(required) || parser_types.contains(required),
            "parser type-name spelling should live in shared AST type-name enums: {required}"
        );
    }
}
