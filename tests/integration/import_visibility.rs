use super::*;

#[test]
fn imported_type_method_worklist_helpers_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
inner<T> = (value: T) T {
    value
}

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(1)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to unimported helpers");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'inner'")
            || message.contains("undefined function `inner`"),
        "expected unimported helper diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_method_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Box<T>: {
    value: T
}

Box.inner<T> = (self: Box<T>) T {
    self.value
}

pub Box.get_inner<T> = (self: Box<T>) T {
    self.inner<T>()
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    box = Box<i32> { value: 47 }
    box.inner<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to unimported methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Box_i32` has no method `inner`")
            || message.contains("type `Box` has no method `inner`"),
        "expected unimported method diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_method_imported_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub inner<T> = (value: T) T {
    value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ inner } = helper

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(59)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to source-module imports");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'inner'")
            || message.contains("undefined function `inner`"),
        "expected unimported helper diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_impl_imported_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub Holder<T>: {
    value: T
}

pub Holder.get<T> = (self: Holder<T>) T {
    self.value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ Holder } = helper

pub Box<T>: {
    value: T
}

Box.impl = {
    pub get_held<T> = (self: Box<T>) T {
        holder = Holder<T> { value: self.value }
        holder.get<T>()
    }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    holder = Holder<i32> { value: 61 }
    holder.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct source-module imported type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Holder'")
            || message.contains("unknown type `Holder`")
            || message.contains("unknown generic type `Holder`")
            || message.contains("type `Holder_i32` has no method `get`"),
        "expected unimported helper type or method diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_function_imported_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub Holder<T>: {
    value: T
}

pub Holder.get<T> = (self: Holder<T>) T {
    self.value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ Holder } = helper

pub get_held<T> = (value: T) T {
    holder = Holder<T> { value: value }
    holder.get<T>()
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ get_held } = model

main = () i32 {
    holder = Holder<i32> { value: 73 }
    holder.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct source-module imported type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Holder'")
            || message.contains("unknown type `Holder`")
            || message.contains("unknown generic type `Holder`")
            || message.contains("type `Holder_i32` has no method `get`"),
        "expected unimported helper type or method diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_function_transitive_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
inner<T> = (value: T) T {
    value
}

pub middle<T> = (value: T) T {
    inner(value)
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ middle } = helper

pub outer<T> = (value: T) T {
    middle(value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ outer } = model

main = () i32 {
    middle<i32>(89)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct transitive helper calls");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'middle'")
            || message.contains("undefined function `middle`"),
        "expected unimported transitive helper diagnostic, panic={message}"
    );
}

#[test]
fn imported_function_signature_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Point: {
    x: i32
}

pub make_point = () Point {
    Point { x: 109 }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ make_point } = model

main = () i32 {
    point = Point { x: 109 }
    point.x
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct signature dependency type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Point'")
            || message.contains("unknown type `Point`")
            || message.contains("unknown struct `Point`"),
        "expected unimported signature dependency type diagnostic, panic={message}"
    );
}

#[test]
fn imported_private_type_impl_methods_are_not_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Box<T>: {
    value: T
}

Box.impl = {
    get<T> = (self: Box<T>) T {
        self.value
    }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    box = Box<i32> { value: 34 }
    box.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject private imported impl methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Box_i32` has no method `get`"),
        "expected private imported impl method diagnostic, panic={message}"
    );
}

#[test]
fn imported_private_behavior_impl_methods_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
Hidden: behavior {
    reveal: (Self) str
}

pub Point: {
    x: i32
}

Point.implements(Hidden) {
    reveal = (value: Point) str {
        "hidden"
    }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Point } = model

main = () i32 {
    point = Point { x: 34 }
    point.reveal()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject private imported behavior impl methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Point` has no method `reveal`"),
        "expected private imported behavior impl method diagnostic, panic={message}"
    );
}
