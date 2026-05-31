//! Async/await state-machine lowering for the C backend (ASYNC_PLAN.md
//! milestone 1).
//!
//! An `@async` function `f(args) -> T` is lowered to three C artifacts:
//!
//! - a heap-allocated **frame struct** `f__frame` whose first field is a uniform
//!   poll function pointer (so any future can be polled through a `void*`),
//!   followed by `__state` (the resume label: `0` = start, `k` = "resumed after
//!   the k-th await", `-1` = done), the spilled params + locals, and `__ret`;
//! - a **poll function** `static bool f__poll(void* self, void* out)` — a
//!   `switch (fr->__state)` that runs the body up to each await, saving state and
//!   returning `false` (Pending) at a suspend, and writing `*out` + returning
//!   `true` (Ready) at completion;
//! - an allocating **constructor** `f__frame* f(args)` — what a *call* to `f`
//!   (typed `Future<T>`) lowers to: it `malloc`s the frame, wires `__poll`,
//!   copies the args, sets `__state = 0`, and returns the frame pointer.
//!
//! `@await e` (inside a poll body) evaluates `e` to a frame pointer, polls it
//! through its `__poll` field; if Pending it saves the next state and returns
//! `false`, otherwise it binds the ready value and falls through. `block_on(e)`
//! (a compiler-provided driver, emitted in the runtime preamble) loops the poll
//! until Ready and yields the value.
//!
//! ## Scope of what is lowered
//!
//! Supported: a body that is a linear sequence of `VarDecl` / expression
//! statements plus an optional tail, where every `@await` appears only at a
//! *top-level* position — the value of a `VarDecl`, a bare expression statement,
//! or the tail expression. Locals are threaded across suspends because *all*
//! params and locals are spilled into the frame. Awaits nested inside a
//! sub-expression, `match`/`if`, or a loop are **not** supported here and are
//! rejected before codegen (E3082); see `async_is_lowerable`.

use super::*;

/// One spilled slot in the frame: a param or a local, stored by value.
struct FrameSlot {
    name: String,
    ty: Type,
}

impl CEmitter {
    /// The C name of an async function's frame struct.
    fn frame_struct_name(func_name: &str) -> String {
        format!("{}__frame", c_func_ident(func_name))
    }

    /// The C name of an async function's poll function.
    fn poll_fn_name(func_name: &str) -> String {
        format!("{}__poll", c_func_ident(func_name))
    }

    /// Emit the frame `struct` for an async function. Layout:
    /// `{ poll-fn ptr, __state, <params>, <locals>, __ret }`.
    pub(super) fn emit_async_frame_struct(&mut self, func: &TypedFunction) {
        let name = Self::frame_struct_name(&func.name);
        let slots = collect_frame_slots(func);
        self.line(&format!("struct {name} {{"));
        self.indent();
        // Uniform first field: poll through `void*` regardless of the concrete
        // frame type, so any future can be driven generically.
        self.line("zen_poll_fn __poll;");
        self.line("int __state;");
        for slot in &slots {
            self.line(&format!("{};", c_declarator(&slot.ty, &slot.name)));
        }
        if func.return_type != Type::Void && func.return_type != Type::Never {
            self.line(&format!("{} __ret;", Self::c_type(&func.return_type)));
        }
        self.dedent();
        self.line("};");
    }

    /// Emit the allocating constructor: `f__frame* f(args) { ... }`.
    pub(super) fn emit_async_constructor(&mut self, func: &TypedFunction) {
        let frame = Self::frame_struct_name(&func.name);
        let poll = Self::poll_fn_name(&func.name);
        let ctor = c_func_ident(&func.name);
        let params = self.format_params(&func.params);
        self.line(&format!("{frame}* {ctor}({params}) {{"));
        self.indent();
        self.line(&format!(
            "{frame}* __fr = ({frame}*)malloc(sizeof({frame}));"
        ));
        self.line(&format!("__fr->__poll = {poll};"));
        self.line("__fr->__state = 0;");
        for param in &func.params {
            let id = c_ident(&param.name);
            self.line(&format!("__fr->{id} = {id};"));
        }
        self.line("return __fr;");
        self.dedent();
        self.line("}");
    }

    /// Emit the poll function — the state machine itself.
    pub(super) fn emit_async_poll(&mut self, func: &TypedFunction) {
        let frame = Self::frame_struct_name(&func.name);
        let poll = Self::poll_fn_name(&func.name);
        self.line(&format!("static bool {poll}(void* __self, void* __out) {{"));
        self.indent();
        self.line(&format!("{frame}* __fr = ({frame}*)__self;"));
        // Inside the poll body, every spilled name (`x`) is referenced as
        // `__fr->x`; this set drives `emit_expr_inline` for async bodies.
        let slots = collect_frame_slots(func);
        let saved_prefix = std::mem::take(&mut self.async_frame_fields);
        for slot in &slots {
            self.async_frame_fields.insert(c_ident(&slot.name));
        }

        // Pre-declare a result temp + future-handle temp for every await point,
        // at function scope. C forbids a declaration immediately after a `case`
        // label and a `case` may jump past an in-block declaration, so all await
        // state must live outside the `switch`.
        let await_types = collect_await_types(func);
        for (i, ty) in await_types.iter().enumerate() {
            self.line(&format!("{} __aw{};", Self::c_type(ty), i));
            self.line(&format!("void* __af{} = NULL;", i));
        }
        self.async_await_index = 0;

        self.line("switch (__fr->__state) {");
        self.line("case 0:");
        self.indent();

        // Emit the linear body, splitting at each await into a new case.
        for stmt in &func.body.statements {
            self.emit_async_statement(stmt);
        }

        if let Some(tail) = &func.body.expr {
            if func.return_type != Type::Void && func.return_type != Type::Never {
                let val = self.emit_async_value(tail);
                self.line(&format!("__fr->__ret = {val};"));
                self.line(&format!(
                    "*({}*)__out = __fr->__ret;",
                    Self::c_type(&func.return_type)
                ));
            } else {
                let s = self.emit_async_value(tail);
                if !s.is_empty() {
                    self.line(&format!("{s};"));
                }
            }
        }

        self.line("__fr->__state = -1;");
        self.line("return true;");
        self.dedent();
        self.line("default: return true;");
        self.line("}"); // end switch

        self.async_frame_fields = saved_prefix;
        self.dedent();
        self.line("}");
    }

    /// Emit one statement inside an async poll body. A `VarDecl`/expression whose
    /// value is `@await e` becomes an await sequence (poll + maybe-suspend); the
    /// local is written back into the frame.
    fn emit_async_statement(&mut self, stmt: &TypedStatement) {
        match &stmt.kind {
            TypedStatementKind::VarDecl { name, value, .. } => {
                let id = c_ident(name);
                let val = self.emit_async_value(value);
                // The slot already exists in the frame; assign into it.
                self.line(&format!("__fr->{id} = {val};"));
            }
            TypedStatementKind::Expression(expr) => {
                let s = self.emit_async_value(expr);
                if !s.is_empty() {
                    self.line(&format!("{s};"));
                }
            }
        }
    }

    /// Emit an expression that may be (or be, at top level) an `@await`,
    /// returning a C expression for its value. An await emits its poll/suspend
    /// sequence as statements and returns the bound result temp.
    fn emit_async_value(&mut self, expr: &TypedExpression) -> String {
        if let TypedExprKind::Await { expr: inner } = &expr.kind {
            return self.emit_await(inner, &expr.ty);
        }
        self.emit_expr_inline(expr)
    }

    /// Emit the await sequence for `@await inner` (inner : `Future<value_ty>`):
    /// evaluate the inner future to a frame pointer, save the resume state, then
    /// at the resume `case` poll it through its uniform `__poll` field. If
    /// Pending, stay parked and report Pending; else fall through with the ready
    /// value in `__aw<i>`. Returns the name of that result temp.
    fn emit_await(&mut self, inner: &TypedExpression, _value_ty: &Type) -> String {
        let i = self.async_await_index;
        self.async_await_index += 1;
        // State numbering: state 0 is the start; the k-th await resumes at k.
        let state = i + 1;
        let fut = self.emit_expr_inline(inner);
        self.line(&format!("__af{i} = (void*){fut};"));
        self.line(&format!("__fr->__state = {state};"));
        self.line(&format!("case {state}:"));
        // Poll the inner future through its uniform `__poll` field. If Pending,
        // stay parked at this state and report Pending to our caller.
        self.line(&format!(
            "if (!(*(zen_poll_fn*)__af{i})(__af{i}, &__aw{i})) {{ return false; }}"
        ));
        format!("__aw{i}")
    }
}

/// The inner value type of each `@await` in the body, in source order — used to
/// pre-declare a result temp per await at poll-function scope.
fn collect_await_types(func: &TypedFunction) -> Vec<Type> {
    let mut tys = Vec::new();
    let mut visit = |expr: &TypedExpression| {
        if let TypedExprKind::Await { .. } = &expr.kind {
            tys.push(expr.ty.clone());
        }
    };
    for stmt in &func.body.statements {
        match &stmt.kind {
            TypedStatementKind::VarDecl { value, .. } => visit(value),
            TypedStatementKind::Expression(expr) => visit(expr),
        }
    }
    if let Some(tail) = &func.body.expr {
        visit(tail);
    }
    tys
}

/// All slots spilled into the frame: every param, then every `VarDecl` local in
/// the (linear) body. Spilling everything is always sound and lets any local be
/// threaded across a suspend.
fn collect_frame_slots(func: &TypedFunction) -> Vec<FrameSlot> {
    let mut slots = Vec::new();
    for param in &func.params {
        slots.push(FrameSlot {
            name: param.name.clone(),
            ty: param.ty.clone(),
        });
    }
    for stmt in &func.body.statements {
        if let TypedStatementKind::VarDecl { name, ty, .. } = &stmt.kind {
            slots.push(FrameSlot {
                name: name.clone(),
                ty: ty.clone(),
            });
        }
    }
    slots
}

/// Whether an async function's body fits the milestone-1 lowering shape: a
/// linear sequence of statements + optional tail, with `@await` only at the
/// top level of a `VarDecl` value, an expression statement, or the tail. Anything
/// else (await nested in a sub-expression, branch, or loop) is out of scope and
/// is gated with E3082 by the caller.
pub fn async_is_lowerable(func: &TypedFunction) -> bool {
    for stmt in &func.body.statements {
        match &stmt.kind {
            TypedStatementKind::VarDecl { value, .. } => {
                if !value_is_lowerable(value) {
                    return false;
                }
            }
            TypedStatementKind::Expression(expr) => {
                if !value_is_lowerable(expr) {
                    return false;
                }
            }
        }
    }
    match &func.body.expr {
        Some(tail) => value_is_lowerable(tail),
        None => true,
    }
}

/// A top-level value position: either a bare `@await e` (where `e` itself must
/// contain no further await), or an await-free expression.
fn value_is_lowerable(expr: &TypedExpression) -> bool {
    match &expr.kind {
        TypedExprKind::Await { expr: inner } => !contains_await(inner),
        _ => !contains_await(expr),
    }
}

/// Whether `@await` appears anywhere in `expr` (used to reject nested awaits).
fn contains_await(expr: &TypedExpression) -> bool {
    match &expr.kind {
        TypedExprKind::Await { .. } => true,
        TypedExprKind::BinaryOp { left, right, .. } => {
            contains_await(left) || contains_await(right)
        }
        TypedExprKind::UnaryOp { operand, .. } => contains_await(operand),
        TypedExprKind::FunctionCall { args, .. } | TypedExprKind::Intrinsic { args, .. } => {
            args.iter().any(contains_await)
        }
        TypedExprKind::FieldAccess { object, .. } => contains_await(object),
        TypedExprKind::IndexAccess { object, index } => {
            contains_await(object) || contains_await(index)
        }
        TypedExprKind::StructLiteral { fields, .. } => {
            fields.iter().any(|(_, e)| contains_await(e))
        }
        TypedExprKind::EnumVariant { payload, .. } => {
            payload.as_deref().is_some_and(contains_await)
        }
        TypedExprKind::ArrayLiteral { elements } => elements.iter().any(contains_await),
        TypedExprKind::Cast { expr, .. }
        | TypedExprKind::Ref(expr)
        | TypedExprKind::MutRef(expr)
        | TypedExprKind::Deref(expr) => contains_await(expr),
        TypedExprKind::Assign { target, value } => {
            contains_await(target) || contains_await(value)
        }
        TypedExprKind::Match { scrutinee, arms, .. } => {
            contains_await(scrutinee)
                || arms.iter().any(|arm| {
                    arm.body
                        .expr
                        .as_deref()
                        .is_some_and(contains_await)
                        || arm.body.statements.iter().any(stmt_contains_await)
                })
        }
        TypedExprKind::Block(block) => {
            block.statements.iter().any(stmt_contains_await)
                || block.expr.as_deref().is_some_and(contains_await)
        }
        TypedExprKind::StringInterpolation { parts } => parts.iter().any(|p| match p {
            TypedStringPart::Expr(e) => contains_await(e),
            TypedStringPart::Literal(_) => false,
        }),
        _ => false,
    }
}

fn stmt_contains_await(stmt: &TypedStatement) -> bool {
    match &stmt.kind {
        TypedStatementKind::VarDecl { value, .. } => contains_await(value),
        TypedStatementKind::Expression(expr) => contains_await(expr),
    }
}
