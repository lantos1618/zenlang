mod inherited_requirements;
mod overlaps;

const JSON_PRETTY_TRAITS: &str = r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
@export({ Json, PrettyJson })
"#;

const GENERIC_JSON_TRAIT: &str = r#"
Json<T>: behavior {
    encode: (Self) T
}
@export({ Json })
"#;
