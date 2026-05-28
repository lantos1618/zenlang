use super::support::{
    assert_emit_c_source, assert_no_build_dir, assert_zen_success, run_zen_in,
    write_single_executable_graph, EMIT_ARGS,
};

#[test]
fn emit_command_build_zen_outputs_target_c_source() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_single_executable_graph(&tmp);

    let output = run_zen_in(&tmp, EMIT_ARGS);

    assert_zen_success(EMIT_ARGS, &output);
    assert_emit_c_source(&output);
    assert_no_build_dir(tmp.path(), "zen emit build.zen");
}
