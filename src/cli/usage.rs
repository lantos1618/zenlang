pub(super) fn print_usage() {
    eprintln!("zen compiler v0.8.0");
    eprintln!("Usage: zen <command> [args]");
    eprintln!("Commands:");
    eprintln!("  check <file>   Parse and typecheck a .zen file");
    eprintln!("  build <file>   Compile a .zen file to a binary");
    eprintln!("  test <build.zen>   Compile and run deterministic test targets");
    eprintln!(
        "  build-graph <build.zen>   Compile executable targets from deterministic build graph"
    );
    eprintln!("  emit  <file>   Emit C source (no compilation)");
    eprintln!("  emit-json ast <file>   Emit unchecked AST JSON");
    eprintln!("  emit-json symbols <file>   Emit resolver symbol tables JSON");
    eprintln!("  emit-json typed <file>   Emit checked typed program JSON");
    eprintln!("  emit-json diagnostics <file>   Emit diagnostics JSON");
    eprintln!("  emit-json build-graph <build.zen>   Emit deterministic build graph JSON");
    eprintln!("  emit-json hir <file>   Gated HIR JSON");
    eprintln!("  emit-json mir <file>   Gated MIR JSON");
    eprintln!("  emit-json layout <file>   Gated type layout JSON");
    eprintln!("  emit-json target-yaml <file>   Gated target YAML validation");
    eprintln!("  <file>         Run a .zen file");
}
