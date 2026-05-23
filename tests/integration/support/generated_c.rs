#[path = "generated_c/calls.rs"]
mod calls;
#[path = "generated_c/definitions.rs"]
mod definitions;

use calls::{generated_c_calls_on_line, is_any_c_function_signature_line};
use definitions::{
    assert_c_function_definition, assert_c_function_definition_count, c_function_definitions,
};

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

pub fn assert_generated_c_calls_resolve_to_definitions(c_source: &str) {
    let undefined = undefined_generated_c_calls(c_source);
    assert!(
        undefined.is_empty(),
        "generated C calls missing emitted definitions: {undefined:?}\n{c_source}"
    );
}

pub fn undefined_generated_c_calls(c_source: &str) -> Vec<String> {
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
