use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;
use zen::formatting::{format_braces, format_enum_variants, fix_indentation, normalize_variable_declarations, remove_trailing_whitespace};
use zen::lexer::Lexer;
use zen::parser::Parser;

#[derive(ClapParser)]
#[clap(name = "zen-format")]
#[clap(about = "Zen language formatter and style checker", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// File to format or check (if no subcommand specified, defaults to check)
    file: Option<PathBuf>,

    /// Check mode - don't modify files, just report issues
    #[clap(short, long)]
    check: bool,

    /// Fix issues automatically
    #[clap(short, long)]
    fix: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check file syntax and style
    Check {
        /// File to check
        file: PathBuf,
    },
    /// Format file according to Zen style guide
    Format {
        /// File to format
        file: PathBuf,

        /// Write output to stdout instead of modifying file
        #[clap(short, long)]
        stdout: bool,
    },
    /// Validate imports are at module level
    ValidateImports {
        /// File to validate
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Check { file }) => check_file(&file),
        Some(Commands::Format { file, stdout }) => format_file(&file, stdout),
        Some(Commands::ValidateImports { file }) => validate_imports(&file),
        None => {
            // Default behavior when just a file is provided
            if let Some(file) = cli.file {
                if cli.fix {
                    format_file(&file, false)
                } else {
                    check_file(&file)
                }
            } else {
                eprintln!("Error: Please provide a file to check or format");
                std::process::exit(1);
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn check_file(path: &PathBuf) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    println!("Checking {}...", path.display());

    // Parse the file
    let lexer = Lexer::new(&content);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(_program) => {
            // Check for style issues
            let mut issues = Vec::new();

            // Check for imports in comptime blocks
            if has_comptime_imports(&content) {
                issues
                    .push("Found imports inside comptime blocks - imports must be at module level");
            }

            // Check for proper indentation
            if has_inconsistent_indentation(&content) {
                issues.push("Inconsistent indentation detected");
            }

            // Check for trailing whitespace
            if has_trailing_whitespace(&content) {
                issues.push("Trailing whitespace detected");
            }

            if issues.is_empty() {
                println!("✓ No issues found!");
                Ok(())
            } else {
                println!("Found {} issue(s):", issues.len());
                for issue in issues {
                    println!("  - {}", issue);
                }
                Err("Style issues found".to_string())
            }
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

fn format_file(path: &PathBuf, stdout: bool) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Apply text-based fixes BEFORE parsing (to fix syntax that won't parse otherwise)
    // Normalize variable declarations (fix syntax like `i: i32 ::= 0` → `i :: i32 = 0`)
    let pre_normalized = normalize_variable_declarations(&content);

    // Parse the normalized file
    let lexer = Lexer::new(&pre_normalized);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(_program) => {
            // Apply additional formatting fixes
            let mut formatted = pre_normalized.clone();

            // Fix trailing whitespace
            formatted = remove_trailing_whitespace(&formatted);

            // Fix indentation (tabs to spaces)
            formatted = fix_indentation(&formatted);

            // Format braces and pattern matching
            formatted = format_braces(&formatted);

            // Format enum variants on separate lines (must be after braces)
            formatted = format_enum_variants(&formatted);

            // Move imports out of comptime if needed
            formatted = fix_comptime_imports(&formatted);

            if stdout {
                print!("{}", formatted);
            } else if formatted != content {
                fs::write(path, formatted)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
                println!("✓ Formatted {}", path.display());
            } else {
                println!("✓ {} is already formatted", path.display());
            }
            Ok(())
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

fn validate_imports(path: &PathBuf) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    println!("Validating imports in {}...", path.display());

    if has_comptime_imports(&content) {
        println!("✗ Found imports inside comptime blocks!");
        println!("  Imports must be at module level, not inside comptime blocks.");
        println!("  Use: identifier := @std.module");
        println!("  Not: comptime {{ identifier := @std.module }}");
        Err("Import validation failed".to_string())
    } else {
        println!("✓ All imports are correctly placed at module level");
        Ok(())
    }
}

fn has_comptime_imports(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_comptime = false;
    let mut brace_depth = 0;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("comptime") && trimmed.contains('{') {
            in_comptime = true;
            brace_depth = 1;
        } else if in_comptime {
            // Count braces
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        in_comptime = false;
                    }
                }
            }

            // Check for imports while in comptime
            if in_comptime
                && (trimmed.contains("@std.")
                    || trimmed.contains("@compiler.")
                    || (trimmed.contains("build.import") && trimmed.contains(":=")))
            {
                return true;
            }
        }
    }

    false
}

fn has_inconsistent_indentation(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let mut uses_tabs = false;
    let mut uses_spaces = false;

    for line in lines {
        if line.starts_with('\t') {
            uses_tabs = true;
        }
        if line.starts_with(' ') {
            uses_spaces = true;
        }
    }

    // Inconsistent if using both tabs and spaces
    uses_tabs && uses_spaces
}

fn has_trailing_whitespace(content: &str) -> bool {
    content
        .lines()
        .any(|line| line.ends_with(' ') || line.ends_with('\t'))
}

fn fix_comptime_imports(content: &str) -> String {
    // This is a simplified version - a full implementation would need proper AST manipulation
    // For now, just return the content as-is if no comptime imports are found
    if !has_comptime_imports(content) {
        return content.to_string();
    }

    // AST-based refactoring for comptime imports not implemented
    println!("Warning: Automatic fixing of comptime imports not yet implemented.");
    println!("Please manually move imports to module level.");
    content.to_string()
}

