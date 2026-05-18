use super::*;

#[test]
fn ci_and_release_only_advertise_existing_targets() {
    let ci = read(".github/workflows/ci.yml");
    let release = read(".github/workflows/release.yml");
    let makefile = read("Makefile");
    let vscode_settings = read(".vscode/settings.json");
    let vscode_launch = read(".vscode/launch.json");
    let extension_readme = read("vscode-extension/README.md");
    let extension_package = read("vscode-extension/package.json");
    let extension_source = read("vscode-extension/src/extension.ts");
    let setup_lsp = read("setup_lsp.sh");

    assert!(ci.contains("cargo fmt --check"));
    assert!(ci.contains("cargo clippy -- -D warnings"));
    assert!(ci.contains("cargo test --lib"));
    assert!(ci.contains("cargo test --tests"));
    assert!(
        ci.contains("types: [reopened, ready_for_review]"),
        "CI pull_request triggers must avoid draft-open and synchronize spam while still running when PRs leave draft"
    );
    assert!(
        !ci.contains("\n  push:"),
        "CI workflow should not run on normal branch pushes; use PR ready-for-review checks or manual dispatch"
    );
    assert!(
        !ci.contains("types: [opened") && !ci.contains(", opened") && !ci.contains("- opened"),
        "CI workflow should not create skipped runs for draft PR creation"
    );
    assert!(
        !ci.contains("synchronize"),
        "CI workflow should not run on every draft PR synchronize push"
    );
    assert_eq!(
        ci.matches("github.event.pull_request.draft == false")
            .count(),
        3,
        "CI fmt, clippy, and test jobs must keep the draft-PR guard"
    );
    assert!(
        ci.contains("workflow_dispatch"),
        "CI must keep a manual dispatch path when draft PR pushes do not run checks"
    );

    let phase_plan = read("docs/PHASE_PLAN.md");
    let completion_audit = read("docs/COMPLETION_AUDIT.md");
    for (path, contents) in [
        ("docs/PHASE_PLAN.md", phase_plan),
        ("docs/COMPLETION_AUDIT.md", completion_audit),
    ] {
        assert!(
            contents.contains("normal branch pushes"),
            "{path} should record that CI stays quiet on normal branch pushes"
        );
    }

    for unsupported in ["LLVM", "zen-lsp", "aarch64-apple-darwin"] {
        assert!(
            !release.contains(unsupported),
            "release workflow advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !makefile.contains(unsupported),
            "Makefile advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !vscode_settings.contains(unsupported),
            ".vscode/settings.json advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !vscode_launch.contains(unsupported),
            ".vscode/launch.json advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !extension_readme.contains(unsupported),
            "vscode-extension/README.md advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !extension_package.contains(unsupported),
            "vscode-extension/package.json advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !extension_source.contains(unsupported),
            "vscode-extension/src/extension.ts advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !setup_lsp.contains(unsupported),
            "setup_lsp.sh advertises unsupported target/artifact: {unsupported}"
        );
    }
}

#[test]
fn checked_in_configs_do_not_contain_secret_literals() {
    let config_paths = [
        ".github/copilot-mcp.json",
        ".github/workflows/ci.yml",
        ".github/workflows/release.yml",
        ".vscode/settings.json",
        ".vscode/launch.json",
        ".vscode/zen-language-config.json",
    ];

    for path in config_paths {
        let contents = read(path);
        for marker in ["Bearer ", "sk-", "ghp_", "github_pat_", "nk_"] {
            assert!(
                !contents.contains(marker),
                "{path} contains credential-looking marker {marker}"
            );
        }
    }
}
