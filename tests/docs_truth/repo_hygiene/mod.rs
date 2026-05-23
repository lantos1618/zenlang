use super::*;

fn owns_static_spelling_from_str(source: &str, enum_name: &str) -> bool {
    source.contains(&format!("impl FromStr for {enum_name}"))
        || source.contains(&format!("impl_static_spelling_from_str!({enum_name},"))
        || source.contains(&format!(
            "impl_static_spelling_from_str!(\n    {enum_name},"
        ))
}

fn owns_static_spelling_display(source: &str, enum_name: &str) -> bool {
    source.contains(&format!("impl fmt::Display for {enum_name}"))
        || source.contains(&format!("impl_static_spelling_display!({enum_name},"))
        || source.contains(&format!("impl_static_spelling_display!(\n    {enum_name},"))
}

fn uses_static_spelling_parser(source: &str) -> bool {
    source.contains("crate::static_spelling::parse_static_spelling(")
        || source.contains("crate::static_spelling::parse_static_spelling_table(")
        || source.contains("crate::static_spelling::impl_static_spelling_from_str!")
}

mod ast_expression_operators;
mod build_graph_dsl;
mod builtin_type_spelling;
mod ci_configs;
mod codegen_c;
mod error_diagnostics;
mod file_size;
mod frontend_diagnostics;
mod ir_json;
mod module_system;
mod parser_behavior_declarations;
mod parser_core;
mod parser_declarations;
mod parser_enums;
mod parser_function_forms;
mod parser_keywords;
mod removed_syntax;
mod resolver_expression_validation;
mod resolver_metadata_helpers;
mod resolver_symbol_table;
mod resolver_validation;
mod semantic_overlap;
mod source_truth;
mod stdlib_boundaries;
mod typechecker_aggregate_constructors;
mod typechecker_behavior_impls;
mod typechecker_call_validation;
mod typechecker_closures;
mod typechecker_declaration_collection;
mod typechecker_generic_type_references;
mod typechecker_imports;
mod typechecker_patterns;
mod typechecker_program_checking;
mod typechecker_resolve;
mod typechecker_resolver_validation;
mod typechecker_runtime;
mod typechecker_semantic_validation;
mod typechecker_state;
