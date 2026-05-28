use super::{
    assert_build_zen_rejected, assert_zen_rejected_without_build_outputs, build_graph_source,
    write_file, write_zero_main_sources,
};

pub(crate) type DependencyShapeCase = (&'static [&'static str], &'static str);

pub(crate) const EXECUTABLE_DEPENDENCY_SHAPE_CASES: &[DependencyShapeCase] = &[
    (
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/", dependencies: ["core"] })"#,
        ],
        "build target `app` depends on unknown target `core`",
    ),
    (
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/", dependencies: ["app"] })"#,
        ],
        "build target `app` cannot depend on itself",
    ),
    (
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/", dependencies: ["tool"] })"#,
            r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/", dependencies: ["app"] })"#,
        ],
        "build target dependency cycle includes `app`",
    ),
];

pub(crate) const TEST_DEPENDENCY_SHAPE_CASES: &[DependencyShapeCase] = &[
    (
        &[r#"    b.add(Test { name: "unit", root: "test.zen", dependencies: ["core"] })"#],
        "build target `unit` depends on unknown target `core`",
    ),
    (
        &[r#"    b.add(Test { name: "unit", root: "test.zen", dependencies: ["unit"] })"#],
        "build target `unit` cannot depend on itself",
    ),
    (
        &[
            r#"    b.add(Test { name: "unit", root: "unit.zen", dependencies: ["integration"] })"#,
            r#"    b.add(Test { name: "integration", root: "integration.zen", dependencies: ["unit"] })"#,
        ],
        "build target dependency cycle includes `integration`",
    ),
];

pub(crate) fn assert_dependency_shape_rejections(
    args: &[&str],
    cases: &[DependencyShapeCase],
    command_label: &str,
) {
    for &(targets, diagnostic) in cases {
        let graph = build_graph_source(targets);
        assert_build_zen_rejected(args, &graph, diagnostic, command_label);
    }
}

pub(crate) fn assert_direct_gated_test_dependency_rejected(args: &[&str], command_label: &str) {
    assert_gated_dependency_rejected(
        args,
        command_label,
        &[
            r#"    b.add(Test { name: "unit", root: "test.zen" })"#,
            r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["unit"],
    })"#,
        ],
        &["test.zen", "app.zen"],
        &[],
        "build graph target `app` depends on gated test target `unit`",
    );
}

pub(crate) fn assert_transitive_gated_test_dependency_rejected(args: &[&str], command_label: &str) {
    assert_gated_dependency_rejected(
        args,
        command_label,
        &[
            r#"    b.add(Test { name: "unit", root: "test.zen" })"#,
            r#"    b.add(Library {
        name: "core",
        exports: ["lib.zen"],
        dependencies: ["unit"],
    })"#,
            r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
    })"#,
        ],
        &["test.zen", "app.zen"],
        &[("lib.zen", "value = () i32 { 1 }")],
        "build graph target `core` depends on gated test target `unit`",
    );
}

pub(crate) fn assert_direct_gated_executable_dependency_rejected(
    args: &[&str],
    command_label: &str,
) {
    assert_gated_dependency_rejected(
        args,
        command_label,
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
            r#"    b.add(Test { name: "unit", root: "test.zen", dependencies: ["app"] })"#,
        ],
        &["app.zen", "test.zen"],
        &[],
        "build graph target `unit` depends on gated executable target `app`",
    );
}

pub(crate) fn assert_transitive_gated_executable_dependency_rejected(
    args: &[&str],
    command_label: &str,
) {
    assert_gated_dependency_rejected(
        args,
        command_label,
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
            r#"    b.add(Library {
        name: "core",
        exports: ["lib.zen"],
        dependencies: ["app"],
    })"#,
            r#"    b.add(Test { name: "unit", root: "test.zen", dependencies: ["core"] })"#,
        ],
        &["app.zen", "test.zen"],
        &[("lib.zen", "value = () i32 { 1 }")],
        "build graph target `core` depends on gated executable target `app`",
    );
}

fn assert_gated_dependency_rejected(
    args: &[&str],
    command_label: &str,
    targets: &[&str],
    zero_main_sources: &[&str],
    files: &[(&str, &str)],
    expected_diagnostic: &str,
) {
    let tmp = gated_dependency_graph(targets, zero_main_sources, files);
    assert_zen_rejected_without_build_outputs(&tmp, args, expected_diagnostic, command_label);
}

fn gated_dependency_graph(
    targets: &[&str],
    zero_main_sources: &[&str],
    files: &[(&str, &str)],
) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", &build_graph_source(targets));
    write_zero_main_sources(&tmp, zero_main_sources);
    for (path, source) in files {
        write_file(&tmp, path, source);
    }
    tmp
}
