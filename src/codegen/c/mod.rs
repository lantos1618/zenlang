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

    // ── Type mapping tests ───────────────────────────────────

    #[test]
    fn c_type_primitives() {
        let e = CEmitter::new();
        assert_eq!(e.c_type(&Type::I8), "int8_t");
        assert_eq!(e.c_type(&Type::I16), "int16_t");
        assert_eq!(e.c_type(&Type::I32), "int32_t");
        assert_eq!(e.c_type(&Type::I64), "int64_t");
        assert_eq!(e.c_type(&Type::U8), "uint8_t");
        assert_eq!(e.c_type(&Type::U16), "uint16_t");
        assert_eq!(e.c_type(&Type::U32), "uint32_t");
        assert_eq!(e.c_type(&Type::U64), "uint64_t");
        assert_eq!(e.c_type(&Type::Usize), "size_t");
        assert_eq!(e.c_type(&Type::F32), "float");
        assert_eq!(e.c_type(&Type::F64), "double");
        assert_eq!(e.c_type(&Type::Bool), "bool");
        assert_eq!(e.c_type(&Type::Void), "void");
    }

    #[test]
    fn c_type_strings() {
        let e = CEmitter::new();
        assert_eq!(e.c_type(&Type::Str), "zen_str");
        assert_eq!(e.c_type(&Type::String), "zen_string");
    }

    #[test]
    fn c_type_pointers() {
        let e = CEmitter::new();
        assert_eq!(e.c_type(&Type::Ptr(Box::new(Type::I32))), "const int32_t*");
        assert_eq!(e.c_type(&Type::MutPtr(Box::new(Type::I32))), "int32_t*");
        assert_eq!(e.c_type(&Type::RawPtr(Box::new(Type::U8))), "uint8_t*");
        assert_eq!(e.c_type(&Type::Slice(Box::new(Type::F64))), "double*");
    }

    #[test]
    fn c_type_named_and_struct() {
        let e = CEmitter::new();
        assert_eq!(e.c_type(&Type::Named("Widget".into())), "Widget");
        assert_eq!(
            e.c_type(&Type::Struct {
                name: "Point".into(),
                fields: vec![],
            }),
            "Point"
        );
        assert_eq!(
            e.c_type(&Type::Enum {
                name: "Color".into(),
                variants: vec![],
            }),
            "Color"
        );
    }

    #[test]
    fn c_type_function_pointer() {
        let e = CEmitter::new();
        assert_eq!(
            e.c_type(&Type::Function {
                params: vec![Type::I32, Type::I32],
                ret: Box::new(Type::Bool),
            }),
            "bool(*)(int32_t, int32_t)"
        );
    }

    // ── Expression emission tests ────────────────────────────

    fn dummy() -> crate::error::Span {
        crate::error::Span::dummy()
    }

    fn texpr(kind: TypedExprKind, ty: Type) -> TypedExpression {
        TypedExpression {
            kind,
            ty,
            span: dummy(),
        }
    }

    #[test]
    fn emit_int_literal() {
        let mut e = CEmitter::new();
        let expr = texpr(TypedExprKind::IntLiteral(42), Type::I32);
        assert_eq!(e.emit_expr_inline(&expr), "42LL");
    }

    #[test]
    fn emit_float_literal() {
        let mut e = CEmitter::new();
        let expr = texpr(TypedExprKind::FloatLiteral(3.14), Type::F64);
        assert_eq!(e.emit_expr_inline(&expr), "3.14");
    }

    #[test]
    fn emit_bool_literal() {
        let mut e = CEmitter::new();
        assert_eq!(
            e.emit_expr_inline(&texpr(TypedExprKind::BoolLiteral(true), Type::Bool)),
            "true"
        );
        assert_eq!(
            e.emit_expr_inline(&texpr(TypedExprKind::BoolLiteral(false), Type::Bool)),
            "false"
        );
    }

    #[test]
    fn emit_string_literal() {
        let mut e = CEmitter::new();
        let expr = texpr(TypedExprKind::StringLiteral("hi".into()), Type::Str);
        assert_eq!(e.emit_expr_inline(&expr), "zen_str_from_literal(\"hi\")");
    }

    #[test]
    fn emit_variable() {
        let mut e = CEmitter::new();
        let expr = texpr(TypedExprKind::Variable("count".into()), Type::I32);
        assert_eq!(e.emit_expr_inline(&expr), "count");
    }

    #[test]
    fn emit_binary_ops() {
        let mut e = CEmitter::new();
        let left = Box::new(texpr(TypedExprKind::Variable("x".into()), Type::I32));
        let right = Box::new(texpr(TypedExprKind::IntLiteral(1), Type::I32));
        let expr = texpr(
            TypedExprKind::BinaryOp {
                op: BinaryOp::Sub,
                left,
                right,
            },
            Type::I32,
        );
        assert_eq!(e.emit_expr_inline(&expr), "(x - 1LL)");
    }

    #[test]
    fn emit_unary_ops() {
        use crate::ast::expressions::UnaryOp;
        let mut e = CEmitter::new();

        let operand = Box::new(texpr(TypedExprKind::Variable("x".into()), Type::I32));
        let neg = texpr(
            TypedExprKind::UnaryOp {
                op: UnaryOp::Neg,
                operand,
            },
            Type::I32,
        );
        assert_eq!(e.emit_expr_inline(&neg), "(-x)");

        let operand = Box::new(texpr(TypedExprKind::Variable("b".into()), Type::Bool));
        let not = texpr(
            TypedExprKind::UnaryOp {
                op: UnaryOp::Not,
                operand,
            },
            Type::Bool,
        );
        assert_eq!(e.emit_expr_inline(&not), "(!b)");
    }

    #[test]
    fn emit_function_call() {
        let mut e = CEmitter::new();
        let args = vec![
            texpr(TypedExprKind::IntLiteral(1), Type::I32),
            texpr(TypedExprKind::IntLiteral(2), Type::I32),
        ];
        let expr = texpr(
            TypedExprKind::FunctionCall {
                function: "add".into(),
                args,
            },
            Type::I32,
        );
        assert_eq!(e.emit_expr_inline(&expr), "add(1LL, 2LL)");
    }

    #[test]
    fn emit_field_access() {
        let mut e = CEmitter::new();
        let obj = Box::new(texpr(
            TypedExprKind::Variable("p".into()),
            Type::Struct {
                name: "Point".into(),
                fields: vec![],
            },
        ));
        let expr = texpr(
            TypedExprKind::FieldAccess {
                object: obj,
                field: "x".into(),
            },
            Type::I32,
        );
        assert_eq!(e.emit_expr_inline(&expr), "p.x");
    }

    #[test]
    fn emit_field_access_through_ptr() {
        let mut e = CEmitter::new();
        let obj = Box::new(texpr(
            TypedExprKind::Variable("p".into()),
            Type::Ptr(Box::new(Type::Struct {
                name: "Point".into(),
                fields: vec![],
            })),
        ));
        let expr = texpr(
            TypedExprKind::FieldAccess {
                object: obj,
                field: "x".into(),
            },
            Type::I32,
        );
        assert_eq!(e.emit_expr_inline(&expr), "p->x");
    }

    #[test]
    fn emit_cast() {
        let mut e = CEmitter::new();
        let inner = Box::new(texpr(TypedExprKind::Variable("x".into()), Type::I32));
        let expr = texpr(
            TypedExprKind::Cast {
                expr: inner,
                to_type: Type::I64,
                from_type: Type::I32,
            },
            Type::I64,
        );
        assert_eq!(e.emit_expr_inline(&expr), "((int64_t)x)");
    }

    #[test]
    fn emit_ref_deref() {
        let mut e = CEmitter::new();
        let var = texpr(TypedExprKind::Variable("x".into()), Type::I32);
        let r = texpr(
            TypedExprKind::Ref(Box::new(var.clone())),
            Type::Ptr(Box::new(Type::I32)),
        );
        assert_eq!(e.emit_expr_inline(&r), "(&x)");

        let d = texpr(
            TypedExprKind::Deref(Box::new(texpr(
                TypedExprKind::Variable("p".into()),
                Type::Ptr(Box::new(Type::I32)),
            ))),
            Type::I32,
        );
        assert_eq!(e.emit_expr_inline(&d), "(*p)");
    }

    #[test]
    fn emit_index_access() {
        let mut e = CEmitter::new();
        let obj = Box::new(texpr(
            TypedExprKind::Variable("arr".into()),
            Type::Slice(Box::new(Type::I32)),
        ));
        let idx = Box::new(texpr(TypedExprKind::IntLiteral(3), Type::I32));
        let expr = texpr(
            TypedExprKind::IndexAccess {
                object: obj,
                index: idx,
            },
            Type::I32,
        );
        assert_eq!(e.emit_expr_inline(&expr), "arr[3LL]");
    }

    #[test]
    fn emit_struct_literal() {
        let mut e = CEmitter::new();
        let expr = texpr(
            TypedExprKind::StructLiteral {
                type_name: "Point".into(),
                fields: vec![
                    ("x".into(), texpr(TypedExprKind::IntLiteral(1), Type::I32)),
                    ("y".into(), texpr(TypedExprKind::IntLiteral(2), Type::I32)),
                ],
            },
            Type::Struct {
                name: "Point".into(),
                fields: vec![],
            },
        );
        assert_eq!(e.emit_expr_inline(&expr), "(Point){ .x = 1LL, .y = 2LL }");
    }

    #[test]
    fn emit_var_decl_mutable_vs_const() {
        let mut e = CEmitter::new();
        let stmt_const = TypedStatement {
            kind: TypedStatementKind::VarDecl {
                name: "x".into(),
                ty: Type::I32,
                value: texpr(TypedExprKind::IntLiteral(42), Type::I32),
                mutable: false,
            },
            span: dummy(),
        };
        e.emit_statement(&stmt_const);
        assert!(e.output.contains("const int32_t x = 42LL;"));

        let mut e = CEmitter::new();
        let stmt_mut = TypedStatement {
            kind: TypedStatementKind::VarDecl {
                name: "y".into(),
                ty: Type::I32,
                value: texpr(TypedExprKind::IntLiteral(0), Type::I32),
                mutable: true,
            },
            span: dummy(),
        };
        e.emit_statement(&stmt_mut);
        assert!(e.output.contains("int32_t y = 0LL;"));
        assert!(!e.output.contains("const"));
    }

    // ── Helper tests ─────────────────────────────────────────

    #[test]
    fn c_keyword_escaping() {
        assert_eq!(c_ident("int"), "zen_int");
        assert_eq!(c_ident("return"), "zen_return");
        assert_eq!(c_ident("void"), "zen_void");
        assert_eq!(c_ident("while"), "zen_while");
        // Non-keywords pass through
        assert_eq!(c_ident("count"), "count");
        assert_eq!(c_ident("my_var"), "my_var");
    }

    #[test]
    fn c_func_ident_renames_main() {
        assert_eq!(c_func_ident("main"), "zen_main");
        assert_eq!(c_func_ident("add"), "add");
        assert_eq!(c_func_ident("process"), "process");
    }

    #[test]
    fn format_float_values() {
        assert_eq!(format_float(3.14), "3.14");
        assert_eq!(format_float(0.0), "0.0");
        assert_eq!(format_float(1.0), "1.0");
        assert_eq!(format_float(100.0), "100.0");
    }

    #[test]
    fn fresh_tmp_increments() {
        let mut e = CEmitter::new();
        assert_eq!(e.fresh_tmp(), "__tmp0");
        assert_eq!(e.fresh_tmp(), "__tmp1");
        assert_eq!(e.fresh_tmp(), "__tmp2");
    }

    #[test]
    fn emit_return_statement() {
        let mut e = CEmitter::new();
        let ret = texpr(
            TypedExprKind::Return(Some(Box::new(texpr(
                TypedExprKind::IntLiteral(0),
                Type::I32,
            )))),
            Type::Never,
        );
        assert_eq!(e.emit_expr_to_stmt(&ret), "return 0LL;");
    }

    #[test]
    fn emit_return_void() {
        let mut e = CEmitter::new();
        let ret = texpr(TypedExprKind::Return(None), Type::Never);
        assert_eq!(e.emit_expr_to_stmt(&ret), "return;");
    }

    #[test]
    fn emit_break_continue() {
        let mut e = CEmitter::new();
        assert_eq!(
            e.emit_expr_to_stmt(&texpr(TypedExprKind::Break, Type::Never)),
            "break;"
        );
        assert_eq!(
            e.emit_expr_to_stmt(&texpr(TypedExprKind::Continue, Type::Never)),
            "continue;"
        );
    }

    #[test]
    fn generates_enum_with_payload() {
        let backend = CBackend;
        let program = TypedProgram {
            functions: vec![],
            types: vec![TypedTypeDef {
                name: "Shape".into(),
                kind: TypeDefKind::Enum {
                    variants: vec![
                        TypedVariant {
                            name: "Circle".into(),
                            tag: 0,
                            payload: Some(vec![("radius".into(), Type::I32)]),
                        },
                        TypedVariant {
                            name: "Square".into(),
                            tag: 1,
                            payload: Some(vec![("side".into(), Type::I32)]),
                        },
                    ],
                },
                methods: vec![],
                span: dummy(),
            }],
            globals: vec![],
            entry_point: None,
        };
        let output = backend.generate(&program).unwrap();
        assert!(output.contains("Shape_Circle = 0"));
        assert!(output.contains("Shape_Square = 1"));
        assert!(output.contains("int32_t circle;"));
        assert!(output.contains("int32_t square;"));
    }

    #[test]
    fn generates_function_with_defers() {
        let backend = CBackend;
        let program = TypedProgram {
            functions: vec![TypedFunction {
                name: "process".into(),
                params: vec![],
                return_type: Type::I32,
                body: TypedBlock {
                    statements: vec![],
                    expr: Some(Box::new(texpr(
                        TypedExprKind::Return(Some(Box::new(texpr(
                            TypedExprKind::IntLiteral(42),
                            Type::I32,
                        )))),
                        Type::Never,
                    ))),
                    ty: Type::I32,
                    span: dummy(),
                },
                defers: vec![texpr(
                    TypedExprKind::FunctionCall {
                        function: "cleanup".into(),
                        args: vec![],
                    },
                    Type::Void,
                )],
                span: dummy(),
            }],
            types: vec![],
            globals: vec![],
            entry_point: None,
        };
        let output = backend.generate(&program).unwrap();
        // Defer should emit cleanup before return
        assert!(output.contains("cleanup()"));
        // Should save return value to temp, run defers, then return temp
        assert!(output.contains("__tmp"));
    }
}
