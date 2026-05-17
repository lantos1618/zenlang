use std::path::Path;

pub(super) fn cmd_check(path_str: &str) {
    if super::is_build_zen_path(path_str) {
        let graph = super::load_build_graph(path_str);
        let build_path = Path::new(path_str);
        let base_dir = build_path.parent().unwrap_or_else(|| Path::new("."));
        super::validate_build_graph_sources(base_dir, &graph);
        super::check_build_graph_sources(base_dir, &graph);
        println!("  {} build targets — ok", graph.targets().len());
        return;
    }

    let typed = super::graph_frontend(path_str);
    println!(
        "  {} functions, {} types — ok",
        typed.functions.len(),
        typed.types.len()
    );
}

pub(super) fn cmd_emit(path_str: &str) {
    if super::is_build_zen_path(path_str) {
        let target = super::single_executable_build_target(path_str);
        print!("{}", super::compile_file_to_c_source(&target.root_path));
        return;
    }

    let typed = super::graph_frontend(path_str);
    print!("{}", super::typed_program_to_c_source(&typed));
}
