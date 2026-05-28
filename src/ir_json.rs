use serde::Serialize;

mod diagnostics;
mod hir;
mod layout;
mod mir;

use crate::ast::behavior_ref_display;
use crate::ast::typed::TypedProgram;
use crate::ast::AstType;
use crate::ast::Program;
use crate::error::{Diagnostic, FileTable, Span};
use crate::module_system::ResolvedModuleGraph;
use crate::resolver::{BehaviorMethodTypeMetadata, BehaviorRefMetadata, Symbol};

#[derive(Serialize)]
struct AstJsonGraph<'a> {
    format: &'static str,
    schema_version: u32,
    semantic_status: &'static str,
    entry_module: u32,
    modules: Vec<AstJsonModule<'a>>,
}

#[derive(Serialize)]
struct AstJsonModule<'a> {
    id: u32,
    package_id: u32,
    canonical_path: &'a str,
    imports: Vec<AstJsonImport<'a>>,
    program: &'a Program,
}

#[derive(Serialize)]
struct AstJsonImport<'a> {
    local_name: &'a str,
    source_module: u32,
    source_symbol: &'a str,
    span: Span,
}

#[derive(Serialize)]
struct TypedJsonProgram<'a> {
    format: &'static str,
    semantic_status: &'static str,
    program: &'a TypedProgram,
}

#[derive(Serialize)]
struct SymbolsJsonGraph<'a> {
    format: &'static str,
    semantic_status: &'static str,
    entry_module: u32,
    modules: Vec<SymbolsJsonModule<'a>>,
}

#[derive(Serialize)]
struct SymbolsJsonModule<'a> {
    id: u32,
    package_id: u32,
    canonical_path: &'a str,
    symbols: Vec<SymbolJson<'a>>,
}

#[derive(Serialize)]
struct SymbolJson<'a> {
    id: u32,
    namespace: String,
    name: &'a str,
    is_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    import_source: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter_names: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter_type_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_parameter_names: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_type_names: Option<Vec<(String, String)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant_names: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant_owner_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant_payload_type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    behavior_method_signatures: Option<Vec<(String, Vec<String>, String)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    behavior_parent_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    behavior_impl_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    behavior_required_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_mutable: Option<bool>,
    scope_id: u32,
    definition_span: Span,
}

pub fn ast_graph_to_json(graph: &ResolvedModuleGraph) -> serde_json::Result<String> {
    let graph = AstJsonGraph {
        format: "zen.ast.v0",
        schema_version: 0,
        semantic_status: "unchecked",
        entry_module: graph.entry.0,
        modules: graph
            .sorted_modules()
            .into_iter()
            .map(|module| AstJsonModule {
                id: module.info.id.0,
                package_id: module.info.package_id.0,
                canonical_path: module.info.canonical_path.as_str(),
                imports: module
                    .imports
                    .iter()
                    .map(|import| AstJsonImport {
                        local_name: import.local_name.as_str(),
                        source_module: import.source_module.0,
                        source_symbol: import.source_symbol.as_str(),
                        span: import.span,
                    })
                    .collect(),
                program: &module.program,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&graph)
}

pub fn symbols_graph_to_json(graph: &ResolvedModuleGraph) -> serde_json::Result<String> {
    let graph = SymbolsJsonGraph {
        format: "zen.symbols.v0",
        semantic_status: "resolved",
        entry_module: graph.entry.0,
        modules: graph
            .sorted_modules()
            .into_iter()
            .map(|module| SymbolsJsonModule {
                id: module.info.id.0,
                package_id: module.info.package_id.0,
                canonical_path: module.info.canonical_path.as_str(),
                symbols: module.symbols.symbols().iter().map(symbol_json).collect(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&graph)
}

pub fn typed_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let graph = TypedJsonProgram {
        format: "zen.typed.v0",
        semantic_status: "checked",
        program,
    };

    serde_json::to_string_pretty(&graph)
}

pub fn hir_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    hir::program_to_json(program)
}

pub fn layout_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    layout::program_to_json(program)
}

pub fn mir_program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    mir::program_to_json(program)
}

pub fn diagnostics_to_json(
    diagnostics: &[Diagnostic],
    files: &FileTable,
) -> serde_json::Result<String> {
    diagnostics::diagnostics_to_json(diagnostics, files)
}

fn symbol_json(symbol: &Symbol) -> SymbolJson<'_> {
    SymbolJson {
        id: symbol.id.0,
        namespace: format!("{:?}", symbol.namespace),
        name: symbol.name.as_str(),
        is_public: symbol.is_public,
        import_source: symbol.import_source.as_deref(),
        parameter_names: symbol.parameter_names.as_deref(),
        parameter_type_names: symbol.parameter_types.as_deref().map(ast_type_names),
        return_type_name: symbol.return_type.as_ref().map(AstType::display_name),
        type_parameter_names: symbol.type_parameter_names.as_deref(),
        field_type_names: symbol.field_types.as_deref().map(field_type_names),
        variant_names: symbol.variant_names.as_deref(),
        variant_owner_name: symbol.variant_owner_name.as_deref(),
        variant_payload_type_name: symbol
            .variant_payload_type
            .as_ref()
            .map(AstType::display_name),
        behavior_method_signatures: symbol
            .behavior_method_types
            .as_deref()
            .map(behavior_method_signatures),
        behavior_parent_names: symbol
            .behavior_parent_refs
            .as_deref()
            .map(behavior_ref_names),
        behavior_impl_names: symbol.behavior_impl_refs.as_deref().map(behavior_ref_names),
        behavior_required_names: symbol
            .behavior_required_refs
            .as_deref()
            .map(behavior_ref_names),
        is_mutable: symbol.is_mutable,
        scope_id: symbol.scope_id,
        definition_span: symbol.definition_span,
    }
}

fn ast_type_names(types: &[AstType]) -> Vec<String> {
    types.iter().map(AstType::display_name).collect()
}

fn field_type_names(fields: &[(String, AstType)]) -> Vec<(String, String)> {
    fields
        .iter()
        .map(|(name, ty)| (name.clone(), ty.display_name()))
        .collect()
}

fn behavior_method_signatures(
    methods: &[BehaviorMethodTypeMetadata],
) -> Vec<(String, Vec<String>, String)> {
    methods
        .iter()
        .map(|method| {
            (
                method.name.clone(),
                ast_type_names(&method.parameter_types),
                method.return_type.display_name(),
            )
        })
        .collect()
}

fn behavior_ref_names(refs: &[BehaviorRefMetadata]) -> Vec<String> {
    refs.iter()
        .map(|behavior| behavior_ref_display(&behavior.name, &behavior.type_args))
        .collect()
}
