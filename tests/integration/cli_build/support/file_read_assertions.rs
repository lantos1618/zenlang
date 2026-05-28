use super::file_read_graphs::{
    write_mixed_target_file_read_graph, write_multiple_executable_file_read_graph,
    write_single_executable_file_read_graph,
};
use super::{
    assert_built_binaries_run, assert_check_build_zen_summary, assert_no_build_dir,
    assert_path_exists, assert_stderr_lacks, assert_stdout_contains, assert_zen_failure_contains,
    assert_zen_success, build_graph_source, run_zen_in, write_file, write_zero_main_sources,
    LIBRARY_SOURCE,
};

pub(crate) const DECLARED_FILE_READ_FALLBACK_ARMS: &[&str] = &[
    r#"| .Err { "default" }"#,
    r#"| _ { "default" }"#,
    r#"| err { "default" }"#,
];

pub(crate) fn assert_declared_file_read_single_executable_build(
    args: &[&str],
    fallback_arm: &str,
    run_binary: bool,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_single_executable_file_read_graph(&tmp, fallback_arm);

    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);

    let bin_path = tmp.path().join("build").join("myapp");
    if run_binary {
        assert_built_binaries_run(&[bin_path]);
    } else {
        assert_path_exists(bin_path);
    }
}

pub(crate) fn assert_declared_file_read_multiple_executable_build(
    args: &[&str],
    fallback_arm: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_multiple_executable_file_read_graph(&tmp, fallback_arm);

    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);

    for bin_path in [
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ] {
        assert_path_exists(bin_path);
    }
}

pub(crate) fn assert_declared_file_read_emit(args: &[&str], fallback_arm: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_single_executable_file_read_graph(&tmp, fallback_arm);

    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);
    assert_stdout_contains(
        &output,
        "int32_t zen_main(void)",
        "expected target C source",
    );
    assert_no_build_dir(tmp.path(), &format!("zen {}", args.join(" ")));
}

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

pub(crate) fn assert_declared_file_read_unselected_target_build(args: &[&str], fallback_arm: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_unselected_target_file_read_graph(&tmp, fallback_arm, "missing_unit.zen", false);

    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);

    let bin_path = tmp.path().join("build").join("app").join("app");
    assert_path_exists(bin_path);
}

pub(crate) fn assert_declared_file_read_unselected_target_emit(args: &[&str], fallback_arm: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_unselected_target_file_read_graph(&tmp, fallback_arm, "unit.zen", true);

    let output = run_zen_in(&tmp, args);
    assert_zen_success(args, &output);
    assert_stdout_contains(
        &output,
        "int32_t zen_main(void)",
        "expected target C source",
    );
    assert_no_build_dir(tmp.path(), &format!("zen {}", args.join(" ")));
}

pub(crate) fn assert_file_read_rejected_before_unselected_targets(
    args: &[&str],
    build_source: impl AsRef<str>,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", build_source.as_ref());
    write_zero_main_sources(&tmp, &["app.zen"]);
    write_file(&tmp, "lib.zen", LIBRARY_SOURCE);

    let output = run_zen_in(&tmp, args);
    assert_zen_failure_contains(
        args,
        &output,
        "undeclared host effect: read file `build.targets`",
    );
    assert_stderr_lacks(
        &output,
        &["missing_unit.zen"],
        "host-effect validation should run before unrelated test source handling",
    );
    assert_no_build_dir(tmp.path(), &format!("zen {}", args.join(" ")));
}

pub(crate) fn undeclared_file_read_unselected_target_graph() -> String {
    build_graph_source(&[r#"
    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
"#])
}

pub(crate) fn missing_fallback_file_read_unselected_target_graph() -> String {
    build_graph_source(&[r#"
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
"#])
}

fn write_unselected_target_file_read_graph(
    tmp: &tempfile::TempDir,
    fallback_arm: &str,
    test_root: &str,
    write_test_source: bool,
) {
    let body = format!(
        r#"    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
    b.add(Executable {{ name: "app", main: "app.zen", out_dir: "build/app/" }})
    b.add(Test {{ name: "unit", root: "{test_root}" }})
    b.add(Library {{ name: "core", exports: ["lib.zen"] }})"#,
    );
    write_file(tmp, "build.zen", &build_graph_source(&[&body]));
    write_file(tmp, "build.targets", "app\n");
    write_zero_main_sources(tmp, &["app.zen"]);
    write_file(tmp, "lib.zen", LIBRARY_SOURCE);
    if write_test_source {
        write_zero_main_sources(tmp, &[test_root]);
    }
}
