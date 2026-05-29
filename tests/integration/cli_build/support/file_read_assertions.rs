use super::assert_check_build_zen_summary;
use super::file_read_graphs::{
    write_mixed_target_file_read_graph, write_single_executable_file_read_graph,
};

pub(crate) const DECLARED_FILE_READ_FALLBACK_ARMS: &[&str] = &[
    r#"| .Err { "default" }"#,
    r#"| _ { "default" }"#,
    r#"| err { "default" }"#,
];

pub(crate) fn assert_declared_file_read_single_executable_check(fallback_arm: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_single_executable_file_read_graph(&tmp, fallback_arm);
    assert_check_build_zen_summary(&tmp, "1 build targets");
}

pub(crate) fn assert_declared_file_read_mixed_target_check(fallback_arm: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_mixed_target_file_read_graph(&tmp, fallback_arm);
    assert_check_build_zen_summary(&tmp, "3 build targets");
}
