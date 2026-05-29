use super::{build_graph_source, write_file, write_zero_main_sources, LIBRARY_SOURCE};

pub(crate) fn write_single_executable_file_read_graph(tmp: &tempfile::TempDir, fallback_arm: &str) {
    write_file_read_executable_graph(
        tmp,
        fallback_arm,
        "myapp\n",
        &[("myapp", "main.zen", "build/")],
    );
}

pub(crate) fn write_mixed_target_file_read_graph(tmp: &tempfile::TempDir, fallback_arm: &str) {
    write_file(
        tmp,
        "build.zen",
        &declared_file_read_graph(
            fallback_arm,
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
        ),
    );
    write_file(tmp, "build.targets", "app\nunit\ncore\n");
    write_zero_main_sources(tmp, &["app.zen", "unit.zen"]);
    write_file(tmp, "lib.zen", LIBRARY_SOURCE);
}

fn write_file_read_executable_graph(
    tmp: &tempfile::TempDir,
    fallback_arm: &str,
    manifest: &str,
    targets: &[(&str, &str, &str)],
) {
    let target_adds = targets
        .iter()
        .map(|(name, main, out_dir)| {
            format!(
                r#"    b.add(Executable {{ name: "{name}", main: "{main}", out_dir: "{out_dir}" }})"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    write_file(
        tmp,
        "build.zen",
        &declared_file_read_graph(fallback_arm, &target_adds),
    );
    write_file(tmp, "build.targets", manifest);
    let target_sources = targets.iter().map(|(_, main, _)| *main).collect::<Vec<_>>();
    write_zero_main_sources(tmp, &target_sources);
}

fn declared_file_read_graph(fallback_arm: &str, target_adds: &str) -> String {
    build_graph_source(&[&format!(
        r#"    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
{target_adds}"#,
    )])
}
