use super::*;

#[test]
fn private_import_is_rejected() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "add = (a: i32, b: i32) i32 { a + b }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 { add(1, 2) }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&main_path, &mut files);
    assert!(result.is_err(), "private import should be rejected");
    let msg = format!("{}", result.unwrap_err()[0]);
    assert!(
        msg.contains("not exported"),
        "error should mention export visibility, got: {msg}"
    );
}

#[test]
fn missing_imported_symbol_is_rejected() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(&main_path, "{ subtract } = math\n\nmain = () i32 { 0 }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&main_path, &mut files);
    assert!(
        result.is_err(),
        "missing imported symbol should be rejected"
    );
    let msg = format!("{}", result.unwrap_err()[0]);
    assert!(
        msg.contains("does not export"),
        "error should mention missing export, got: {msg}"
    );
}

#[test]
fn duplicate_imported_symbol_is_rejected() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(&main_path, "{ add, add } = math\n\nmain = () i32 { 0 }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&main_path, &mut files);
    assert!(result.is_err(), "duplicate import should be rejected");
    let msg = format!("{}", result.unwrap_err()[0]);
    assert!(
        msg.contains("duplicate import"),
        "error should mention duplicate import, got: {msg}"
    );
}
