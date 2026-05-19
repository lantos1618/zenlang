use super::*;

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
