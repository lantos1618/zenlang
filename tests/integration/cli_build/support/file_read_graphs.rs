use super::{main_source, write_file};

pub(crate) fn write_single_executable_file_read_graph(tmp: &tempfile::TempDir, fallback_arm: &str) {
    write_file_read_executable_graph(
        tmp,
        fallback_arm,
        "myapp\n",
        &[ExecutableFileReadTarget {
            name: "myapp",
            main: "main.zen",
            out_dir: "build/",
            result: "0",
        }],
    );
}

pub(crate) fn write_multiple_executable_file_read_graph(
    tmp: &tempfile::TempDir,
    fallback_arm: &str,
) {
    write_file_read_executable_graph(
        tmp,
        fallback_arm,
        "app\ntool\n",
        &[
            ExecutableFileReadTarget {
                name: "app",
                main: "app.zen",
                out_dir: "build/app/",
                result: "0",
            },
            ExecutableFileReadTarget {
                name: "tool",
                main: "tool.zen",
                out_dir: "build/tool/",
                result: "0",
            },
        ],
    );
}

struct ExecutableFileReadTarget<'a> {
    name: &'a str,
    main: &'a str,
    out_dir: &'a str,
    result: &'a str,
}

fn write_file_read_executable_graph(
    tmp: &tempfile::TempDir,
    fallback_arm: &str,
    manifest: &str,
    targets: &[ExecutableFileReadTarget<'_>],
) {
    let target_adds = targets
        .iter()
        .map(|target| {
            format!(
                r#"    b.add(Executable {{ name: "{}", main: "{}", out_dir: "{}" }})"#,
                target.name, target.main, target.out_dir
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    write_file(
        tmp,
        "build.zen",
        &format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
{target_adds}
    .Ok(b.config())
}}
"#,
        ),
    );
    write_file(tmp, "build.targets", manifest);
    for target in targets {
        write_file(tmp, target.main, main_source(target.result).as_str());
    }
}
