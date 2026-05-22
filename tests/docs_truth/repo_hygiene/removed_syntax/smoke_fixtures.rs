use super::*;

#[test]
fn root_smoke_fixtures_do_not_use_removed_or_gated_syntax() {
    let mut paths = std::fs::read_dir(repo_root().join("tests"))
        .expect("read tests directory")
        .map(|entry| {
            let entry = entry.expect("tests directory entry should be readable");
            entry
                .path()
                .strip_prefix(repo_root())
                .expect("test path should be under repo root")
                .to_string_lossy()
                .into_owned()
        })
        .filter(|path| path.starts_with("tests/test_") && path.ends_with(".zen"))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(!paths.is_empty(), "expected root smoke fixtures");

    for path in paths {
        let source = read(&path);
        assert!(
            !source.contains("return "),
            "{path} still uses the removed return keyword"
        );
        for gated_claim in [
            "@std.memory",
            "Heap.sync",
            "Arena.async",
            "Allocator",
            "ExecutionMode",
            "function coloring",
            "async/await",
        ] {
            assert!(
                !source.contains(gated_claim),
                "{path} still teaches gated allocator/effect syntax: {gated_claim}"
            );
        }
    }
}
