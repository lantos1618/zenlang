use std::str::FromStr;

#[derive(Clone, Copy)]
pub(super) enum EmitJsonMode {
    Ast,
    Symbols,
    Typed,
    Diagnostics,
    BuildGraph,
    Hir,
    Mir,
    Layout,
    TargetYaml,
}

impl EmitJsonMode {
    const AST: &'static str = "ast";
    const SYMBOLS: &'static str = "symbols";
    const TYPED: &'static str = "typed";
    const DIAGNOSTICS: &'static str = "diagnostics";
    const BUILD_GRAPH: &'static str = "build-graph";
    const HIR: &'static str = "hir";
    const MIR: &'static str = "mir";
    const LAYOUT: &'static str = "layout";
    const TARGET_YAML: &'static str = "target-yaml";

    const ORDERED: [Self; 9] = [
        Self::Ast,
        Self::Symbols,
        Self::Typed,
        Self::Diagnostics,
        Self::BuildGraph,
        Self::Hir,
        Self::Mir,
        Self::Layout,
        Self::TargetYaml,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::Ast => Self::AST,
            Self::Symbols => Self::SYMBOLS,
            Self::Typed => Self::TYPED,
            Self::Diagnostics => Self::DIAGNOSTICS,
            Self::BuildGraph => Self::BUILD_GRAPH,
            Self::Hir => Self::HIR,
            Self::Mir => Self::MIR,
            Self::Layout => Self::LAYOUT,
            Self::TargetYaml => Self::TARGET_YAML,
        }
    }

    fn usage() -> String {
        Self::ORDERED
            .iter()
            .map(|mode| mode.as_str())
            .collect::<Vec<_>>()
            .join("|")
    }

    pub(super) fn gate_message(self) -> Option<&'static str> {
        match self {
            Self::Ast
            | Self::Symbols
            | Self::Typed
            | Self::Diagnostics
            | Self::BuildGraph
            | Self::Hir
            | Self::Mir
            | Self::Layout
            | Self::TargetYaml => None,
        }
    }
}

impl FromStr for EmitJsonMode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ORDERED
            .iter()
            .copied()
            .find(|mode| mode.as_str() == value)
            .ok_or(())
    }
}

pub(super) fn emit_json_usage() -> String {
    format!(
        "Usage: zen emit-json <{}> <file.zen>",
        EmitJsonMode::usage()
    )
}
