use super::*;

impl CEmitter {
    // ── String interpolation ──────────────────────────────────

    pub(super) fn emit_string_interpolation(&mut self, parts: &[TypedStringPart]) -> String {
        // Strategy: build a format string and args for snprintf, then use zen_str
        // For simplicity, use stack buffer concatenation
        if parts.len() == 1 {
            match &parts[0] {
                TypedStringPart::Literal(s) => {
                    let escaped = c_escape_string(s);
                    return c_static_str_literal(&escaped);
                }
                TypedStringPart::Expr(e) => {
                    return self.emit_to_str(e);
                }
            }
        }

        // Multiple parts: build format string + args for snprintf
        let mut format_parts = Vec::new();
        let mut arg_exprs = Vec::new();

        for part in parts {
            match part {
                TypedStringPart::Literal(s) => {
                    format_parts.push(c_escape_string(s));
                }
                TypedStringPart::Expr(e) => {
                    let (fmt, arg) = self.emit_printf_arg(e);
                    format_parts.push(fmt);
                    arg_exprs.push(arg);
                }
            }
        }

        let fmt_str = format_parts.join("");
        let buf = self.fresh_tmp();
        self.line(&format!("char {}[1024];", buf));
        if arg_exprs.is_empty() {
            self.line(&format!("snprintf({buf}, sizeof({buf}), \"{fmt_str}\");"));
        } else {
            self.line(&format!(
                "snprintf({buf}, sizeof({buf}), \"{fmt_str}\", {});",
                arg_exprs.join(", ")
            ));
        }
        format!("zen_str_from_cstr({})", buf)
    }

    pub(super) fn emit_printf_arg(&mut self, expr: &TypedExpression) -> (String, String) {
        let val = self.emit_expr_inline(expr);
        match &expr.ty {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => {
                ("%lld".into(), format!("(long long)({})", val))
            }
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Usize => {
                ("%llu".into(), format!("(unsigned long long)({})", val))
            }
            Type::F32 | Type::F64 => ("%g".into(), format!("(double)({})", val)),
            Type::Bool => ("%s".into(), format!("({}) ? \"true\" : \"false\"", val)),
            Type::Str => ("%.*s".into(), format!("(int)({val}).len, ({val}).ptr")),
            Type::String => ("%.*s".into(), format!("(int)({val}).len, ({val}).ptr")),
            _ => ("%s".into(), format!("\"<{}>\"", expr.ty.display_name())),
        }
    }

    pub(super) fn emit_to_str(&mut self, expr: &TypedExpression) -> String {
        let val = self.emit_expr_inline(expr);
        match &expr.ty {
            Type::Str | Type::String => val,
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => {
                let buf = self.fresh_tmp();
                self.line(&format!("char {}[32];", buf));
                format!(
                    "zen_i64_to_str((int64_t)({}), {}, sizeof({}))",
                    val, buf, buf
                )
            }
            Type::F32 | Type::F64 => {
                let buf = self.fresh_tmp();
                self.line(&format!("char {}[64];", buf));
                format!(
                    "zen_f64_to_str((double)({}), {}, sizeof({}))",
                    val, buf, buf
                )
            }
            Type::Bool => {
                let true_str = c_static_str_literal("true");
                let false_str = c_static_str_literal("false");
                format!("(({}) ? {} : {})", val, true_str, false_str)
            }
            _ => c_static_str_literal(&format!("<{}>", expr.ty.display_name())),
        }
    }
}
