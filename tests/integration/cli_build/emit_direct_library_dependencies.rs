use super::support::{
    assert_no_build_dir, assert_stdout_contains, assert_zen_success, build_graph_source,
    run_zen_in, write_file, EMIT_ARGS, LIBRARY_SOURCE, MAIN_ZERO,
};

#[test]
fn emit_command_build_zen_accepts_library_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "build.zen",
        &build_graph_source(&[
            r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
            r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
    })"#,
        ]),
    );
    write_file(&tmp, "lib.zen", LIBRARY_SOURCE);
    write_file(&tmp, "app.zen", MAIN_ZERO);

    let output = run_zen_in(&tmp, EMIT_ARGS);

    assert_zen_success(EMIT_ARGS, &output);
    assert_stdout_contains(&output, "int main(", "expected emitted C source");
    assert_no_build_dir(tmp.path(), "zen emit build.zen");
}
