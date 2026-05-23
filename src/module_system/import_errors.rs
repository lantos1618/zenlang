use crate::error::{CompileError, Span};

pub(in crate::module_system) fn private_export_error(
    name: &str,
    module_name: &str,
    span: Span,
) -> Vec<CompileError> {
    vec![CompileError::Resolution(
        format!(
            "symbol '{}' in module '{}' is not exported",
            name, module_name
        ),
        Some(span),
    )]
}

pub(in crate::module_system) fn missing_export_error(
    name: &str,
    module_name: &str,
    span: Span,
) -> Vec<CompileError> {
    vec![CompileError::Resolution(
        format!("module '{}' does not export '{}'", module_name, name),
        Some(span),
    )]
}
