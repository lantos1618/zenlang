use super::super::*;

#[test]
fn v1_spec_records_resolver_generic_and_behavior_evidence() {
    let spec = read("docs/V1_SPEC.md");

    for required in [
        "resolver_records_value_symbol_generic_parameter_counts",
        "resolver_records_value_symbol_generic_bounds",
        "parse_public_behavior_declaration",
        "resolver_records_public_visibility_for_exported_declarations",
        "check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs",
        "behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied",
        "behavior_extends_generic_parent_accepts_child_type_parameter_arg",
        "resolver_rejects_duplicate_struct_field_names",
        "resolver_rejects_unknown_struct_literal_types",
        "tests/zen/duplicate_enum_variant_names.zen",
        "check_module_graph_entry_seeds_imported_function_type_signatures",
        "check_module_graph_entry_specializes_imported_generic_functions",
        "tests/zen/multi_file_generic/main.zen",
        "tests/zen/multi_file_imported_behavior_requires/main.zen",
        "Generic specialization for functions, structs, enums, and methods",
        "generic_specializations_emit_each_generated_c_definition_once",
        "compile_to_c_with_generated_call_check",
        "undefined_generated_c_calls",
        "generic_specializations::enum_generated_c::enum_specializations_do_not_emit_unspecialized_c_symbols",
        "Explicit behavior association proving ground",
        "tests/zen/behavior_json_explicit_impl.zen",
        "tests/zen/behavior_json_generic_association.zen",
        "imported_behavior_extends_requires_parent_methods",
        "imported_behavior_extends_imported_parent_requires_parent_methods",
        "imported_behavior_extends_requires_transitive_parent_methods",
        "resolver_rejects_duplicate_behavior_impl_edges",
        "resolver_rejects_duplicate_behavior_required_edges",
        "resolver_rejects_duplicate_behavior_parent_edges",
        "tests/zen/generic_method_worklist.zen",
        "resolver_records_behavior_impl_methods_as_value_symbols",
        "imported_private_behavior_impl_methods_are_not_directly_visible",
        "check_program_with_symbols_validates_resolver_generic_behavior_required_refs",
        "resolver_records_method_signatures_as_value_symbols",
        "resolver_records_method_function_type_signatures",
        "check_program_with_symbols_validates_resolver_method_signature",
        "tests/zen/multi_file_type_method/main.zen",
        "parse_impl_block",
        "parse_generic_impl_block_hoists_receiver_type_params_to_methods",
        "resolver_accepts_non_behavior_impl_blocks_as_method_symbols",
        "tests/zen/generic_type_impl_methods.zen",
        "resolver_records_struct_function_type_fields",
        "resolver_records_enum_function_type_payloads",
        "resolver_records_generic_enum_function_type_payloads",
        "resolver_records_behavior_function_type_method_signatures",
        "resolver_records_behavior_impl_method_body_locals",
        "resolver_records_top_level_expr_locals",
        "resolver_records_closure_locals",
        "resolver_records_mutable_closure_parameter_locals",
        "resolver_records_pattern_locals",
        "check_program_with_symbols_requires_resolver_pattern_locals",
        "resolver_records_same_name_locals_in_distinct_scopes",
    ] {
        assert!(
            spec.contains(required),
            "docs/V1_SPEC.md is missing implementation evidence: {required}"
        );
    }

    assert!(
        spec.lines().count() <= 270,
        "docs/V1_SPEC.md should stay compact; move exhaustive evidence to tests, golden fixtures, or git history"
    );
}
