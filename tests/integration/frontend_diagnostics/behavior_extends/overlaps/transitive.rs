use super::*;

#[test]
fn imported_behavior_extends_transitive_parent_impl_overlap_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub CompactJson: behavior {
    compact: (Self) StaticString
}

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

CompactJson.extends(Json<StaticString>)
PrettyJson.extends(CompactJson)
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Json, PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString {
        "point"
    }
}

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString {
        "point"
    }

    compact = (value: Point) StaticString {
        "compact"
    }

    pretty = (value: Point) StaticString {
        "pretty"
    }
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported transitive overlapping behavior impls");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains(
            "overlapping implementations of behaviors `Json_StaticString` and `PrettyJson` for type `Point`"
        ),
        "expected imported transitive behavior impl overlap diagnostic, panic={message}"
    );
}
