use super::*;
use std::fs;

mod graph_loading;

fn setup_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn load_file_with_relative_import() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(
        &math_path,
        "pub add = (a: i32, b: i32) i32 {\n    a + b\n}\n",
    )
    .unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 {\n    add(1, 2)\n}\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let program = ms.load_with_imports(&main_path, &mut files).unwrap();

    let func_names: Vec<&str> = program
        .declarations
        .iter()
        .filter_map(|d| d.name())
        .collect();
    assert!(func_names.contains(&"main"), "should contain main");
    assert!(func_names.contains(&"add"), "should contain imported add");
}

#[test]
fn circular_import_detected() {
    let tmp = setup_temp_dir();

    let a_path = tmp.path().join("a.zen");
    fs::write(&a_path, "{ bar } = b\n\nfoo = () i32 { 1 }\n").unwrap();

    let b_path = tmp.path().join("b.zen");
    fs::write(&b_path, "{ foo } = a\n\nbar = () i32 { 2 }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&a_path, &mut files);
    assert!(result.is_err(), "circular import should be an error");
    let errs = result.unwrap_err();
    let msg = format!("{}", errs[0]);
    assert!(
        msg.contains("circular import"),
        "error should mention circular import, got: {}",
        msg
    );
}

#[test]
fn missing_import_file_error() {
    let tmp = setup_temp_dir();

    let main_path = tmp.path().join("main.zen");
    fs::write(&main_path, "{ Foo } = nonexistent\n\nmain = () i32 { 0 }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&main_path, &mut files);
    assert!(result.is_err(), "missing import file should be an error");
    let errs = result.unwrap_err();
    let msg = format!("{}", errs[0]);
    assert!(
        msg.contains("cannot find imported module"),
        "error should mention missing file, got: {}",
        msg
    );
}

#[test]
fn std_imports_are_skipped() {
    let tmp = setup_temp_dir();

    let main_path = tmp.path().join("main.zen");
    fs::write(&main_path, "{ io } = std\n\nmain = () i32 { 0 }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let program = ms.load_with_imports(&main_path, &mut files).unwrap();
    assert!(program
        .declarations
        .iter()
        .any(|d| d.name() == Some("main")));
}

#[test]
fn stdlib_submodule_import_loads_through_module_system() {
    let tmp = setup_temp_dir();
    let stdlib = tmp.path().join("stdlib");
    fs::create_dir(&stdlib).unwrap();
    fs::write(&stdlib.join("math.zen"), "pub answer = () i32 { 42 }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ answer } = std.math\n\nmain = () i32 { answer() }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::with_stdlib_root(stdlib.clone());

    let program = ms.load_with_imports(&main_path, &mut files).unwrap();
    let func_names: Vec<&str> = program
        .declarations
        .iter()
        .filter_map(|d| d.name())
        .collect();

    assert!(func_names.contains(&"main"));
    assert!(func_names.contains(&"answer"));
    assert_eq!(
        files.file_count(),
        2,
        "main and stdlib module should both be loaded"
    );

    let std_key = stdlib
        .join("math.zen")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let std_info = ms.module_info(&std_key).expect("stdlib module info");
    assert_eq!(std_info.package_id.0, 1, "stdlib modules use package 1");
}

#[test]
fn cached_module_not_reloaded() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n").unwrap();

    let a_path = tmp.path().join("a.zen");
    fs::write(&a_path, "{ add } = math\n\nfoo = () i32 { add(1, 2) }\n").unwrap();

    let b_path = tmp.path().join("b.zen");
    fs::write(&b_path, "{ add } = math\n\nbar = () i32 { add(3, 4) }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    ms.load_with_imports(&a_path, &mut files).unwrap();
    ms.load_with_imports(&b_path, &mut files).unwrap();

    let math_canonical = math_path.canonicalize().unwrap();
    let math_key = math_canonical.display().to_string();
    assert!(
        ms.modules().contains_key(&math_key),
        "math.zen should be cached by canonical path"
    );
    assert_eq!(files.file_count(), 3, "should have 3 files: a, math, b");
}

#[test]
fn loaded_modules_have_stable_ids_and_package_ids() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 { add(1, 2) }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();
    ms.load_with_imports(&main_path, &mut files).unwrap();

    let main_key = main_path.canonicalize().unwrap().display().to_string();
    let math_key = math_path.canonicalize().unwrap().display().to_string();
    let main_info = ms.module_info(&main_key).expect("main module info");
    let math_info = ms.module_info(&math_key).expect("math module info");

    assert_ne!(main_info.id, math_info.id);
    assert_eq!(main_info.package_id, math_info.package_id);
    assert_eq!(main_info.package_id.0, 0, "local modules use package 0");
    assert_eq!(main_info.canonical_path, main_key);
}

#[test]
fn transitive_imports() {
    let tmp = setup_temp_dir();

    let c_path = tmp.path().join("c.zen");
    fs::write(&c_path, "pub helper = () i32 { 42 }\n").unwrap();

    let b_path = tmp.path().join("b.zen");
    fs::write(
        &b_path,
        "{ helper } = c\n\npub wrapper = () i32 { helper() }\n",
    )
    .unwrap();

    let a_path = tmp.path().join("a.zen");
    fs::write(&a_path, "{ wrapper } = b\n\nmain = () i32 { wrapper() }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let program = ms.load_with_imports(&a_path, &mut files).unwrap();
    let func_names: Vec<&str> = program
        .declarations
        .iter()
        .filter_map(|d| d.name())
        .collect();
    assert!(func_names.contains(&"main"));
    assert!(func_names.contains(&"wrapper"));
}

#[test]
fn dotted_path_resolves_to_subdir() {
    let tmp = setup_temp_dir();

    let utils_dir = tmp.path().join("utils");
    fs::create_dir(&utils_dir).unwrap();
    let math_path = utils_dir.join("math.zen");
    fs::write(&math_path, "pub square = (x: i32) i32 { x * x }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ square } = utils.math\n\nmain = () i32 { square(5) }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let program = ms.load_with_imports(&main_path, &mut files).unwrap();
    let func_names: Vec<&str> = program
        .declarations
        .iter()
        .filter_map(|d| d.name())
        .collect();
    assert!(func_names.contains(&"main"));
    assert!(func_names.contains(&"square"));
}

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
