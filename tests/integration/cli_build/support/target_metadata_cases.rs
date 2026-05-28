pub(crate) type TargetMetadataCase = (&'static str, &'static str);

pub(crate) const EXECUTABLE_TARGET_METADATA_CASES: &[TargetMetadataCase] = &[
    (
        r#"    b.add(Executable { name: "app", name: "tool", main: "app.zen", out_dir: "build/app/" })"#,
        "duplicate field `name` in `Executable` build target",
    ),
    (
        r#"    b.add(Executable { name: "app", main: "app.zen" })"#,
        "missing required field `out_dir` in `Executable` build target",
    ),
    (
        r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: 42 })"#,
        "field `out_dir` in `Executable` build target must be a string",
    ),
];

pub(crate) const TEST_TARGET_METADATA_CASES: &[TargetMetadataCase] = &[
    (
        r#"    b.add(Test { name: "unit", name: "integration", root: "test.zen" })"#,
        "duplicate field `name` in `Test` build target",
    ),
    (
        r#"    b.add(Test { name: "unit" })"#,
        "missing required field `root` or `root_source_file` in `Test` build target",
    ),
    (
        r#"    b.add(Test { name: "unit", root: 42 })"#,
        "field `root` in `Test` build target must be a string",
    ),
    (
        r#"    b.add(Test { name: "unit", root: "test.zen", out_dir: "build/tests/" })"#,
        "unknown field `out_dir` in `Test` build target",
    ),
];

pub(crate) const LIBRARY_TARGET_METADATA_CASES: &[TargetMetadataCase] = &[
    (
        r#"    b.add(Library { name: "core", name: "utils", exports: ["lib.zen"] })"#,
        "duplicate field `name` in `Library` build target",
    ),
    (
        r#"    b.add(Library { name: "core" })"#,
        "missing required field `exports` in `Library` build target",
    ),
    (
        r#"    b.add(Library { name: "core", exports: "lib.zen" })"#,
        "field `exports` in `Library` build target must be an array of strings",
    ),
    (
        r#"    b.add(Library { name: "core", exports: [] })"#,
        "field `exports` in `Library` build target must contain at least one source",
    ),
    (
        r#"    b.add(Library { name: "core", exports: ["lib.zen"], output_dir: "build/lib/" })"#,
        "unknown field `output_dir` in `Library` build target",
    ),
];
