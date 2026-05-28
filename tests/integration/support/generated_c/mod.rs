mod calls;
mod definitions;

use std::collections::HashSet;

use calls::{generated_c_calls_on_line, is_any_c_function_signature_line};
use definitions::{
    assert_c_function_definition, assert_c_function_definition_count, c_function_definitions,
};

pub fn assert_generated_c_function_definitions_are_unique(c_source: &str) {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for definition in c_function_definitions(c_source) {
        if !seen.insert(definition) && !duplicates.contains(&definition) {
            duplicates.push(definition);
        }
    }

    assert!(
        duplicates.is_empty(),
        "generated C emitted duplicate function definitions: {duplicates:?}\n{c_source}"
    );
}

fn assert_c_call_resolves_to_definition(c_source: &str, name: &str) {
    assert_c_function_definition(c_source, name);
    assert!(
        has_c_call_outside_signature(c_source, name),
        "expected generated C call to `{name}` outside declarations/definitions:\n{c_source}"
    );
}

pub fn assert_c_call_resolves_to_single_definition(c_source: &str, name: &str) {
    assert_c_call_resolves_to_definition(c_source, name);
    assert_c_function_definition_count(c_source, name, 1);
}

pub fn assert_generated_c_specialization(
    c_source: &str,
    required_snippets: &[&str],
    single_definition_calls: &[&str],
    forbidden_snippets: &[&str],
) {
    assert_c_source_contains_all(c_source, required_snippets);
    for name in single_definition_calls {
        assert_c_call_resolves_to_single_definition(c_source, name);
    }
    assert_c_source_lacks_all(c_source, forbidden_snippets);
}

fn assert_c_source_contains_all(c_source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            c_source.contains(snippet),
            "expected generated C to contain `{snippet}`:\n{c_source}"
        );
    }
}

fn assert_c_source_lacks_all(c_source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !c_source.contains(snippet),
            "expected generated C to omit `{snippet}`:\n{c_source}"
        );
    }
}

pub fn assert_generated_c_calls_resolve_to_definitions(c_source: &str) {
    let undefined = undefined_generated_c_calls(c_source);
    assert!(
        undefined.is_empty(),
        "generated C calls missing emitted definitions: {undefined:?}\n{c_source}"
    );
}

pub fn undefined_generated_c_calls(c_source: &str) -> Vec<&str> {
    let definitions = c_function_definitions(c_source);
    let mut undefined = Vec::new();

    for line in c_source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("typedef ")
            || is_any_c_function_signature_line(trimmed)
        {
            continue;
        }

        for call in generated_c_calls_on_line(trimmed) {
            if !definitions.contains(&call) && !undefined.contains(&call) {
                undefined.push(call);
            }
        }
    }

    undefined
}

pub fn has_c_call_outside_signature(c_source: &str, name: &str) -> bool {
    calls::has_c_call_outside_signature(c_source, name)
}
