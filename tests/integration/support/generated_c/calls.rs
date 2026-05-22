use super::definitions::assert_c_function_definition;

#[path = "calls/function_pointers.rs"]
mod function_pointers;
#[path = "calls/scan.rs"]
mod scan;
#[path = "calls/signatures.rs"]
mod signatures;

pub use scan::undefined_generated_c_calls;
pub use signatures::has_c_call_outside_signature;

pub fn assert_c_call_resolves_to_definition(c_source: &str, name: &str) {
    assert_c_function_definition(c_source, name);
    assert!(
        has_c_call_outside_signature(c_source, name),
        "expected generated C call to `{name}` outside declarations/definitions:\n{c_source}"
    );
}

pub fn assert_generated_c_calls_resolve_to_definitions(c_source: &str) {
    let undefined = undefined_generated_c_calls(c_source);
    assert!(
        undefined.is_empty(),
        "generated C calls missing emitted definitions: {undefined:?}\n{c_source}"
    );
}
