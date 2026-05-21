use super::super::*;

#[test]
fn typechecker_program_declaration_checking_lives_in_focused_helper() {
    let root = read("src/typechecker/program_checking.rs");
    let declarations = read("src/typechecker/program_checking/declaration_checking.rs");

    for helper in [
        "check_program_declaration_after_collection",
        "check_impl_block_after_collection",
        "push_non_generic_struct_type",
        "push_non_generic_enum_type",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "program_checking.rs should not own declaration-checking helper: {helper}"
        );
        assert!(
            declarations.contains(&format!("fn {helper}")),
            "declaration-checking helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.lines().count() < 150,
        "program_checking.rs should stay focused on program entry points and final assembly"
    );
    assert!(
        root.contains("mod declaration_checking;"),
        "program_checking.rs should include focused declaration checking helpers"
    );
}
