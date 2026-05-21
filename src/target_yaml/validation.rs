use super::{TargetBackendCodegen, TargetYamlError, TargetYamlInput};

pub(super) fn validate_target_yaml(input: &TargetYamlInput) -> Result<(), TargetYamlError> {
    if input.layout.is_some() || input.overrides.is_some() {
        return Err(TargetYamlError::Schema(
            "target YAML cannot override compiler-owned type layouts".into(),
        ));
    }
    if input.triple.trim().is_empty() {
        return Err(TargetYamlError::Schema(
            "target YAML `triple` cannot be empty".into(),
        ));
    }
    if !matches!(input.pointer_width, 32 | 64) {
        return Err(TargetYamlError::Schema(
            "target YAML `pointer_width` must be 32 or 64".into(),
        ));
    }
    if input.abi.trim().is_empty() {
        return Err(TargetYamlError::Schema(
            "target YAML `abi` cannot be empty".into(),
        ));
    }
    if let Some(backend) = &input.backend {
        if matches!(backend.codegen, TargetBackendCodegen::Unsupported) {
            return Err(TargetYamlError::Schema(
                "target YAML `backend.codegen` supports only `c` in this phase".into(),
            ));
        }
        if backend
            .c_compiler
            .as_ref()
            .is_some_and(|compiler| compiler.trim().is_empty())
        {
            return Err(TargetYamlError::Schema(
                "target YAML `backend.c_compiler` cannot be empty".into(),
            ));
        }
        if backend.c_flags.iter().any(|flag| flag.trim().is_empty()) {
            return Err(TargetYamlError::Schema(
                "target YAML `backend.c_flags` entries cannot be empty".into(),
            ));
        }
    }
    Ok(())
}
