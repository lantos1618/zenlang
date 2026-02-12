mod emit;
mod intrinsics;
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
}

impl CEmitter {
    fn new() -> Self {
        Self {
            output: String::with_capacity(4096),
            indent: 0,
            tmp_counter: 0,
            closure_defs: Vec::new(),
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

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expressions::BinaryOp;

    fn make_simple_program() -> TypedProgram {
        TypedProgram {
            functions: vec![TypedFunction {
                name: "add".into(),
                params: vec![
                    TypedParam {
                        name: "a".into(),
                        ty: Type::I32,
                        span: crate::error::Span::dummy(),
                    },
                    TypedParam {
                        name: "b".into(),
                        ty: Type::I32,
                        span: crate::error::Span::dummy(),
                    },
                ],
                return_type: Type::I32,
                body: TypedBlock {
                    statements: vec![],
                    expr: Some(Box::new(TypedExpression {
                        kind: TypedExprKind::Return(Some(Box::new(TypedExpression {
                            kind: TypedExprKind::BinaryOp {
                                op: BinaryOp::Add,
                                left: Box::new(TypedExpression {
                                    kind: TypedExprKind::Variable("a".into()),
                                    ty: Type::I32,
                                    span: crate::error::Span::dummy(),
                                }),
                                right: Box::new(TypedExpression {
                                    kind: TypedExprKind::Variable("b".into()),
                                    ty: Type::I32,
                                    span: crate::error::Span::dummy(),
                                }),
                            },
                            ty: Type::I32,
                            span: crate::error::Span::dummy(),
                        }))),
                        ty: Type::Never,
                        span: crate::error::Span::dummy(),
                    })),
                    ty: Type::I32,
                    span: crate::error::Span::dummy(),
                },
                defers: vec![],
                span: crate::error::Span::dummy(),
            }],
            types: vec![],
            globals: vec![],
            entry_point: None,
        }
    }

    #[test]
    fn generates_function() {
        let backend = CBackend;
        let program = make_simple_program();
        let output = backend.generate(&program).unwrap();
        assert!(output.contains("int32_t add(int32_t a, int32_t b)"));
        assert!(output.contains("return (a + b)"));
    }

    #[test]
    fn generates_struct() {
        let backend = CBackend;
        let program = TypedProgram {
            functions: vec![],
            types: vec![TypedTypeDef {
                name: "Point".into(),
                kind: TypeDefKind::Struct {
                    fields: vec![("x".into(), Type::F64), ("y".into(), Type::F64)],
                },
                methods: vec![],
                span: crate::error::Span::dummy(),
            }],
            globals: vec![],
            entry_point: None,
        };
        let output = backend.generate(&program).unwrap();
        assert!(output.contains("typedef struct Point Point;"));
        assert!(output.contains("double x;"));
        assert!(output.contains("double y;"));
    }

    #[test]
    fn generates_enum() {
        let backend = CBackend;
        let program = TypedProgram {
            functions: vec![],
            types: vec![TypedTypeDef {
                name: "Color".into(),
                kind: TypeDefKind::Enum {
                    variants: vec![
                        TypedVariant {
                            name: "Red".into(),
                            tag: 0,
                            payload: None,
                        },
                        TypedVariant {
                            name: "Green".into(),
                            tag: 1,
                            payload: None,
                        },
                        TypedVariant {
                            name: "Blue".into(),
                            tag: 2,
                            payload: None,
                        },
                    ],
                },
                methods: vec![],
                span: crate::error::Span::dummy(),
            }],
            globals: vec![],
            entry_point: None,
        };
        let output = backend.generate(&program).unwrap();
        assert!(output.contains("Color_Red = 0"));
        assert!(output.contains("Color_Green = 1"));
        assert!(output.contains("Color_Blue = 2"));
        assert!(output.contains("enum Color_Tag tag;"));
    }

    #[test]
    fn generates_entry_point() {
        let backend = CBackend;
        let program = TypedProgram {
            functions: vec![TypedFunction {
                name: "main".into(),
                params: vec![],
                return_type: Type::I32,
                body: TypedBlock {
                    statements: vec![],
                    expr: Some(Box::new(TypedExpression {
                        kind: TypedExprKind::Return(Some(Box::new(TypedExpression {
                            kind: TypedExprKind::IntLiteral(0),
                            ty: Type::I32,
                            span: crate::error::Span::dummy(),
                        }))),
                        ty: Type::Never,
                        span: crate::error::Span::dummy(),
                    })),
                    ty: Type::I32,
                    span: crate::error::Span::dummy(),
                },
                defers: vec![],
                span: crate::error::Span::dummy(),
            }],
            types: vec![],
            globals: vec![],
            entry_point: Some("main".into()),
        };
        let output = backend.generate(&program).unwrap();
        assert!(output.contains("int main(int argc, char** argv)"));
        assert!(output.contains("return zen_main()"));
    }

    #[test]
    fn c_ident_sanitization() {
        assert_eq!(c_ident("Point"), "Point");
        assert_eq!(c_ident("@std"), "_std");
        assert_eq!(c_ident("Channel<SensorReading>"), "Channel_SensorReading");
        assert_eq!(c_ident("std.io"), "std_io");
    }

    #[test]
    fn c_escape() {
        assert_eq!(c_escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(c_escape_string("say \"hi\""), "say \\\"hi\\\"");
    }
}
