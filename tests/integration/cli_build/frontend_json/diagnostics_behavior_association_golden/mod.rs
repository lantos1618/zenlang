use super::golden_support::assert_diagnostics_failure_golden;
mod relationship_arity;
mod requires;

fn assert_behavior_association_diagnostics_golden(source: &str, filename: &str, description: &str) {
    let stem = filename.strip_suffix(".zen").unwrap_or(filename);
    assert_diagnostics_failure_golden(filename, source, description, 1, description, stem);
}
