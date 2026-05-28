use super::golden_support::assert_diagnostics_failure_golden;
mod behavior_association;
mod control_flow;
mod generic_methods;
mod removed_syntax;
mod type_resolution;

fn assert_diagnostics_golden(
    file_name: &str,
    source: &str,
    description: &str,
    expected_diagnostic_count: usize,
    followup_message: &str,
) {
    let stem = match file_name {
        "derive_gate.zen" => "behavior_derive_gate",
        "return_keyword.zen" => "return",
        _ => file_name.strip_suffix(".zen").unwrap_or(file_name),
    };
    assert_diagnostics_failure_golden(
        file_name,
        source,
        description,
        expected_diagnostic_count,
        followup_message,
        stem,
    );
}
