use super::*;

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
    let func_names = module_function_names(&program);

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
    let func_names = module_function_names(&program);
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
    let func_names = module_function_names(&program);
    assert!(func_names.contains(&"main"));
    assert!(func_names.contains(&"square"));
}
