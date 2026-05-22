use super::*;

#[test]
fn codegen_c_intrinsics_use_owned_name_enum() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let names = read("src/codegen/c/intrinsics/names.rs");
    let spelling = read("src/codegen/c/intrinsics/names/spelling.rs");
    let source = format!("{lowering}\n{names}\n{spelling}");

    for forbidden in [
        "match name",
        r#""raw_allocate" =>"#,
        r#""raw_deallocate" =>"#,
        r#""raw_reallocate" =>"#,
        r#""memcpy" =>"#,
        r#""memmove" =>"#,
        r#""memset" =>"#,
        r#""memcmp" =>"#,
        r#""atomic_load" =>"#,
        r#""atomic_store" =>"#,
        r#""atomic_add" =>"#,
        r#""atomic_sub" =>"#,
        r#""atomic_cas" =>"#,
        r#""atomic_xchg" =>"#,
        r#""syscall0" =>"#,
        r#""syscall1" =>"#,
        r#""syscall2" =>"#,
        r#""syscall3" =>"#,
        r#""syscall4" =>"#,
        r#""syscall5" =>"#,
        r#""syscall6" =>"#,
    ] {
        assert!(
            !lowering.contains(forbidden),
            "C intrinsic lowering should parse through CIntrinsic, not raw spelling dispatch: {forbidden}"
        );
    }

    for required in [
        "enum CIntrinsic",
        "mod spelling;",
        "const ALL: &[CIntrinsic]",
        "impl fmt::Display for CIntrinsic",
        "impl FromStr for CIntrinsic",
        "name.parse::<CIntrinsic>()",
        "Self::RAW_ALLOCATE",
        "Self::ATOMIC_LOAD",
        "Self::SYSCALL6",
    ] {
        assert!(
            source.contains(required),
            "C intrinsic spelling should live in CIntrinsic: {required}"
        );
    }

    assert!(
        names.lines().count() < 220,
        "names.rs should stay focused on the intrinsic enum, ordered table, and parse/display glue"
    );
    assert!(
        !names.contains("const ADD_OVERFLOW"),
        "intrinsic spelling constants should live in names/spelling.rs"
    );
    assert!(
        spelling.contains("pub(super) const fn as_str"),
        "intrinsic spelling helper should own string rendering"
    );
}
