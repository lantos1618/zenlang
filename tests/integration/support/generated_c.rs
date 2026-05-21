#[path = "generated_c/calls.rs"]
mod calls;
#[path = "generated_c/definitions.rs"]
mod definitions;

pub use calls::{
    assert_c_call_resolves_to_definition, assert_generated_c_calls_resolve_to_definitions,
    has_c_call_outside_signature, undefined_generated_c_calls,
};
pub use definitions::{
    assert_c_function_definition_count, assert_generated_c_function_definitions_are_unique,
};

pub(super) fn is_generated_c_function_name(name: &str) -> bool {
    name.contains('_')
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
