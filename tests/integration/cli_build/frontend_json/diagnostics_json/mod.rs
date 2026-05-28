mod behavior_gates;
mod core;
mod fixes;

fn emit_diagnostics_json(source: &str, filename: &str, description: &str) -> serde_json::Value {
    super::golden_support::diagnostics_failure_json(filename, source, description)
}
