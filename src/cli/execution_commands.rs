use std::process;

pub(super) fn cmd_build(path_str: &str) {
    if super::is_build_zen_path(path_str) {
        cmd_build_graph(path_str);
    } else {
        super::compile_file_to_binary(path_str, None, None);
    }
}

pub(super) fn cmd_test(path_str: &str) {
    if !super::is_build_zen_path(path_str) {
        eprintln!("error: zen test expects a build.zen file");
        process::exit(1);
    }

    for target in super::test_build_targets(path_str) {
        if let Err(err) = std::fs::create_dir_all(&target.out_dir) {
            eprintln!("error creating {}: {}", target.out_dir.display(), err);
            process::exit(1);
        }

        let bin_path = super::compile_file_to_binary(
            target
                .root_path
                .to_str()
                .unwrap_or(&target.root_source_file),
            Some(&target.out_dir),
            Some(&target.name),
        );
        let run = process::Command::new(&bin_path).status();
        match run {
            Ok(status) if status.success() => {
                println!("  test {} passed", target.name);
            }
            Ok(status) => {
                eprintln!("  test {} exited with {}", target.name, status);
                process::exit(1);
            }
            Err(err) => {
                eprintln!("  failed to run test {}: {}", target.name, err);
                process::exit(1);
            }
        }
    }
}

pub(super) fn cmd_run_file(path_str: &str) {
    if super::is_build_zen_path(path_str) {
        cmd_build_graph(path_str);
    } else {
        super::compile_file_to_binary(path_str, None, None);
    }
}

pub(super) fn cmd_build_graph(path_str: &str) {
    for target in super::executable_build_targets(path_str) {
        if let Err(err) = std::fs::create_dir_all(&target.out_dir) {
            eprintln!("error creating {}: {}", target.out_dir.display(), err);
            process::exit(1);
        }

        super::compile_file_to_binary(
            target
                .root_path
                .to_str()
                .unwrap_or(&target.root_source_file),
            Some(&target.out_dir),
            Some(&target.name),
        );
    }
}
