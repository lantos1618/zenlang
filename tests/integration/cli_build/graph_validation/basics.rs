use super::super::support::{
    assert_check_build_zen_summary, build_graph_source, write_file, LIBRARY_SOURCE, MAIN_ZERO,
};

#[test]
fn check_command_validates_build_zen_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "build.zen",
        &build_graph_source(&[
            r#"    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#,
        ]),
    );
    write_file(&tmp, "main.zen", MAIN_ZERO);

    assert_check_build_zen_summary(&tmp, "1 build targets");
}

#[test]
fn check_command_build_zen_accepts_library_only_graph_validation() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "build.zen",
        &build_graph_source(&[r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#]),
    );
    write_file(&tmp, "lib.zen", LIBRARY_SOURCE);

    assert_check_build_zen_summary(&tmp, "1 build targets");
}
