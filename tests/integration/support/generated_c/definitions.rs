use super::is_generated_c_function_name;

pub(super) fn assert_c_function_definition(c_source: &str, name: &str) {
    let needle = format!(" {name}(");
    assert!(
        c_source
            .lines()
            .any(|line| line.trim_end().ends_with('{') && line.contains(&needle)),
        "expected generated C definition for `{name}`:\n{c_source}"
    );
}

pub fn assert_c_function_definition_count(c_source: &str, name: &str, expected: usize) {
    let actual = c_function_definitions(c_source)
        .iter()
        .filter(|definition| definition.as_str() == name)
        .count();
    assert_eq!(
        actual, expected,
        "expected {expected} generated C definitions for `{name}`, found {actual}:\n{c_source}"
    );
}

pub fn assert_generated_c_function_definitions_are_unique(c_source: &str) {
    let definitions = c_function_definitions(c_source);
    let mut duplicates = Vec::new();

    for definition in &definitions {
        if definitions
            .iter()
            .filter(|candidate| *candidate == definition)
            .count()
            > 1
            && !duplicates.contains(definition)
        {
            duplicates.push(definition.clone());
        }
    }

    assert!(
        duplicates.is_empty(),
        "generated C emitted duplicate function definitions: {duplicates:?}\n{c_source}"
    );
}

pub(super) fn c_function_definitions(c_source: &str) -> Vec<String> {
    c_source
        .lines()
        .filter_map(|line| c_function_definition_name(line.trim()))
        .collect()
}

fn c_function_definition_name(trimmed: &str) -> Option<String> {
    if !trimmed.ends_with('{') {
        return None;
    }

    let paren = trimmed.find('(')?;
    let before = trimmed[..paren].trim_end();
    let name_start = before
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map_or(0, |idx| idx + 1);
    let name = &before[name_start..];

    if is_generated_c_function_name(name) {
        Some(name.to_string())
    } else {
        None
    }
}
