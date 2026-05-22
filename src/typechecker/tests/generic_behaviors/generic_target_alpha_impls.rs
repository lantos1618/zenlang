use super::*;

fn assert_generic_impl_error_contains(source: &str, expected: &str) {
    let program = parse_program(source);
    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("alpha-equivalent generic target behavior impl should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(expected)),
        "expected diagnostic containing `{expected}`, got {errors:?}"
    );
}

#[test]
fn rejects_alpha_equivalent_duplicate() {
    assert_generic_impl_error_contains(
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
        "duplicate implementation of behavior `Json_T` for generic type `Box<U>`",
    );
}

#[test]
fn rejects_alpha_equivalent_parent_child_overlap() {
    assert_generic_impl_error_contains(
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
        "overlapping implementations of behaviors `Json_T` and `Pretty_T` for generic type `Box<U>`",
    );
}

#[test]
fn rejects_alpha_equivalent_parent_child_overlap_in_reverse_order() {
    assert_generic_impl_error_contains(
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

Box<U>.implements(Json<U>) {
    encode = (self: Box<U>) U { self.value }
}
"#,
        "overlapping implementations of behaviors `Pretty_T` and `Json_T` for generic type `Box<U>`",
    );
}

#[test]
fn rejects_alpha_equivalent_overlap_when_behavior_param_is_renamed() {
    assert_generic_impl_error_contains(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Pretty<U>: behavior {
    pretty: (Self) U
}

Pretty.extends(Json<U>)

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: Box<T>) T { self.value }
}

Box<V>.implements(Pretty<V>) {
    encode = (self: Box<V>) V { self.value }
    pretty = (self: Box<V>) V { self.value }
}
"#,
        "overlapping implementations of behaviors `Json_T` and `Pretty_T` for generic type `Box<V>`",
    );
}

#[test]
fn rejects_transitive_alpha_equivalent_parent_child_overlap() {
    assert_generic_impl_error_contains(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Compact<T>: behavior {
    compact: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) T
}

Compact.extends(Json<T>)
Pretty.extends(Compact<T>)

Box<T>: { value: T }

Box<T>.implements(Json<T>) {
    encode = (self: Box<T>) T { self.value }
}

Box<U>.implements(Pretty<U>) {
    encode = (self: Box<U>) U { self.value }
    compact = (self: Box<U>) U { self.value }
    pretty = (self: Box<U>) U { self.value }
}
"#,
        "overlapping implementations of behaviors `Json_T` and `Pretty_T` for generic type `Box<U>`",
    );
}

#[test]
fn rejects_nested_alpha_equivalent_behavior_type_args() {
    assert_generic_impl_error_contains(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Box<T>: {
    value: Ptr<T>
}

Box<T>.implements(Json<Ptr<T>>) {
    encode = (self: Box<T>) Ptr<T> { self.value }
}

Box<U>.implements(Json<Ptr<U>>) {
    encode = (self: Box<U>) Ptr<U> { self.value }
}
"#,
        "duplicate implementation of behavior `Json_ptr_T` for generic type `Box<U>`",
    );
}
