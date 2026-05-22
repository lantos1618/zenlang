use super::*;

#[test]
fn generic_target_behavior_impl_checks_self_parameter_shape() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: T) T { self }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic target behavior impl parameter mismatch should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "parameter 1 for method `encode` in behavior `Json<T>` expects `Box<T>`, found `T`"
        )),
        "expected generic target behavior impl parameter diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_target_behavior_impl_checks_return_shape() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: Box<T>) i32 { 1 }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic target behavior impl return mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("method `encode` for behavior `Json<T>` expects return `T`, found `i32`")),
        "expected generic target behavior impl return diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_target_behavior_impl_rejects_extra_methods() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: Box<T>) T { self.value }
    debug = (self: Box<T>) StaticString { "box" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic target behavior impl extra method should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("method `debug` is not declared by behavior `Json<T>`")),
        "expected generic target behavior impl extra method diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_target_behavior_impl_satisfies_inherited_generic_bound() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) T
}

Pretty.extends(Json<T>)

Box<T>: { value: T }

Box<T>.implements(Pretty<T>) {
    encode = (self: Box<T>) T { self.value }
    pretty = (self: Box<T>) T { self.value }
}

need_json<T: Json<i32>> = (value: T) i32 {
    value.encode()
}

main = () i32 {
    box = Box<i32> { value: 41 }
    need_json(box)
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic child behavior impl should satisfy inherited parent bound");
}

#[test]
fn generic_target_behavior_impl_rejects_parent_child_overlap() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) T
}

Pretty.extends(Json<T>)

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: Box<T>) T { self.value }
}

Box<T>.implements(Pretty<T>) {
    encode = (self: Box<T>) T { self.value }
    pretty = (self: Box<T>) T { self.value }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic target parent/child behavior overlap should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "overlapping implementations of behaviors `Json_T` and `Pretty_T` for generic type `Box<T>`"
        )),
        "expected generic target behavior overlap diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_target_behavior_impl_rejects_alpha_equivalent_duplicate() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: Box<T>) T { self.value }
}

Box<U>.implements(Json<U>) {
    encode = (self: Box<U>) U { self.value }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("alpha-equivalent generic target behavior impl should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("duplicate implementation of behavior `Json_T` for generic type `Box<U>`")),
        "expected alpha-equivalent generic target duplicate diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_target_behavior_impl_rejects_alpha_equivalent_parent_child_overlap() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) T
}

Pretty.extends(Json<T>)

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: Box<T>) T { self.value }
}

Box<U>.implements(Pretty<U>) {
    encode = (self: Box<U>) U { self.value }
    pretty = (self: Box<U>) U { self.value }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("alpha-equivalent generic target parent/child overlap should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "overlapping implementations of behaviors `Json_T` and `Pretty_T` for generic type `Box<U>`"
        )),
        "expected alpha-equivalent generic target overlap diagnostic, got {errors:?}"
    );
}
