mod emit;
mod functions;
mod literals;
mod matches;
mod strings;
mod types;

use crate::ast::typed::*;

pub fn generate(program: &TypedProgram) -> String {
    let mut emitter = CEmitter::new();
    emitter.emit_program(program);
    emitter.output
}

struct CEmitter {
    output: String,
    indent: usize,
    tmp_counter: usize,
}

impl CEmitter {
    fn new() -> Self {
        Self {
            output: String::with_capacity(4096),
            indent: 0,
            tmp_counter: 0,
        }
    }

    fn fresh_tmp(&mut self) -> String {
        let n = self.tmp_counter;
        self.tmp_counter += 1;
        format!("__tmp{}", n)
    }

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

fn c_func_ident(name: &str) -> String {
    if name == "main" {
        "zen_main".into()
    } else {
        c_ident(name)
    }
}

fn c_const_qualifier(mutable: bool) -> &'static str {
    if mutable {
        ""
    } else {
        "const "
    }
}

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

fn c_static_str_literal(escaped_literal: &str) -> String {
    format!("(zen_str){{ .ptr = \"{escaped_literal}\", .len = sizeof(\"{escaped_literal}\") - 1 }}")
}

fn format_float(v: f64) -> String {
    let s = format!("{}", v);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{}.0", s)
    }
}
