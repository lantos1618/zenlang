use super::*;

#[test]
fn public_cast_fixture_uses_prefix_cast_syntax() {
    let source = read("tests/zen/cast.zen");
    assert!(
        !source
            .lines()
            .any(|line| line.trim_start().starts_with("y = x as ")
                || line.trim_start().starts_with("z = 3.14 as ")),
        "tests/zen/cast.zen should use prefix cast(value, Type), not infix as-cast syntax"
    );
    assert!(
        source.contains("cast("),
        "tests/zen/cast.zen should keep executable prefix cast coverage"
    );
}
