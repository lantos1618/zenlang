use super::*;

#[test]
fn parser_type_declaration_suffixes_use_owned_keyword_enum() {
    let source = read("src/parser/declarations.rs");

    for forbidden in [
        r#"method_name == "impl""#,
        r#"method_name == "implements""#,
        r#"method_name == "requires""#,
        r#"method_name == "extends""#,
        r#"matches!(method_name.as_str(), "implements" | "requires" | "extends")"#,
    ] {
        assert!(
            !source.contains(forbidden),
            "parser type declaration suffix dispatch should use TypeDeclarationKeyword, not raw spelling checks: {forbidden}"
        );
    }
    assert!(
        source.contains("TypeDeclarationKeyword"),
        "parser type declaration suffix dispatch should use TypeDeclarationKeyword"
    );
}

#[test]
fn parser_loop_control_calls_use_owned_action_enum() {
    for path in [
        "src/parser/expressions.rs",
        "src/parser/expressions/suffixes.rs",
    ] {
        let source = read(path);
        for forbidden in [
            r#"name.as_str() == "done""#,
            r#"name.as_str() == "next""#,
            r#"match name.as_str()"#,
            r#""done" => Expression::LoopControl"#,
            r#""next" => Expression::LoopControl"#,
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should parse loop control calls through LoopControlAction, not raw spelling checks: {forbidden}"
            );
        }
    }

    let suffixes = read("src/parser/expressions/suffixes.rs");
    assert!(
        suffixes.contains("name.parse::<LoopControlAction>()"),
        "parser loop-control suffix handling should parse through LoopControlAction"
    );
}

#[test]
fn parser_type_names_use_owned_type_name_enums() {
    let parser_types = read("src/parser/types.rs");
    let type_names = read("src/parser/type_names.rs");

    for forbidden in [
        r#""i8" =>"#,
        r#""i16" =>"#,
        r#""i32" =>"#,
        r#""i64" =>"#,
        r#""u8" =>"#,
        r#""u16" =>"#,
        r#""u32" =>"#,
        r#""u64" =>"#,
        r#""usize" =>"#,
        r#""f32" =>"#,
        r#""f64" =>"#,
        r#""bool" =>"#,
        r#""void" =>"#,
        r#""str" =>"#,
        r#""StaticString" =>"#,
        r#""Self" =>"#,
        r#""Ptr" if"#,
        r#""MutPtr" if"#,
        r#""RawPtr" if"#,
        r#""Slice" if"#,
        "match base.as_str()",
    ] {
        assert!(
            !parser_types.contains(forbidden),
            "parser type-name resolution should parse through owned parser type-name enums: {forbidden}"
        );
    }

    for forbidden in [
        "Self::I8_NAME => Ok(Self::I8)",
        "Self::I16_NAME => Ok(Self::I16)",
        "Self::I32_NAME => Ok(Self::I32)",
        "Self::I64_NAME => Ok(Self::I64)",
        "Self::U8_NAME => Ok(Self::U8)",
        "Self::U16_NAME => Ok(Self::U16)",
        "Self::U32_NAME => Ok(Self::U32)",
        "Self::U64_NAME => Ok(Self::U64)",
        "Self::USIZE_NAME => Ok(Self::Usize)",
        "Self::F32_NAME => Ok(Self::F32)",
        "Self::F64_NAME => Ok(Self::F64)",
        "Self::BOOL_NAME => Ok(Self::Bool)",
        "Self::VOID_NAME => Ok(Self::Void)",
        "Self::STR_NAME => Ok(Self::Str)",
        "STATIC_STRING_TYPE_NAME => Ok(Self::StaticString)",
        "Self::SELF_NAME => Ok(Self::SelfType)",
        "Self::PTR => Ok(Self::Ptr)",
        "Self::MUT_PTR => Ok(Self::MutPtr)",
        "Self::RAW_PTR => Ok(Self::RawPtr)",
        "Self::SLICE => Ok(Self::Slice)",
    ] {
        assert!(
            !type_names.contains(forbidden),
            "parser type-name FromStr should use enum-owned static tables, not raw match arms: {forbidden}"
        );
    }

    for required in [
        "enum ParserBuiltinTypeName",
        "enum ParserBuiltinGenericTypeName",
        "const ALL: &[ParserBuiltinTypeName]",
        "const ALL: &[ParserBuiltinGenericTypeName]",
        "impl FromStr for ParserBuiltinTypeName",
        "impl FromStr for ParserBuiltinGenericTypeName",
        ".find(|name| name.as_str() == value)",
        "name.parse::<ParserBuiltinTypeName>()",
        "base.parse::<ParserBuiltinGenericTypeName>()",
    ] {
        assert!(
            type_names.contains(required) || parser_types.contains(required),
            "parser type-name spelling should live in parser type-name enums: {required}"
        );
    }
}

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

#[test]
fn codegen_c_intrinsics_use_owned_name_enum() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let names = read("src/codegen/c/intrinsics/names.rs");
    let source = format!("{lowering}\n{names}");

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
}

#[test]
fn build_graph_host_effect_methods_parse_dsl_ident_enum() {
    let lowering = read("src/build_graph/lowering.rs");
    let dsl = read("src/build_graph/lowering/dsl.rs");

    for forbidden in [
        "match method.as_str()",
        "method == BuildTargetDslIdent::Env.as_str()",
        "method == BuildTargetDslIdent::ReadFile.as_str()",
    ] {
        assert!(
            !lowering.contains(forbidden),
            "build graph host-effect method dispatch should parse through BuildTargetDslIdent: {forbidden}"
        );
    }
    assert!(
        lowering.contains("method.parse::<BuildTargetDslIdent>()"),
        "build graph host-effect method dispatch should parse method names through BuildTargetDslIdent"
    );
    assert!(
        dsl.contains("impl FromStr for BuildTargetDslIdent"),
        "BuildTargetDslIdent should own parsing for build DSL method names"
    );
}

#[test]
fn cli_emit_json_modes_use_owned_mode_enum() {
    let source = read("src/cli.rs");

    assert!(
        source.contains("enum EmitJsonMode"),
        "emit-json command routing should use an owned EmitJsonMode enum"
    );
    assert!(
        source.contains("mode.parse::<EmitJsonMode>()"),
        "emit-json command routing should parse modes through EmitJsonMode"
    );
    assert!(
        source.contains("EmitJsonMode::usage()"),
        "emit-json usage should be generated from EmitJsonMode"
    );
    assert!(
        source.contains("fn gate_message(self) -> Option<&'static str>"),
        "emit-json gated diagnostics should be owned by EmitJsonMode"
    );
    assert!(
        source.contains("mode.gate_message()"),
        "emit-json command routing should read gated diagnostics from EmitJsonMode"
    );
    assert!(
        !source.contains("<ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml>"),
        "emit-json usage should not duplicate the mode list as a raw string"
    );
}
