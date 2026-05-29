use super::*;

impl CEmitter {
    pub(super) fn emit_runtime_types(&mut self) {
        self.line("/* Runtime types */");
        self.line("typedef struct { const char* ptr; size_t len; } zen_str;");
        self.blank();

        self.line("static zen_str zen_str_from_cstr(const char* s) {");
        self.indent();
        self.line("return (zen_str){ .ptr = s, .len = strlen(s) };");
        self.dedent();
        self.line("}");
        self.blank();

        self.line("static zen_str zen_i64_to_str(int64_t v, char* buf, size_t bufsz) {");
        self.indent();
        self.line("int n = snprintf(buf, bufsz, \"%lld\", (long long)v);");
        self.line("return (zen_str){ .ptr = buf, .len = (size_t)(n > 0 ? n : 0) };");
        self.dedent();
        self.line("}");
        self.blank();

        self.line("static zen_str zen_f64_to_str(double v, char* buf, size_t bufsz) {");
        self.indent();
        self.line("int n = snprintf(buf, bufsz, \"%g\", v);");
        self.line("return (zen_str){ .ptr = buf, .len = (size_t)(n > 0 ? n : 0) };");
        self.dedent();
        self.line("}");
        self.blank();
    }
}
