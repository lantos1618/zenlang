use std::path::Path;
use std::process;

pub(super) fn is_build_zen_path(path_str: &str) -> bool {
    Path::new(path_str)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "build.zen")
}

pub(super) fn reject_build_zen_for_emit_json_mode(path_str: &str) {
    if is_build_zen_path(path_str) {
        eprintln!(
            "error: this emit-json mode does not support build.zen; use `emit-json build-graph`"
        );
        process::exit(1);
    }
}

#[derive(Clone, Copy)]
enum CompilerOwnedJsonBoundary {
    Ast,
    Typed,
    Symbols,
    Diagnostics,
    BuildGraph,
    Hir,
    Mir,
    Layout,
}

impl CompilerOwnedJsonBoundary {
    fn rejection_message(self) -> &'static str {
        match self {
            Self::Ast => "compiler-owned AST JSON emission rejects hand-authored JSON IR before it can override unchecked syntax trees",
            Self::Typed => "compiler-owned typed JSON emission rejects hand-authored JSON IR before it can override checked types or layouts",
            Self::Symbols => "compiler-owned symbols JSON emission rejects hand-authored resolver IR before it can override symbol metadata",
            Self::Diagnostics => "compiler-owned diagnostics JSON emission rejects hand-authored diagnostic IR before it can override compiler diagnostics",
            Self::BuildGraph => "compiler-owned build graph JSON emission rejects hand-authored graph IR before it can override deterministic build metadata",
            Self::Hir => "compiler-owned IR schemas reject hand-authored HIR JSON before it can override checked types or layouts",
            Self::Mir => "compiler-owned IR schemas reject hand-authored MIR JSON before it can override checked types or layouts",
            Self::Layout => "compiler-owned layout schemas reject hand-authored layout IR before it can override compiler-owned types or layouts",
        }
    }
}

fn reject_hand_authored_json_for_emit(path_str: &str, boundary: CompilerOwnedJsonBoundary) {
    if has_json_extension(path_str) {
        eprintln!("error: {}", boundary.rejection_message());
        process::exit(1);
    }
}

pub(super) fn reject_hand_authored_json_for_ast_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Ast);
}

pub(super) fn reject_hand_authored_json_for_typed_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Typed);
}

pub(super) fn reject_hand_authored_json_for_symbols_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Symbols);
}

pub(super) fn reject_hand_authored_json_for_diagnostics_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Diagnostics);
}

pub(super) fn reject_hand_authored_json_for_build_graph_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::BuildGraph);
}

pub(super) fn reject_hand_authored_json_for_hir_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Hir);
}

pub(super) fn reject_hand_authored_json_for_mir_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Mir);
}

pub(super) fn reject_hand_authored_json_for_layout_emit(path_str: &str) {
    reject_hand_authored_json_for_emit(path_str, CompilerOwnedJsonBoundary::Layout);
}

fn has_json_extension(path_str: &str) -> bool {
    Path::new(path_str)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}
