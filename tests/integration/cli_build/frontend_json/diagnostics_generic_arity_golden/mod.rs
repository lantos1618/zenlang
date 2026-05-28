use super::golden_support::assert_diagnostics_failure_golden;

mod annotations;
mod call_site_annotations;
mod constructors;

fn assert_diagnostics_golden(
    zen_filename: &str,
    source: &str,
    failure_context: &str,
    single_diagnostic_context: &str,
) {
    let stem = zen_filename.strip_suffix(".zen").unwrap_or(zen_filename);
    assert_diagnostics_failure_golden(
        zen_filename,
        source,
        failure_context,
        1,
        single_diagnostic_context,
        stem,
    );
}
