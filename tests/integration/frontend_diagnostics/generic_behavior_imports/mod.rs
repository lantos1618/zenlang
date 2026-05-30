mod arity;
mod duplicate_requires;
mod requirements;

use super::support::frontend_diagnostics_for_module;
use zen::error::Diagnostic;

const GENERIC_JSON_TRAIT: &str = r#"
Json<T>: behavior {
    encode: (Self) T
}
@export({ Json })
"#;

fn generic_json_diagnostics(main: &str) -> Vec<Diagnostic> {
    frontend_diagnostics_for_module("traits.zen", GENERIC_JSON_TRAIT, main)
}
