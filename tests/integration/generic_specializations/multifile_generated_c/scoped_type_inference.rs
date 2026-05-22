use super::*;

#[test]
fn imported_ufc_infers_from_scoped_generic_type_specialization() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_scoped_type_inference/main.zen"),
    );
    assert!(c_source.contains("typedef struct Box_i32 Box_i32;"));
    assert!(c_source.contains("typedef struct right_Box_i32 right_Box_i32;"));
    assert!(c_source.contains("typedef struct Holder_right_Box_i32 Holder_right_Box_i32;"));
    assert!(c_source.contains("int32_t take_box_i32(right_Box_i32 box)"));
    assert!(c_source.contains("int32_t Box_extra_i32(right_Box_i32 self)"));
    assert!(c_source.contains("int32_t Holder_extra_right_Box_i32(Holder_right_Box_i32 self)"));
    assert!(c_source.contains("take_box_i32(box)"));
    assert!(c_source.contains("Box_extra_i32(box)"));
    assert!(c_source.contains("Holder_extra_right_Box_i32(holder)"));
    assert!(c_source.contains("const int32_t found = self.value.extra;"));
    assert!(c_source.contains("left_i32(1LL)"));
    assert!(c_source.contains("right_i32(2LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "take_box_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "Box_extra_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "Holder_extra_right_Box_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "left_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "right_i32");
    assert!(!c_source.contains("T take_box"));
    assert!(!c_source.contains("T Box_extra"));
    assert!(!c_source.contains("T Holder_extra"));
    assert!(!c_source.contains("take_box(box"));
}
