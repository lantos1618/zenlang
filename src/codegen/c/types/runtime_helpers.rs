use super::*;

impl CEmitter {
    pub(super) fn emit_runtime_types(&mut self) {
        self.line("/* Runtime types */");
        self.line("typedef struct { const char* ptr; size_t len; } zen_str;");
        // The uniform async poll function pointer: every coroutine frame's
        // leading field has this type, so any `Future<T>` (held as `void*`) can
        // be driven generically (ASYNC_PLAN.md milestone 1).
        self.line("typedef bool (*zen_poll_fn)(void*, void*);");
        self.blank();

        // A compiler-provided *test* future used to prove genuine suspend/resume
        // (ASYNC_PLAN.md milestone 2): it returns Pending (false) for its first
        // `remaining` polls, then Ready (true) writing `value`. Its frame matches
        // the uniform layout (`__poll` first), so it is drivable through the same
        // `zen_poll_fn` path as any lowered async frame. This is the deterministic
        // Pending source standing in until I/O readiness (milestone 3) lands.
        self.line("typedef struct { zen_poll_fn __poll; int remaining; int value; } zen_ptr_future;");
        self.line("static bool zen_ptr_future_poll(void* self, void* out) {");
        self.indent();
        self.line("zen_ptr_future* f = (zen_ptr_future*)self;");
        self.line("if (f->remaining > 0) { f->remaining -= 1; return false; }");
        self.line("*(int*)out = f->value;");
        self.line("return true;");
        self.dedent();
        self.line("}");
        self.line("static zen_ptr_future* zen_pending_then_ready(int n, int value) {");
        self.indent();
        self.line("zen_ptr_future* f = (zen_ptr_future*)malloc(sizeof(zen_ptr_future));");
        self.line("f->__poll = zen_ptr_future_poll;");
        self.line("f->remaining = n;");
        self.line("f->value = value;");
        self.line("return f;");
        self.dedent();
        self.line("}");
        self.blank();

        // Cooperative single-threaded scheduler (ASYNC_PLAN.md milestone 2,
        // increment 2). The irreducible run-queue + polling lives here in the
        // runtime; the stdlib drives the *policy* (spawn what, run when) on top.
        // A scheduler is a growable array of future-frame handles (each a `void*`
        // whose first field is a `zen_poll_fn`). `scheduler_run` round-robins:
        // it repeatedly sweeps the queue polling each not-yet-Ready frame once,
        // until every frame reports Ready — so tasks that suspend (return
        // Pending) cooperatively interleave.
        self.line("typedef struct { void** items; int len; int cap; } zen_scheduler;");
        self.line("static zen_scheduler* zen_scheduler_new(void) {");
        self.indent();
        self.line("zen_scheduler* s = (zen_scheduler*)malloc(sizeof(zen_scheduler));");
        self.line("s->items = NULL; s->len = 0; s->cap = 0;");
        self.line("return s;");
        self.dedent();
        self.line("}");
        self.line("static void zen_scheduler_spawn(zen_scheduler* s, void* fut) {");
        self.indent();
        self.line("if (s->len == s->cap) {");
        self.indent();
        self.line("int ncap = s->cap == 0 ? 4 : s->cap * 2;");
        self.line("s->items = (void**)realloc(s->items, (size_t)ncap * sizeof(void*));");
        self.line("s->cap = ncap;");
        self.dedent();
        self.line("}");
        self.line("s->items[s->len] = fut;");
        self.line("s->len += 1;");
        self.dedent();
        self.line("}");
        self.line("static void zen_scheduler_run(zen_scheduler* s) {");
        self.indent();
        // `done[i]` marks a frame that has reported Ready; a discarded scratch
        // receives each frame's result (the cooperative scheduler is
        // fire-and-forget — task return values are observed via shared state).
        self.line("char* done = (char*)calloc((size_t)(s->len > 0 ? s->len : 1), 1);");
        self.line("long long scratch;");
        self.line("int remaining = s->len;");
        self.line("while (remaining > 0) {");
        self.indent();
        self.line("for (int i = 0; i < s->len; i++) {");
        self.indent();
        self.line("if (done[i]) { continue; }");
        self.line("void* f = s->items[i];");
        self.line("if ((*(zen_poll_fn*)f)(f, &scratch)) { done[i] = 1; remaining -= 1; }");
        self.dedent();
        self.line("}");
        self.dedent();
        self.line("}");
        self.line("free(done);");
        self.dedent();
        self.line("}");
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
