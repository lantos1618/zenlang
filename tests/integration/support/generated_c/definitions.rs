use super::calls::{c_function_head, is_tracked_c_function_name};

pub(super) fn assert_c_function_definition(c_source: &str, name: &str) {
    let needle = format!(" {name}(");
    assert!(
        c_source
            .lines()
            .any(|line| line.trim_end().ends_with('{') && line.contains(&needle)),
        "expected generated C definition for `{name}`:\n{c_source}"
    );
}

pub(super) fn assert_c_function_definition_count(c_source: &str, name: &str, expected: usize) {
    let actual = c_function_definitions(c_source)
        .into_iter()
        .filter(|definition| *definition == name)
        .count();
    assert_eq!(
        actual, expected,
        "expected {expected} generated C definitions for `{name}`, found {actual}:\n{c_source}"
    );
}

pub(super) fn c_function_definitions(c_source: &str) -> Vec<&str> {
    c_source
        .lines()
        .filter_map(|line| c_function_definition_name(line.trim()))
        .collect()
}

fn c_function_definition_name(trimmed: &str) -> Option<&str> {
    if !trimmed.ends_with('{') {
        return None;
    }

    let (_, name) = c_function_head(trimmed)?;
    is_tracked_c_function_name(name).then_some(name)
}
