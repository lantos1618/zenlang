use zen::error::FileTable;
use zen::lexer;
use zen::parser;
use zen::resolver::{Namespace, Resolver, TypeParameterBoundRefMetadata};

fn parse_program(src: &str) -> zen::ast::Program {
    let mut files = FileTable::new();
    let file_id = files.add_file("test.zen".to_string(), src.to_string());
    let tokens = lexer::tokenize(src, file_id).expect("tokenize");
    parser::parse(tokens, file_id).expect("parse")
}

#[path = "resolver_phase2/behavior_relations.rs"]
mod behavior_relations;
#[path = "resolver_phase2/core_symbols.rs"]
mod core_symbols;
#[path = "resolver_phase2/enum_metadata.rs"]
mod enum_metadata;
#[path = "resolver_phase2/expr_locals.rs"]
mod expr_locals;
#[path = "resolver_phase2/generic_behavior_metadata.rs"]
mod generic_behavior_metadata;
#[path = "resolver_phase2/impls.rs"]
mod impls;
#[path = "resolver_phase2/struct_metadata.rs"]
mod struct_metadata;
#[path = "resolver_phase2/value_metadata.rs"]
mod value_metadata;
