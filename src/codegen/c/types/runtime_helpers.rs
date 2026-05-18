use super::*;

impl CEmitter {
    pub(super) fn emit_runtime_types(&mut self) {
        self.line("/* Runtime types */");
        self.line("typedef struct { const char* ptr; size_t len; } zen_str;");
        self.line("typedef struct zen_allocator zen_allocator;");
        self.line("typedef struct { char* ptr; size_t len; size_t cap; zen_allocator* allocator; } zen_string;");
        self.blank();

        self.emit_c_string_view_helper();
        self.emit_string_print_helper();
        self.emit_int_string_helper();
        self.emit_float_string_helper();
        self.emit_string_concat_helper();
    }

    pub(super) fn emit_io_helpers(&mut self) {
        self.line("/* I/O helpers (stdlib stand-in) */");
        self.line("static void io_println(zen_str s) {");
        self.indent();
        self.line("fwrite(s.ptr, 1, s.len, stdout);");
        self.line("fputc('\\n', stdout);");
        self.dedent();
        self.line("}");
        self.blank();

        self.line("static void io_print(zen_str s) {");
        self.indent();
        self.line("fwrite(s.ptr, 1, s.len, stdout);");
        self.dedent();
        self.line("}");
        self.blank();
    }

    fn emit_c_string_view_helper(&mut self) {
        self.line("static zen_str zen_str_from_cstr(const char* s) {");
        self.indent();
        self.line("return (zen_str){ .ptr = s, .len = strlen(s) };");
        self.dedent();
        self.line("}");
        self.blank();
    }

    fn emit_string_print_helper(&mut self) {
        self.line("static void zen_str_print(zen_str s) {");
        self.indent();
        self.line("fwrite(s.ptr, 1, s.len, stdout);");
        self.dedent();
        self.line("}");
        self.blank();
    }

    fn emit_int_string_helper(&mut self) {
        self.line("static zen_str zen_i64_to_str(int64_t v, char* buf, size_t bufsz) {");
        self.indent();
        self.line("int n = snprintf(buf, bufsz, \"%lld\", (long long)v);");
        self.line("return (zen_str){ .ptr = buf, .len = (size_t)(n > 0 ? n : 0) };");
        self.dedent();
        self.line("}");
        self.blank();
    }

    fn emit_float_string_helper(&mut self) {
        self.line("static zen_str zen_f64_to_str(double v, char* buf, size_t bufsz) {");
        self.indent();
        self.line("int n = snprintf(buf, bufsz, \"%g\", v);");
        self.line("return (zen_str){ .ptr = buf, .len = (size_t)(n > 0 ? n : 0) };");
        self.dedent();
        self.line("}");
        self.blank();
    }

    fn emit_string_concat_helper(&mut self) {
        self.line("static zen_str zen_str_concat(zen_str a, zen_str b, char* buf, size_t bufsz) {");
        self.indent();
        self.line("size_t total = a.len + b.len;");
        self.line("if (total > bufsz - 1) total = bufsz - 1;");
        self.line("memcpy(buf, a.ptr, a.len < total ? a.len : total);");
        self.line("if (a.len < total) memcpy(buf + a.len, b.ptr, total - a.len);");
        self.line("buf[total] = '\\0';");
        self.line("return (zen_str){ .ptr = buf, .len = total };");
        self.dedent();
        self.line("}");
        self.blank();
    }
}
