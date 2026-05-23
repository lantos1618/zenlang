use super::*;

#[test]
fn imported_ufc_infers_from_scoped_generic_type_specialization() {
    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_generic_imported_scoped_type_inference/main.zen"),
        &[
            "typedef struct Box_i32 Box_i32;",
            "typedef struct right_Box_i32 right_Box_i32;",
            "typedef struct Holder_right_Box_i32 Holder_right_Box_i32;",
            "int32_t take_box_i32(right_Box_i32 box)",
            "int32_t Box_extra_i32(right_Box_i32 self)",
            "int32_t Holder_extra_right_Box_i32(Holder_right_Box_i32 self)",
            "take_box_i32(box)",
            "Box_extra_i32(box)",
            "Holder_extra_right_Box_i32(holder)",
            "const int32_t found = self.value.extra;",
            "left_i32(1LL)",
            "right_i32(2LL)",
        ],
        &[
            "take_box_i32",
            "Box_extra_i32",
            "Holder_extra_right_Box_i32",
            "left_i32",
            "right_i32",
        ],
        &[
            "T take_box",
            "T Box_extra",
            "T Holder_extra",
            "take_box(box",
        ],
    );
}
