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

const EMIT_JSON_MODE_SPELLINGS: &[(EmitJsonMode, &str)] = &[
    (EmitJsonMode::Ast, "ast"),
    (EmitJsonMode::Symbols, "symbols"),
    (EmitJsonMode::Typed, "typed"),
    (EmitJsonMode::Diagnostics, "diagnostics"),
    (EmitJsonMode::BuildGraph, "build-graph"),
    (EmitJsonMode::Hir, "hir"),
    (EmitJsonMode::Mir, "mir"),
    (EmitJsonMode::Layout, "layout"),
    (EmitJsonMode::TargetYaml, "target-yaml"),
];

impl EmitJsonMode {
    fn usage() -> String {
        EMIT_JSON_MODE_SPELLINGS
            .iter()
            .map(|(_, spelling)| *spelling)
            .collect::<Vec<_>>()
            .join("|")
    }
}

impl FromStr for EmitJsonMode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        EMIT_JSON_MODE_SPELLINGS
            .iter()
            .find_map(|(mode, spelling)| (*spelling == value).then_some(*mode))
            .ok_or(())
    }
}

pub(super) fn emit_json_usage() -> String {
    format!(
        "Usage: zen emit-json <{}> <file.zen>",
        EmitJsonMode::usage()
    )
}
