use std::env;
use std::fs;
use std::process;
use zen::lexer::Lexer;
use zen::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: zen-check <file.zen> [files...]");
        eprintln!("       zen-check *.zen");
        process::exit(1);
    }

    let mut has_errors = false;
    let mut total_files = 0;
    let mut files_with_errors = 0;

    for file_path in &args[1..] {
        // Handle glob patterns
        let paths = if file_path.contains('*') {
            match glob::glob(file_path) {
                Ok(paths) => paths.filter_map(Result::ok).collect::<Vec<_>>(),
                Err(e) => {
                    eprintln!("Error: Invalid glob pattern '{}': {}", file_path, e);
                    process::exit(1);
                }
            }
        } else {
            vec![std::path::PathBuf::from(file_path)]
        };

        for path in paths {
            total_files += 1;
            let file_str = path.to_string_lossy();

            match fs::read_to_string(&path) {
                Ok(content) => {
                    println!("Checking {}...", file_str);

                    // Check for imports in comptime blocks (which are not allowed)
                    if let Some(error_msg) = check_import_placement(&content) {
                        eprintln!("  ✗ Import error: {}", error_msg);
                        has_errors = true;
                        files_with_errors += 1;
                        continue;
                    }

                    // Try to parse the file
                    let lexer = Lexer::new(&content);
                    let mut parser = Parser::new(lexer);

                    match parser.parse_program() {
                        Ok(_) => {
                            println!("  ✓ No syntax errors found");
                        }
                        Err(e) => {
                            eprintln!("  ✗ Syntax error: {}", e);
                            has_errors = true;
                            files_with_errors += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  ✗ Error reading file: {}", e);
                    has_errors = true;
                    files_with_errors += 1;
                }
            }
        }
    }

    // Print summary if checking multiple files
    if total_files > 1 {
        println!("\n=== Summary ===");
        println!("Files checked: {}", total_files);
        println!("Files with errors: {}", files_with_errors);
        println!("Files without errors: {}", total_files - files_with_errors);
    }

    process::exit(if has_errors {
        1
    } else {
        0
    });
}

/// Check for imports inside comptime blocks
/// Imports are NOT allowed in comptime blocks - they must be at module level
fn check_import_placement(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_comptime = false;
    let mut comptime_line = 0;
    let mut brace_depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Track entering comptime blocks
        if trimmed.starts_with("comptime") && trimmed.contains('{') {
            in_comptime = true;
            comptime_line = i + 1;
            brace_depth = 1;
        } else if in_comptime {
            // Track brace depth in comptime blocks
            for ch in line.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            in_comptime = false;
                        }
                    }
                    _ => {}
                }
            }

            // Check for imports in comptime blocks
            if in_comptime && (trimmed.contains("@std") || trimmed.contains(".import(")) {
                return Some(format!(
                    "Line {}: Import statement found inside comptime block (started at line {}). \
                     Imports must be at module level, not inside comptime blocks.",
                    i + 1,
                    comptime_line
                ));
            }
        }
    }

    None
}
