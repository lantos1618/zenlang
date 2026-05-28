mod inherited_requirements;
mod overlaps;

const JSON_PRETTY_TRAITS: &str = r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
"#;

const GENERIC_JSON_TRAIT: &str = r#"
pub Json<T>: behavior {
    encode: (Self) T
}
"#;
