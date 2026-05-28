use std::process;

use zen::error::FileTable;

pub(super) fn load_build_graph(path_str: &str) -> zen::build_graph::BuildGraph {
    let path = super::require_existing_path(path_str);
    if !super::is_build_zen_path(path_str) {
        eprintln!("error: emit-json build-graph expects a build.zen file");
        process::exit(1);
    }

    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("error reading {}: {}", path_str, err);
            process::exit(1);
        }
    };

    let mut files = FileTable::default();
    let file_id = files.add_file(path_str.to_string(), &source);
    let tokens = match zen::lexer::tokenize(&source, file_id) {
        Ok(tokens) => tokens,
        Err(err) => {
            super::print_errors(&[err], &files);
            process::exit(1);
        }
    };
    let program = match zen::parser::parse(tokens, file_id) {
        Ok(program) => program,
        Err(errs) => {
            super::print_errors(&errs, &files);
            process::exit(1);
        }
    };
    match zen::build_graph::BuildGraph::from_build_program(&program) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("build graph error: {}", err);
            process::exit(1);
        }
    }
}
