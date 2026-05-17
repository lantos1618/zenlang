use super::*;

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

#[test]
fn c_keyword_escaping() {
    assert_eq!(c_ident("int"), "zen_int");
    assert_eq!(c_ident("return"), "zen_return");
    assert_eq!(c_ident("void"), "zen_void");
    assert_eq!(c_ident("while"), "zen_while");
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
