use super::super::support::{assert_zen_failure, run_zen};
use crate::support::*;

#[test]
fn emit_json_usage_lists_supported_and_gated_modes() {
    let missing_mode_output = run_zen(&["emit-json"]);
    assert_zen_failure(&["emit-json"], &missing_mode_output);

    let hello = test_dir().join("hello.zen");
    let unknown_args = ["emit-json", "unknown", hello.to_str().unwrap()];
    let unknown_mode_output = run_zen(&unknown_args);
    assert_zen_failure(&unknown_args, &unknown_mode_output);

    let expected_usage =
        "Usage: zen emit-json <ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml> <file.zen>";
    for output in [&missing_mode_output, &unknown_mode_output] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_usage),
            "emit-json usage should list supported and gated modes, stderr={stderr}"
        );
    }
}
