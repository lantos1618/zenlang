use super::*;

#[test]
fn private_import_is_rejected() {
    let tmp = setup_temp_dir();

    write_private_add_module(&tmp);
    let main_path = write_main_importing_add(&tmp);

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&main_path, &mut files);
    assert!(result.is_err(), "private import should be rejected");
    assert_error_contains(
        result,
        "not exported",
        "error should mention export visibility",
    );
}

#[test]
fn missing_imported_symbol_is_rejected() {
    let tmp = setup_temp_dir();

    write_public_add_module(&tmp);

    let main_path = write_main(&tmp, "{ subtract } = math\n\nmain = () i32 { 0 }\n");

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&main_path, &mut files);
    assert!(
        result.is_err(),
        "missing imported symbol should be rejected"
    );
    assert_error_contains(
        result,
        "does not export",
        "error should mention missing export",
    );
}

#[test]
fn duplicate_imported_symbol_is_rejected() {
    let tmp = setup_temp_dir();

    write_public_add_module(&tmp);

    let main_path = write_main(&tmp, "{ add, add } = math\n\nmain = () i32 { 0 }\n");

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_with_imports(&main_path, &mut files);
    assert!(result.is_err(), "duplicate import should be rejected");
    assert_error_contains(
        result,
        "duplicate import",
        "error should mention duplicate import",
    );
}
