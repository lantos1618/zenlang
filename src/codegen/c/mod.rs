mod closures;
mod emit;
mod intrinsics;
mod matches;
mod strings;
mod types;

use crate::ast::typed::*;
use crate::codegen::Backend;

/// C code generation backend.
pub struct CBackend;

impl Backend for CBackend {
    fn generate(&self, program: &TypedProgram) -> Result<String, String> {
        let mut emitter = CEmitter::new();
        emitter.emit_program(program);
        Ok(emitter.output)
    }
}

// ── Emitter ───────────────────────────────────────────────────

struct CEmitter {
    output: String,
    indent: usize,
    /// Counter for temporary variable names.
    tmp_counter: usize,
    /// Collected closure definitions (env struct + function) to emit before main functions.
    closure_defs: Vec<String>,
    /// Function-scope defers that must run before any return emitted in this body.
    current_defers: Vec<TypedExpression>,
}

impl CEmitter {
    fn new() -> Self {
        Self {
            output: String::with_capacity(4096),
            indent: 0,
            tmp_counter: 0,
            closure_defs: Vec::new(),
            current_defers: Vec::new(),
        }
    }

    fn fresh_tmp(&mut self) -> String {
        let n = self.tmp_counter;
        self.tmp_counter += 1;
        format!("__tmp{}", n)
    }

    // ── Output helpers ────────────────────────────────────────

    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn blank(&mut self) {
        self.output.push('\n');
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }
}

// ── Helpers ───────────────────────────────────────────────────

/// C reserved keywords that must be escaped.
fn is_c_keyword(s: &str) -> bool {
    matches!(
        s,
        "auto"
            | "break"
            | "case"
            | "char"
            | "const"
            | "continue"
            | "default"
            | "do"
            | "double"
            | "else"
            | "enum"
            | "extern"
            | "float"
            | "for"
            | "goto"
            | "if"
            | "int"
            | "long"
            | "register"
            | "return"
            | "short"
            | "signed"
            | "sizeof"
            | "static"
            | "struct"
            | "switch"
            | "typedef"
            | "union"
            | "unsigned"
            | "void"
            | "volatile"
            | "while"
            | "inline"
            | "restrict"
            | "_Bool"
            | "_Complex"
            | "_Imaginary"
    )
}

/// Make a Zen function name safe for C (also renames `main` → `zen_main`).
fn c_func_ident(name: &str) -> String {
    if name == "main" {
        return "zen_main".into();
    }
    c_ident(name)
}

/// Make a Zen identifier safe for C.
fn c_ident(name: &str) -> String {
    let ident = name
        .replace(['.', '<'], "_")
        .replace('>', "")
        .replace(',', "_")
        .replace(' ', "")
        .replace('@', "_");
    if is_c_keyword(&ident) {
        format!("zen_{}", ident)
    } else {
        ident
    }
}

/// Escape a string for inclusion in a C string literal.
fn c_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c => out.push(c),
        }
    }
    out
}

/// Format a float for C source code.
fn format_float(v: f64) -> String {
    let s = format!("{}", v);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{}.0", s)
    }
}

#[cfg(test)]
mod tests;
