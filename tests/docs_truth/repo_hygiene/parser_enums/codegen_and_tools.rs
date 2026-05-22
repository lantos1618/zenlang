use super::*;

mod intrinsics;

#[test]
fn build_graph_host_effect_methods_parse_dsl_ident_enum() {
    let lowering = read("src/build_graph/lowering.rs");
    let host_effects = read("src/build_graph/lowering/host_effects.rs");
    let dsl = read("src/build_graph/lowering/dsl.rs");
    let source = format!("{lowering}\n{host_effects}");

    for forbidden in [
        "match method.as_str()",
        "method == BuildTargetDslIdent::Env.as_str()",
        "method == BuildTargetDslIdent::ReadFile.as_str()",
    ] {
        assert!(
            !source.contains(forbidden),
            "build graph host-effect method dispatch should parse through BuildTargetDslIdent: {forbidden}"
        );
    }
    assert!(
        source.contains("method.parse::<BuildTargetDslIdent>()"),
        "build graph host-effect method dispatch should parse method names through BuildTargetDslIdent"
    );
    assert!(
        dsl.contains("impl FromStr for BuildTargetDslIdent"),
        "BuildTargetDslIdent should own parsing for build DSL method names"
    );
}

#[test]
fn cli_emit_json_modes_use_owned_mode_enum() {
    let cli = read("src/cli.rs");
    let mode = read("src/cli/emit_json_mode.rs");

    assert!(
        !cli.contains("enum EmitJsonMode"),
        "cli.rs should keep command dispatch focused and delegate emit-json mode parsing"
    );
    assert!(
        cli.lines().count() < 260,
        "cli.rs should stay below the cleanup threshold after extracting emit-json modes"
    );
    assert!(
        mode.contains("pub(super) enum EmitJsonMode"),
        "emit-json command routing should use an owned EmitJsonMode enum"
    );
    assert!(
        cli.contains("mode.parse::<EmitJsonMode>()"),
        "emit-json command routing should parse modes through EmitJsonMode"
    );
    assert!(
        mode.contains("pub(super) fn emit_json_usage() -> String"),
        "emit-json usage should be generated from EmitJsonMode"
    );
    assert!(
        mode.contains(".find(|mode| mode.as_str() == value)"),
        "emit-json mode parsing should use the enum-owned ordered table"
    );
    assert!(
        mode.contains("pub(super) fn gate_message(self) -> Option<&'static str>"),
        "emit-json gated diagnostics should be owned by EmitJsonMode"
    );
    assert!(
        cli.contains("mode.gate_message()"),
        "emit-json command routing should read gated diagnostics from EmitJsonMode"
    );
    assert!(
        !cli.contains("<ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml>")
            && !mode
                .contains("<ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml>"),
        "emit-json usage should not duplicate the mode list as a raw string"
    );
}
