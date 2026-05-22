use super::*;

#[test]
fn test_multi_file_imported_scoped_generic_type_inference_ufc() {
    let zen_path = test_dir().join("multi_file_generic_imported_scoped_type_inference/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "1\n60\n");
}
