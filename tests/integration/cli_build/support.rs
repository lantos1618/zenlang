pub(super) fn transitive_gated_test_dependency_graph() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "test.zen" })
    b.add(Library {
        name: "core",
        exports: ["lib.zen"],
        dependencies: ["unit"],
    })
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("test.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write test.zen");
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    1
}
"#,
    )
    .expect("write lib.zen");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");
    tmp
}
