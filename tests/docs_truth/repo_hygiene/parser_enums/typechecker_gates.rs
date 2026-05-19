use super::*;

#[test]
fn typechecker_gated_methods_use_owned_action_enum() {
    let source = read("src/typechecker/expressions/method_call_support.rs");

    for forbidden in [
        r#""raise" => Some(Self::ResultRaise)"#,
        r#""await" => Some(Self::EffectAwait)"#,
        "value == Self::ResultRaise.as_str()",
        "value == Self::EffectAwait.as_str()",
        "from_method_name",
    ] {
        assert!(
            !source.contains(forbidden),
            "typechecker gated methods should use GatedMethod parsing/display, not raw spelling checks: {forbidden}"
        );
    }
    assert!(
        source.contains("method.parse::<GatedMethod>()"),
        "typechecker gated method dispatch should parse through GatedMethod"
    );
    assert!(
        source.contains("const ALL: &[GatedMethod]"),
        "typechecker gated methods should keep an enum-owned static table"
    );
    assert!(
        source.contains("GatedMethod::ALL")
            && source.contains(".iter()")
            && source.contains(".copied()")
            && source.contains(".find(|method| method.as_str() == value)"),
        "typechecker gated method parsing should use the enum-owned static table"
    );
}

#[test]
fn typechecker_gated_intrinsics_use_owned_name_enum() {
    let gated = read("src/typechecker/gated_intrinsics.rs");
    let calls = read("src/typechecker/expressions/call_support.rs");

    for forbidden in [
        r#"name == "atomic_add""#,
        r#"name == "atomic_cas""#,
        r#"name == "atomic_load""#,
        r#"name == "atomic_store""#,
        r#"name == "atomic_sub""#,
        r#"name == "atomic_xchg""#,
        r#"name == "async_enqueue""#,
        r#"name == "async_yield""#,
        r#"name == "fence""#,
        r#"name == "gep""#,
        r#"name == "gep_struct""#,
        r#"name == "int_to_ptr""#,
        r#"name == "load""#,
        r#"name == "memcmp""#,
        r#"name == "memcpy""#,
        r#"name == "memmove""#,
        r#"name == "memset""#,
        r#"name == "ptr_to_int""#,
        r#"name == "raw_allocate""#,
        r#"name == "raw_deallocate""#,
        r#"name == "raw_ptr_cast""#,
        r#"name == "raw_reallocate""#,
        r#"name == "store""#,
        r#"name == "syscall0""#,
        r#"name == "syscall1""#,
        r#"name == "syscall2""#,
        r#"name == "syscall3""#,
        r#"name == "syscall4""#,
        r#"name == "syscall5""#,
        r#"name == "syscall6""#,
        r#"name == "type_match""#,
        r#"match name"#,
        "from_name",
        r#""atomic_add" =>"#,
        r#""atomic_cas" =>"#,
        r#""atomic_load" =>"#,
        r#""atomic_store" =>"#,
        r#""atomic_sub" =>"#,
        r#""atomic_xchg" =>"#,
        r#""async_enqueue" =>"#,
        r#""async_yield" =>"#,
        r#""fence" =>"#,
        r#""gep" =>"#,
        r#""gep_struct" =>"#,
        r#""int_to_ptr" =>"#,
        r#""load" =>"#,
        r#""memcmp" =>"#,
        r#""memcpy" =>"#,
        r#""memmove" =>"#,
        r#""memset" =>"#,
        r#""ptr_to_int" =>"#,
        r#""raw_allocate" =>"#,
        r#""raw_deallocate" =>"#,
        r#""raw_ptr_cast" =>"#,
        r#""raw_reallocate" =>"#,
        r#""store" =>"#,
        r#""syscall0" =>"#,
        r#""syscall1" =>"#,
        r#""syscall2" =>"#,
        r#""syscall3" =>"#,
        r#""syscall4" =>"#,
        r#""syscall5" =>"#,
        r#""syscall6" =>"#,
        r#""type_match" =>"#,
    ] {
        assert!(
            !calls.contains(forbidden),
            "typechecker gated intrinsic dispatch should use GatedIntrinsic, not raw spelling checks: {forbidden}"
        );
    }
    for required in [
        "enum GatedIntrinsic",
        "const ALL: &[GatedIntrinsic]",
        "impl fmt::Display for GatedIntrinsic",
        "impl FromStr for GatedIntrinsic",
        "pub(super) const ATOMIC_ADD: &'static str = \"atomic_add\"",
        "pub(super) const ATOMIC_CAS: &'static str = \"atomic_cas\"",
        "pub(super) const ATOMIC_LOAD: &'static str = \"atomic_load\"",
        "pub(super) const ATOMIC_STORE: &'static str = \"atomic_store\"",
        "pub(super) const ATOMIC_SUB: &'static str = \"atomic_sub\"",
        "pub(super) const ATOMIC_XCHG: &'static str = \"atomic_xchg\"",
        "pub(super) const ASYNC_ENQUEUE: &'static str = \"async_enqueue\"",
        "pub(super) const ASYNC_YIELD: &'static str = \"async_yield\"",
        "pub(super) const FENCE: &'static str = \"fence\"",
        "pub(super) const GEP: &'static str = \"gep\"",
        "pub(super) const GEP_STRUCT: &'static str = \"gep_struct\"",
        "pub(super) const INT_TO_PTR: &'static str = \"int_to_ptr\"",
        "pub(super) const LOAD: &'static str = \"load\"",
        "pub(super) const MEMCMP: &'static str = \"memcmp\"",
        "pub(super) const MEMCPY: &'static str = \"memcpy\"",
        "pub(super) const MEMMOVE: &'static str = \"memmove\"",
        "pub(super) const MEMSET: &'static str = \"memset\"",
        "pub(super) const PTR_TO_INT: &'static str = \"ptr_to_int\"",
        "pub(super) const RAW_ALLOCATE: &'static str = \"raw_allocate\"",
        "pub(super) const RAW_DEALLOCATE: &'static str = \"raw_deallocate\"",
        "pub(super) const RAW_PTR_CAST: &'static str = \"raw_ptr_cast\"",
        "pub(super) const RAW_REALLOCATE: &'static str = \"raw_reallocate\"",
        "pub(super) const STORE: &'static str = \"store\"",
        "pub(super) const SYSCALL0: &'static str = \"syscall0\"",
        "pub(super) const SYSCALL1: &'static str = \"syscall1\"",
        "pub(super) const SYSCALL2: &'static str = \"syscall2\"",
        "pub(super) const SYSCALL3: &'static str = \"syscall3\"",
        "pub(super) const SYSCALL4: &'static str = \"syscall4\"",
        "pub(super) const SYSCALL5: &'static str = \"syscall5\"",
        "pub(super) const SYSCALL6: &'static str = \"syscall6\"",
        "pub(super) const TYPE_MATCH: &'static str = \"type_match\"",
        "pub(super) const fn gate_message(self) -> &'static str",
        ".find(|intrinsic| intrinsic.as_str() == name)",
    ] {
        assert!(
            gated.contains(required),
            "gated intrinsic spelling should live in GatedIntrinsic: {required}"
        );
    }
    assert!(
        calls.contains("name.parse::<GatedIntrinsic>()") && calls.contains("gated.gate_message()"),
        "function-call checking should route gated intrinsics through GatedIntrinsic"
    );
}
