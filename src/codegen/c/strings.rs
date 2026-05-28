use super::*;

impl CEmitter {
    pub(super) fn emit_string_interpolation(&mut self, parts: &[TypedStringPart]) -> String {
        if let [part] = parts {
            return match part {
                TypedStringPart::Literal(s) => c_static_str_literal(&c_escape_string(s)),
                TypedStringPart::Expr(e) => self.emit_to_str(e),
            };
        }

        let mut format_parts = Vec::new();
        let mut arg_exprs = Vec::new();

        for part in parts {
            match part {
                TypedStringPart::Literal(s) => format_parts.push(c_escape_string(s)),
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
        let args = (!arg_exprs.is_empty()).then(|| format!(", {}", arg_exprs.join(", ")));
        self.line(&format!(
            "snprintf({buf}, sizeof({buf}), \"{fmt_str}\"{});",
            args.as_deref().unwrap_or("")
        ));
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
            _ => ("%s".into(), format!("\"<{}>\"", expr.ty.display_name())),
        }
    }

    pub(super) fn emit_to_str(&mut self, expr: &TypedExpression) -> String {
        let val = self.emit_expr_inline(expr);
        match &expr.ty {
            Type::Str => val,
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => {
                self.emit_number_to_str(&val, "zen_i64_to_str", "int64_t", 32)
            }
            Type::F32 | Type::F64 => self.emit_number_to_str(&val, "zen_f64_to_str", "double", 64),
            Type::Bool => {
                let true_str = c_static_str_literal("true");
                let false_str = c_static_str_literal("false");
                format!("(({}) ? {} : {})", val, true_str, false_str)
            }
            _ => c_static_str_literal(&format!("<{}>", expr.ty.display_name())),
        }
    }

    fn emit_number_to_str(&mut self, val: &str, func: &str, cast: &str, buf_size: usize) -> String {
        let buf = self.fresh_tmp();
        self.line(&format!("char {buf}[{buf_size}];"));
        format!("{func}(({cast})({val}), {buf}, sizeof({buf}))")
    }
}
