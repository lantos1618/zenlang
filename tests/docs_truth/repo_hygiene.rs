use super::*;

#[test]
fn v1_spec_is_single_source_of_truth_and_old_spec_is_quarantined() {
    let old_spec = read("LANGUAGE_SPEC.zen");
    let old_spec_header = old_spec.lines().take(20).collect::<Vec<_>>().join("\n");

    assert!(
        old_spec_header.contains("Archived aspirational notes"),
        "LANGUAGE_SPEC.zen must be explicitly quarantined at the top"
    );
    assert!(
        old_spec_header.contains("docs/V1_SPEC.md is the versioned v1 source of truth"),
        "LANGUAGE_SPEC.zen must point to docs/V1_SPEC.md as the source of truth"
    );
}

#[test]
fn aspirational_stdlib_is_explicitly_experimental() {
    let stdlib_readme = read("stdlib/README.md");

    for required in [
        "experimental",
        "not part of the implemented v1 surface",
        "must parse, typecheck, and build",
        "docs/V1_SPEC.md",
    ] {
        assert!(
            stdlib_readme.contains(required),
            "stdlib/README.md is missing experimental gate text: {required}"
        );
    }
}

#[test]
fn stale_rewrite_audit_docs_are_removed() {
    for path in ["docs/REWRITE.md", "docs/AUDIT_PROMPT.md"] {
        assert!(
            !repo_root().join(path).exists(),
            "{path} is stale rewrite/audit scaffolding; keep durable status in PHASE_PLAN and COMPLETION_AUDIT"
        );
    }
}

#[test]
fn generated_vscode_packages_are_not_tracked() {
    let output = std::process::Command::new("git")
        .args(["ls-files", "*.vsix"])
        .current_dir(repo_root())
        .output()
        .expect("list tracked VSIX packages");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tracked = String::from_utf8(output.stdout).expect("git ls-files output is utf-8");
    assert!(
        tracked.trim().is_empty(),
        "generated VS Code packages should not be checked in:\n{tracked}"
    );

    let gitignore = read(".gitignore");
    assert!(
        gitignore.lines().any(|line| line.trim() == "*.vsix"),
        ".gitignore should keep generated VSIX packages out of source control"
    );
}

#[test]
fn production_rust_files_stay_below_cleanup_threshold() {
    const MAX_LINES: usize = 500;

    let output = std::process::Command::new("git")
        .args(["ls-files", "*.rs"])
        .current_dir(repo_root())
        .output()
        .expect("list tracked Rust files");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let paths = String::from_utf8(output.stdout).expect("git ls-files output is utf-8");
    assert!(!paths.trim().is_empty(), "expected tracked Rust files");

    for path in paths.lines() {
        let line_count = read(path).lines().count();
        assert!(
            line_count <= MAX_LINES,
            "{path} has {line_count} lines; split focused helpers before growing past {MAX_LINES}"
        );
    }
}

#[test]
fn source_ast_no_longer_has_return_expression_nodes() {
    for path in [
        "src/ast/expressions.rs",
        "src/ast/typed.rs",
        "src/typechecker/expressions.rs",
        "src/typechecker/expressions/simple_forms.rs",
        "src/codegen/c/emit.rs",
        "src/codegen/c/types.rs",
    ] {
        let source = read(path);
        for forbidden in [
            "Expression::Return",
            "TypedExprKind::Return",
            "check_return_expr",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} still contains dead return-expression support: {forbidden}"
            );
        }
    }
}

#[test]
fn central_stdlib_core_modules_do_not_use_removed_return_keyword() {
    for path in [
        "stdlib/core/option.zen",
        "stdlib/core/result.zen",
        "stdlib/core/propagate.zen",
        "stdlib/core/buffer.zen",
        "stdlib/core/iterator.zen",
        "stdlib/core/ptr.zen",
    ] {
        let source = read(path);
        assert!(
            !source.contains("return "),
            "{path} still uses the removed return keyword"
        );
    }
}

#[test]
fn source_ast_does_not_carry_dead_char_literal_nodes() {
    for path in [
        "src/ast/expressions.rs",
        "src/typechecker/expressions.rs",
        "src/resolver/expression_validation.rs",
        "src/build_graph/lowering.rs",
        "src/typechecker/self_type_validation/expressions.rs",
        "src/typechecker/generic_type_reference_walker/expressions.rs",
        "src/typechecker/resolver_validation/local_traversal.rs",
        "src/typechecker/resolver_validation_support/expected_local_traversal.rs",
    ] {
        let source = read(path);
        for forbidden in ["CharLiteral", "TODO: implement char literal type"] {
            assert!(
                !source.contains(forbidden),
                "{path} still contains dead char-literal AST support: {forbidden}"
            );
        }
    }
}

#[test]
fn parser_type_declaration_suffixes_use_owned_keyword_enum() {
    let source = read("src/parser/declarations.rs");

    for forbidden in [
        r#"method_name == "impl""#,
        r#"method_name == "implements""#,
        r#"method_name == "requires""#,
        r#"method_name == "extends""#,
        r#"matches!(method_name.as_str(), "implements" | "requires" | "extends")"#,
    ] {
        assert!(
            !source.contains(forbidden),
            "parser type declaration suffix dispatch should use TypeDeclarationKeyword, not raw spelling checks: {forbidden}"
        );
    }
    assert!(
        source.contains("TypeDeclarationKeyword"),
        "parser type declaration suffix dispatch should use TypeDeclarationKeyword"
    );
}

#[test]
fn parser_loop_control_calls_use_owned_action_enum() {
    for path in [
        "src/parser/expressions.rs",
        "src/parser/expressions/suffixes.rs",
    ] {
        let source = read(path);
        for forbidden in [
            r#"name.as_str() == "done""#,
            r#"name.as_str() == "next""#,
            r#"match name.as_str()"#,
            r#""done" => Expression::LoopControl"#,
            r#""next" => Expression::LoopControl"#,
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should parse loop control calls through LoopControlAction, not raw spelling checks: {forbidden}"
            );
        }
    }

    let suffixes = read("src/parser/expressions/suffixes.rs");
    assert!(
        suffixes.contains("name.parse::<LoopControlAction>()"),
        "parser loop-control suffix handling should parse through LoopControlAction"
    );
}

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
