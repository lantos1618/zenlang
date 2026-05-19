# Zen v1 Specification Draft

Status: v1 draft. This document is normative for intended v1 behavior, but the
feature matrix below controls what the rewrite compiler may currently advertise.

## Baseline

The active implementation is the rewrite compiler:

```text
source -> tokens -> AST -> module loader -> typechecker -> typed AST -> C backend -> cc
```

Compiler-owned semantic data is the source of truth. Serialized JSON and YAML are
interface formats only; they must be generated from or validated against checked
compiler data.

## Syntax Contract

Implemented syntax forms are limited to the forms covered by `tests/zen` and Rust
unit tests: declarations, function calls, local bindings, structs, enums, field
access, method-style calls, final-expression results, loops, `defer`, casts, string interpolation,
and pattern-style `?` arms supported by the parser and C backend.

Unsupported spec-like constructs must stay gated until parser and semantic tests
exist. This includes unspecialized generic behavior bounds such as `T: Json`,
unspecialized generic type association targets such as `Box.implements(Json)`,
comptime execution, type matching, async operations, actor syntax, package
manifests, and `build.zen` execution beyond the constrained deterministic
graph surface.

Developer UX and Agent UX are product requirements, not polish. The v1 language
surface should grow toward MoonBit-style toolchain integration, but the compiler
must not advertise unsupported language-server binaries or editor features as
implemented. The current contract is:

- the checked-in VS Code extension remains a constrained editor wrapper around
  syntax support and existing CLI commands until language-server tests exist;
- `zen lsp` remains gated until it is backed by the same parser, resolver,
  typechecker, build graph, and diagnostics as the CLI;
- Agent-readable diagnostics must keep stable codes, spans, related locations,
  suggested fixes, gated-feature metadata, and JSON output aligned with CLI and
  editor behavior;
- the machine-readable project graph and symbol graph surfaces must remain
  compiler-owned outputs for modules, imports, visibility, targets,
  dependencies, generated symbols, examples, and stdlib gates;
- structured fix suggestions are part of the planned UX for missing match arms,
  generic arity mistakes, removed syntax, gated features, missing imports, and
  type mismatches;
- quiet deterministic commands such as `zen check`, `zen test`, and
  `zen emit-json` are required for agents and editors before broader automated
  fix or package workflows can be promoted.

## Accepted Syntax Forms

Every accepted syntax form must have a spec entry and Test Evidence before it is
advertised as implemented.

Additional resolver/typechecker generic-bound handoff evidence:
`resolver_phase2::resolver_records_value_symbol_generic_bounds` and
`typechecker::tests::check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs`.
Generic behavior type-parameter bound enforcement is covered by
`typechecker::tests::behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied`
and
`typechecker::tests::behavior_impl_generic_behavior_type_arg_bound_failure_is_error`.
Generic behavior inheritance with child type-parameter parent args is covered by
`typechecker::tests::behavior_extends_generic_parent_accepts_child_type_parameter_arg`.

| Syntax form | Status | Test Evidence |
|---|---|---|
| Function declaration `name = (params) Return { ... }` | implemented | `parser::tests::parse_simple_function`, `tests/zen/functions.zen` |
| Method declaration `Type.method = (...) Return { ... }` | implemented | `parser::tests::parse_method`, `resolver_phase2::resolver_rejects_method_on_unknown_type`, `resolver_phase2::resolver_records_method_signatures_as_value_symbols`, `resolver_phase2::resolver_records_method_function_type_signatures`, `typechecker::tests::check_program_with_symbols_requires_resolver_method_receiver_type`, `typechecker::tests::check_program_with_symbols_validates_resolver_method_signature`, `typechecker::tests::check_program_with_symbols_validates_resolver_method_function_type_signature`, `typechecker::tests::check_module_graph_entry_does_not_seed_private_methods_for_imported_types`, `tests/zen/ufc.zen`, `tests/zen/multi_file_type_method/main.zen`, `tests/zen/multi_file_type_method_worklist/main.zen`, `tests/zen/multi_file_type_method_method_dependency/main.zen`, `tests/zen/multi_file_type_method_imported_dependency/main.zen`, `integration::test_multi_file_type_method_imports`, `integration::test_multi_file_type_method_worklist_imports`, `integration::test_multi_file_type_method_method_dependency_imports`, `integration::test_multi_file_type_method_imported_dependency_imports`, `integration::imported_type_method_worklist_helpers_are_not_directly_visible`, `integration::imported_type_method_dependencies_are_not_directly_visible`, `integration::imported_type_method_imported_dependencies_are_not_directly_visible` |
| Non-behavior impl blocks `Type.impl = { ... }` and `Type<T>.impl = { ... }` | experimental | `parser::tests::parse_impl_block`, `parser::tests::parse_generic_impl_block_hoists_receiver_type_params_to_methods`, `resolver_phase2::resolver_accepts_non_behavior_impl_blocks_as_method_symbols`, `resolver_phase2::resolver_rejects_duplicate_non_behavior_impl_method_names`, `resolver_phase2::resolver_rejects_non_behavior_impl_method_colliding_with_top_level_method`, `tests/zen/type_impl_methods.zen`, `tests/zen/generic_type_impl_methods.zen`, `tests/zen/multi_file_type_impl/main.zen`, `tests/zen/multi_file_type_impl_imported_type_dependency/main.zen`, `integration::test_type_impl_methods`, `integration::test_generic_type_impl_methods`, `integration::test_multi_file_type_impl_imports`, `integration::test_multi_file_type_impl_imported_type_dependency_imports`, `integration::imported_private_type_impl_methods_are_not_visible`, `integration::imported_type_impl_imported_type_dependencies_are_not_directly_visible`, `integration::generic_specializations_do_not_emit_unspecialized_c_symbols` |
| Struct declaration `Name: { field: Type }` | implemented | `parser::tests::parse_struct_def`, `resolver_phase2::resolver_records_struct_field_counts`, `resolver_phase2::resolver_records_struct_field_types`, `resolver_phase2::resolver_records_struct_function_type_fields`, `resolver_phase2::resolver_records_struct_field_default_locals`, `resolver_phase2::resolver_rejects_duplicate_struct_field_names`, `resolver_phase2::resolver_rejects_duplicate_struct_literal_fields`, `resolver_phase2::resolver_rejects_unknown_struct_literal_fields`, `resolver_phase2::resolver_rejects_missing_struct_literal_fields`, `resolver_phase2::resolver_rejects_unknown_struct_literal_types`, `typechecker::tests::check_program_with_symbols_validates_resolver_struct_field_counts`, `typechecker::tests::check_program_with_symbols_validates_resolver_struct_field_types`, `typechecker::tests::check_program_with_symbols_validates_resolver_struct_function_type_fields`, `typechecker::tests::check_program_with_symbols_validates_resolver_struct_typed_field_metadata`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_struct_field_metadata`, `typechecker::tests::check_program_with_symbols_requires_resolver_struct_field_default_locals`, `tests/zen/structs.zen` |
| Enum declaration `Name: Variant, Payload(Type)` | implemented | `parser::tests::parse_enum_def`, `parser::tests::parse_enum_with_payload`, `resolver_phase2::resolver_records_enum_variant_names`, `resolver_phase2::resolver_records_enum_variant_owner_names`, `resolver_phase2::resolver_allows_same_variant_names_in_different_enums`, `resolver_phase2::resolver_rejects_duplicate_variant_names_in_same_enum`, `resolver_phase2::resolver_rejects_unknown_enum_variant_expressions`, `resolver_phase2::resolver_rejects_missing_enum_variant_payload_expressions`, `resolver_phase2::resolver_rejects_unexpected_enum_variant_payload_expressions`, `resolver_phase2::resolver_records_enum_function_type_payloads`, `resolver_phase2::resolver_records_generic_enum_function_type_payloads`, `typechecker::tests::check_program_with_symbols_validates_resolver_enum_variant_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_enum_variant_owner_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_enum_function_type_payloads`, `typechecker::tests::check_program_with_symbols_validates_resolver_enum_typed_payload_metadata`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_enum_payload_metadata`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads`, `tests/zen/enums.zen`, `tests/zen/duplicate_enum_variant_names.zen` |
| Local imports `{ name } = module.path` | implemented | `parser::tests::parse_import`, `module_system::tests::load_file_with_relative_import`, `typechecker::tests::check_module_graph_entry_uses_graph_import_bindings`, `typechecker::tests::check_module_graph_entry_seeds_imported_function_type_signatures`, `typechecker::tests::check_module_graph_entry_specializes_imported_generic_functions`, `typechecker::tests::check_module_graph_entry_specializes_imported_generic_enums`, `typechecker::tests::check_module_graph_entry_seeds_public_methods_for_imported_types`, `typechecker::tests::check_module_graph_entry_does_not_seed_private_methods_for_imported_types`, `typechecker::tests::check_module_graph_entry_specializes_public_generic_methods_for_imported_types`, `tests/zen/multi_file_generic/main.zen`, `tests/zen/multi_file_generic_imported_type_dependency/main.zen`, `tests/zen/multi_file_generic_imported_worklist_chain/main.zen`, `tests/zen/multi_file_generic_imported_transitive_dependency/main.zen`, `integration::test_multi_file_generic_imported_worklist_chain_imports`, `integration::test_multi_file_generic_imported_transitive_dependency_imports`, `integration::imported_generic_function_transitive_dependencies_are_not_directly_visible`, `integration::test_multi_file_generic_imported_type_dependency_imports`, `integration::imported_generic_function_imported_type_dependencies_are_not_directly_visible`, `tests/zen/multi_file_type_method/main.zen`, `tests/zen/multi_file_type_method_worklist/main.zen`, `tests/zen/multi_file_type_method_method_dependency/main.zen`, `tests/zen/multi_file_type_method_imported_dependency/main.zen`, `tests/zen/multi_file_type_impl/main.zen`, `tests/zen/multi_file_type_impl_imported_type_dependency/main.zen`, `tests/zen/multi_file_behavior_bound/main.zen`, `tests/zen/multi_file_behavior_inheritance/main.zen`, `tests/zen/multi_file_imported_behavior_impl/main.zen`, `tests/zen/multi_file_imported_behavior_default/main.zen`, `tests/zen/multi_file_imported_generic_behavior_default/main.zen`, `tests/zen/multi_file_imported_impl_imported_behavior/main.zen`, `tests/zen/multi_file_imported_child_parent_dispatch/main.zen`, `tests/zen/multi_file_imported_behavior_requires/main.zen`, `tests/zen/multi_file_imported_function_imported_behavior_bound/main.zen`, `tests/zen/multi_file_imported_function_param_type_dependency/main.zen`, `tests/zen/multi_file_imported_function_return_type_dependency/main.zen`, `tests/zen/multi_file_imported_function_imported_return_type_behavior/main.zen`, `tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen`, `integration::test_multi_file_imported_function_imported_behavior_bound`, `integration::test_multi_file_imported_function_param_type_dependency`, `integration::test_multi_file_imported_function_return_type_dependency`, `integration::test_multi_file_imported_function_imported_return_type_behavior`, `integration::test_multi_file_imported_generic_function_return_enum_dependency`, `integration::imported_function_signature_type_dependencies_are_not_directly_visible` |
| Immutable and mutable local bindings | implemented | `parser::tests::parse_immutable_var`, `parser::tests::parse_var_decl_mutable`, `resolver_phase2::resolver_records_parameter_and_local_symbols`, `resolver_phase2::resolver_records_top_level_expr_locals`, `resolver_phase2::resolver_records_closure_locals`, `resolver_phase2::resolver_records_mutable_closure_parameter_locals`, `resolver_phase2::resolver_records_same_name_locals_in_distinct_scopes`, `typechecker::tests::check_program_with_symbols_requires_resolver_parameter_locals`, `typechecker::tests::check_program_with_symbols_requires_resolver_var_decl_locals`, `typechecker::tests::check_program_with_symbols_requires_resolver_top_level_expr_locals`, `typechecker::tests::check_program_with_symbols_requires_resolver_closure_locals`, `typechecker::tests::check_program_with_symbols_validates_resolver_closure_parameter_mutability`, `typechecker::tests::check_program_with_symbols_validates_resolver_local_mutability_by_scope`, `tests/zen/mutability.zen` |
| Final expression results, `break`, `continue`, and prefix `loop((l) { ... })` controls used by fixtures | implemented | `parser::tests::parse_return_keyword_is_removed`, `parser::tests::parse_loop_expr`, `parser::tests::parse_loop_control_param_expr`, `tests/zen/loops.zen`, `tests/zen/loop_control.zen` |
| Pattern-style `?` arms supported by parser/codegen | implemented | `parser::tests::parse_pattern_match`, `resolver_phase2::resolver_records_pattern_locals`, `typechecker::tests::check_program_with_symbols_requires_resolver_pattern_locals`, `tests/zen/conditionals.zen`, `tests/zen/enum_match.zen` |
| Field access and struct literals | implemented | `parser::tests::parse_struct_literal`, `tests/zen/nested_structs.zen` |
| UFC-style method calls | implemented | `parser::tests::parse_ufc_chain`, `tests/zen/ufc.zen` |
| Cast expressions `cast(value, Type)` | implemented | `parser::tests::parse_cast_expr`, `tests/zen/cast.zen` |
| String literals as baked `StaticString`; interpolation as non-owning `StaticString` views | implemented | `parser::tests::parse_string_interpolation`, `tests/zen/strings.zen`, `dynamic_string_type_is_rejected_as_allocator_backed_gate` |
| Pointer and slice type syntax accepted by parser | implemented | `parser::tests::parse_pointer_types`, `parser::tests::parse_slice_type` |
| Generic specialization for functions, structs, enums, and methods | implemented | `tests/zen/generic_identity.zen`, `tests/zen/generic_struct.zen`, `tests/zen/generic_enum_option.zen`, `tests/zen/generic_result_enum.zen`, `tests/zen/generic_method.zen`, `tests/zen/generic_method_worklist.zen`, `tests/zen/generic_method_nested_result.zen`, `tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen`, `tests/zen/multi_file_type_method_nested_result_dependency/main.zen`, `integration::generic_specializations_emit_each_generated_c_definition_once`, `generic_specializations::enum_generated_c::enum_specializations_do_not_emit_unspecialized_c_symbols`, `generic_specializations::method_worklist_generated_c::method_and_worklist_specializations_do_not_emit_unspecialized_c_symbols`, `generic_specializations::multifile_generated_c::enum_dependencies::multi_file_generic_enum_specializations_do_not_emit_unspecialized_c_symbols`, `generic_specializations::multifile_generated_c::method_worklist_dependencies::multi_file_generic_method_and_worklist_specializations_do_not_emit_unspecialized_c_symbols` |
| Explicit behavior association proving ground | implemented | `tests/zen/behavior_json_explicit_impl.zen`, `tests/zen/behavior_json_generic_association.zen`, `tests/zen/behavior_distinct_generic_specialization_dispatch.zen`, `tests/zen/behavior_json_generic_bound_ufcs.zen`, `tests/zen/multi_file_imported_behavior_requires/main.zen`, `tests/zen/multi_file_behavior_inheritance/main.zen`, `generic_diagnostics::behavior_impl_for_unspecialized_generic_type_is_error`, `generic_diagnostics::generic_behavior_bound_unknown_method_is_error` |
| Generic syntax and explicit behavior bounds | experimental | `parser::tests::parse_nested_generics`, `parser::tests::parse_generic_behavior_function_bound_with_type_args`, `parser::tests::parse_generic_behavior_type_bound_with_type_args`, `resolver_phase2::resolver_records_generic_behavior_bounds_with_type_args`, `resolver_phase2::resolver_records_value_symbol_generic_parameter_counts`, `resolver_phase2::resolver_records_value_symbol_function_type_metadata`, `resolver_phase2::resolver_records_type_and_behavior_generic_parameter_counts`, `resolver_phase2::resolver_rejects_duplicate_type_parameter_names`, `resolver_phase2::resolver_records_generic_struct_field_types`, `resolver_phase2::resolver_records_generic_enum_variant_payload_types`, `resolver_phase2::resolver_records_generic_enum_function_type_payloads`, `resolver_phase2::resolver_records_generic_behavior_method_signatures`, `resolver_phase2::resolver_records_generic_behavior_function_type_method_signatures`, `typechecker::tests::generic_function_collection`, `typechecker::tests::check_program_with_symbols_validates_resolver_function_type_parameter_metadata`, `typechecker::tests::check_program_with_symbols_validates_resolver_function_type_return_metadata`, `typechecker::tests::check_program_with_symbols_validates_resolver_function_typed_signature_metadata`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_function_type_metadata`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_function_template_metadata`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_method_template_metadata`, `typechecker::tests::check_program_with_symbols_validates_resolver_function_type_parameter_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_type_parameter_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_struct_field_types`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_enum_payload_types`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_method_signatures`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures`, `typechecker::tests::check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds`, `typechecker::tests::generic_bound_rejects_unspecialized_generic_behavior`, `typechecker::tests::behavior_generic_bound_accepts_later_behavior_declaration`, `typechecker::tests::generic_behavior_bound_with_type_args_accepts_matching_impl`, `typechecker::tests::generic_behavior_bound_with_type_args_rejects_mismatched_impl`, `generic_diagnostics::generic_struct_annotation_type_arg_arity_is_error`, `generic_diagnostics::generic_enum_annotation_type_arg_arity_is_error`, `generic_diagnostics::generic_struct_annotation_without_type_args_is_error`, `generic_diagnostics::generic_enum_annotation_without_type_args_is_error`, `generic_diagnostics::generic_struct_local_annotation_type_arg_arity_is_error`, `generic_diagnostics::generic_struct_local_annotation_without_type_args_is_error`, `generic_diagnostics::generic_enum_local_annotation_type_arg_arity_is_error`, `generic_diagnostics::generic_enum_local_annotation_without_type_args_is_error`, `generic_diagnostics::nested_generic_annotation_inner_type_arg_arity_is_error`, `generic_diagnostics::nested_generic_instantiation_inner_type_arg_arity_is_error`, `generic_diagnostics::function_type_parameter_annotation_type_arg_arity_is_error`, `generic_diagnostics::function_type_return_annotation_without_type_args_is_error`, `generic_diagnostics::pointer_type_inner_generic_annotation_arity_is_error`, `generic_diagnostics::slice_type_inner_generic_annotation_without_type_args_is_error`, `generic_diagnostics::array_type_inner_generic_annotation_arity_is_error`, `generic_diagnostics::generic_struct_local_annotation_bound_failure_is_error`, `generic_diagnostics::generic_enum_local_annotation_bound_failure_is_error`, `generic_diagnostics::generic_function_behavior_bound_failure_is_error`, `generic_diagnostics::generic_method_behavior_bound_failure_is_error`, `generic_diagnostics::generic_receiver_method_behavior_bound_failure_is_error`, `generic_diagnostics::generic_ufc_function_behavior_bound_failure_is_error`, `generic_diagnostics::generic_behavior_bound_unknown_method_is_error`, `generic_diagnostics::generic_function_type_arg_annotation_arity_is_error`, `generic_diagnostics::generic_method_type_arg_annotation_arity_is_error`, `generic_diagnostics::closure_param_annotation_type_arg_arity_is_error`, `generic_diagnostics::closure_return_annotation_without_type_args_is_error`, `generic_diagnostics::cast_target_annotation_type_arg_arity_is_error`, `generic_diagnostics::cast_target_annotation_without_type_args_is_error`, `typechecker::tests::generic_behavior_bound_accepts_type_with_impl`, `typechecker::tests::generic_behavior_bound_accepts_inherited_behavior_impl`, `typechecker::tests::generic_behavior_bound_rejects_type_without_impl`, `tests/zen/generic_method_self.zen`, `tests/zen/generic_method_worklist.zen`, `tests/zen/behavior_json_generic_bound.zen`, `tests/zen/behavior_json_generic_bound_ufcs.zen`, `tests/zen/behavior_generic_default_method.zen`, `tests/zen/behavior_inherited_generic_dispatch.zen`, `tests/zen/multi_file_behavior_bound/main.zen`, `tests/zen/multi_file_behavior_inheritance/main.zen`, `tests/zen/multi_file_imported_behavior_impl/main.zen`, `tests/zen/multi_file_imported_behavior_default/main.zen`, `tests/zen/multi_file_imported_impl_imported_behavior/main.zen`, `tests/zen/multi_file_imported_child_parent_dispatch/main.zen`, `tests/zen/multi_file_imported_behavior_requires/main.zen`, `tests/zen/multi_file_imported_function_imported_behavior_bound/main.zen`, `tests/zen/multi_file_imported_function_param_type_dependency/main.zen`, `tests/zen/multi_file_imported_function_return_type_dependency/main.zen`, `tests/zen/multi_file_imported_function_imported_return_type_behavior/main.zen`, `tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen`, `integration::generated_c_call_definition_scan_reports_missing_generated_calls`, `integration::generated_c_definition_count_ignores_prototypes`, `integration::test_behavior_generic_default_method`, `integration::test_multi_file_imported_generic_behavior_defaults`, `integration::test_multi_file_imported_function_imported_behavior_bound`, `integration::test_multi_file_imported_function_param_type_dependency`, `integration::test_multi_file_imported_function_return_type_dependency`, `integration::test_multi_file_imported_function_imported_return_type_behavior`, `integration::test_multi_file_imported_generic_function_return_enum_dependency` |
| Behavior declarations `Name: behavior { method: (Self) Return }` | experimental | `parser::tests::parse_behavior_declaration`, `parser::tests::parse_public_behavior_declaration`, `resolver_phase2::resolver_records_public_visibility_for_exported_declarations`, `resolver_phase2::resolver_records_behavior_method_signatures`, `resolver_phase2::resolver_records_behavior_function_type_method_signatures`, `resolver_phase2::resolver_records_generic_behavior_function_type_method_signatures`, `resolver_phase2::resolver_records_behavior_default_method_body_locals`, `resolver_phase2::resolver_rejects_duplicate_behavior_method_names`, `resolver_phase2::resolver_rejects_duplicate_signature_parameter_names`, `typechecker::tests::behavior_declaration_collection`, `typechecker::tests::check_program_with_symbols_validates_resolver_behavior_method_signatures`, `typechecker::tests::check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures`, `typechecker::tests::check_program_with_symbols_validates_resolver_behavior_method_types`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_method_metadata`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_default_method_metadata`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures`, `typechecker::tests::check_program_with_symbols_requires_resolver_behavior_default_locals` |
| Explicit behavior impl blocks `Type.implements(Behavior) { ... }` | experimental | `parser::tests::parse_behavior_impl_block`, `parser::tests::parse_behavior_impl_with_generic_behavior_args`, `resolver_phase2::resolver_records_behavior_impl_and_requires_names`, `resolver_phase2::resolver_rejects_duplicate_behavior_impl_edges`, `resolver_phase2::resolver_records_behavior_impl_methods_as_value_symbols`, `resolver_phase2::resolver_records_behavior_impl_function_type_methods`, `resolver_phase2::resolver_records_behavior_impl_method_body_locals`, `typechecker::tests::check_program_with_symbols_validates_resolver_behavior_impl_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_impl_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_impl_refs`, `typechecker::tests::check_program_with_symbols_rejects_extra_resolver_behavior_impl_names`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata`, `typechecker::tests::check_program_with_symbols_validates_resolver_impl_method_signature`, `typechecker::tests::check_program_with_symbols_validates_resolver_impl_function_type_signature`, `typechecker::tests::check_program_with_symbols_requires_resolver_impl_method_body_locals`, `typechecker::tests::behavior_impl_with_required_method_passes`, `typechecker::tests::behavior_impl_missing_required_method_is_error`, `typechecker::tests::behavior_impl_can_omit_default_method`, `typechecker::tests::behavior_impl_overlapping_inherited_behavior_is_error`, `typechecker::tests::behavior_impl_generic_parent_overlap_is_error`, `typechecker::tests::behavior_impl_distinct_generic_specializations_do_not_overlap`, `typechecker::tests::behavior_impl_generic_behavior_without_type_args_is_error`, `typechecker::tests::behavior_impl_generic_behavior_with_type_args_passes_requires`, `typechecker::tests::behavior_impl_generic_behavior_substitutes_method_signature`, `generic_diagnostics::behavior_impl_for_unspecialized_generic_type_is_error`, `tests/zen/behavior_json_explicit_impl.zen`, `tests/zen/behavior_json_generic_association.zen`, `tests/zen/multi_file_imported_behavior_impl/main.zen`, `tests/zen/multi_file_imported_behavior_default/main.zen`, `tests/zen/multi_file_imported_impl_imported_behavior/main.zen`, `tests/zen/multi_file_imported_child_parent_dispatch/main.zen`, `tests/zen/multi_file_imported_behavior_requires/main.zen`, `integration::imported_private_behavior_impl_methods_are_not_directly_visible` |
| Type association assertions `.requires` | experimental | `parser::tests::parse_behavior_requires_assertion`, `parser::tests::parse_behavior_requires_with_generic_behavior_args`, `resolver_phase2::resolver_accepts_behavior_requires_known_type_and_behavior`, `resolver_phase2::resolver_rejects_duplicate_behavior_required_edges`, `resolver_phase2::resolver_records_behavior_impl_and_requires_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_behavior_required_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_required_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_required_refs`, `typechecker::tests::check_program_with_symbols_rejects_extra_resolver_behavior_required_names`, `typechecker::tests::behavior_requires_rejects_missing_impl`, `typechecker::tests::behavior_requires_generic_behavior_without_type_args_is_error`, `typechecker::tests::behavior_requires_generic_behavior_type_arg_arity_is_error`, `tests/zen/behavior_json_generic_association.zen`, `tests/zen/multi_file_imported_behavior_requires/main.zen`, `generic_diagnostics::behavior_requires_unspecialized_generic_type_is_error` |
| Behavior inheritance `.extends` | experimental | `parser::tests::parse_behavior_extends_declaration`, `parser::tests::parse_behavior_extends_with_generic_parent_args`, `resolver_phase2::resolver_accepts_behavior_extends_known_behaviors`, `resolver_phase2::resolver_rejects_duplicate_behavior_parent_edges`, `resolver_phase2::resolver_records_behavior_parent_names`, `resolver_phase2::resolver_records_generic_behavior_parent_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_behavior_parent_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_parent_names`, `typechecker::tests::check_program_with_symbols_validates_resolver_generic_behavior_parent_refs`, `typechecker::tests::check_program_with_symbols_rejects_extra_resolver_behavior_parent_names`, `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata`, `typechecker::tests::behavior_extends_requires_parent_methods`, `typechecker::tests::behavior_extends_generic_parent_requires_substituted_methods`, `typechecker::tests::behavior_extends_generic_parent_satisfies_specialized_requires`, `typechecker::tests::behavior_extends_duplicate_parent_is_error`, `typechecker::tests::behavior_extends_duplicate_generic_parent_is_error`, `typechecker::tests::behavior_extends_generic_parent_without_type_args_is_error`, `typechecker::tests::behavior_extends_cycle_is_error`, `typechecker::tests::behavior_extends_conflicting_method_signature_is_error`, `tests/zen/behavior_inherited_default_method.zen`, `tests/zen/behavior_generic_parent_inheritance.zen`, `tests/zen/multi_file_behavior_inheritance/main.zen`, `integration::imported_behavior_extends_requires_parent_methods`, `integration::imported_behavior_extends_imported_parent_requires_parent_methods`, `integration::imported_behavior_extends_requires_transitive_parent_methods` |

## Type, Module, ABI, Error, Effect, And Comptime Decisions

- `StaticString` is baked into the program. It denotes literal/static text with
  stable storage and compile-time length in the generated runtime layout. The
  allocator-backed `String` type is owned, dynamic text and carries allocator
  identity before it can be promoted as a stable construction target. Static
  string literals do not implicitly allocate or coerce into `String`; dynamic
  `String` construction must go through an explicit allocator-aware path once
  that path is promoted. String interpolation currently returns a
  `StaticString`-shaped non-owning view, but only literal text is guaranteed to
  be baked program storage; interpolation must not imply allocator-backed
  `String` construction. Source-level `String` use currently reports a gated
  allocator-backed text diagnostic until dynamic string ownership is promoted.
  Public diagnostics JSON pins both direct `String` annotations and `String`
  nested inside generic annotations through
  `emit_json_diagnostics_dynamic_string_gate_schema_matches_golden` and
  `emit_json_diagnostics_generic_dynamic_string_gate_schema_matches_golden`.
- `Sync/Async effects`: gated. `Sync` and `Async` are real effects in v1, not
  marker-only types. Sync code must not call async operations except through an
  explicit runtime blocking boundary. Async operations lower through checked task,
  queue, scheduler, yield, and await-like APIs. `@builtin.async_enqueue(...)`
  and `@builtin.async_yield()` currently report that async task enqueue and
  async yield are gated until Sync/Async effect checking and task lowering
  exist. Imports of std async runtime sketches are gated before loading
  aspirational scheduler/task source files, covered by
  `stdlib_async_runtime_import_is_gated_before_loading_sketch` and
  `module_graph_gates_stdlib_async_runtime_import_before_loading_sketch`, with
  public diagnostics JSON pinned by
  `emit_json_diagnostics_async_runtime_import_gate_schema_matches_golden`.
  Imports of std sync runtime sketches are likewise gated before loading
  aspirational channel source files, covered by
  `stdlib_sync_runtime_import_is_gated_before_loading_sketch` and
  `module_graph_gates_stdlib_sync_runtime_import_before_loading_sketch`, with
  public diagnostics JSON pinned by
  `emit_json_diagnostics_sync_runtime_import_gate_schema_matches_golden`.
  Atomic compiler intrinsics `@builtin.atomic_load(...)`,
  `@builtin.atomic_store(...)`, `@builtin.atomic_add(...)`,
  `@builtin.atomic_sub(...)`, `@builtin.atomic_cas(...)`,
  `@builtin.atomic_xchg(...)`, and `@builtin.fence()` report gated diagnostics
  until memory-order and Sync/Async effect semantics exist.
- `Typed allocators`: gated. v1 allocators are typed by allocated value and effect
  mode, such as `Allocator<T, Sync>` and `Allocator<T, Async>`. Sync allocation
  returns a direct checked result; async allocation returns a task/effect result.
  Until allocator lowering exists, these spellings produce gated diagnostics
  rather than unknown-type errors. `@builtin.raw_allocate(...)`,
  `@builtin.raw_deallocate(...)`, and `@builtin.raw_reallocate(...)` currently
  report raw memory operation gates until allocator ownership and effect
  semantics exist. Byte-memory intrinsics `@builtin.memcpy(...)`,
  `@builtin.memmove(...)`, `@builtin.memset(...)`, and `@builtin.memcmp(...)`
  report the same allocator/effect gate. Imports of std allocator sketches are
  gated before loading aspirational allocator source files, covered by
  `stdlib_allocator_import_is_gated_before_loading_sketch` and
  `module_graph_gates_stdlib_allocator_import_before_loading_sketch`, with
  public diagnostics JSON pinned by
  `emit_json_diagnostics_allocator_import_gate_schema_matches_golden`.
- `Type matching`: gated. Comptime type matching operates on typed metadata for
  primitives, structs, enums, fields, variants, behaviors, allocator modes, and
  effect modes. It is separate from runtime value matching.
  `@builtin.type_match<T>()` currently reports that comptime type matching is
  gated until typed metadata and derive lowering exist.
- `Ownership and raw pointer operations`: gated. Raw pointer offset, cast,
  integer conversion, load, and store intrinsics are compiler-owned operations
  that require ownership, pointer provenance, memory access, and layout
  semantics before promotion. Until then, `@builtin.gep(...)`,
  `@builtin.gep_struct(...)`, `@builtin.raw_ptr_cast(...)`,
  `@builtin.ptr_to_int(...)`, `@builtin.int_to_ptr(...)`,
  `@builtin.load<T>(...)`, and `@builtin.store<T>(...)` report ownership gates.
- `Host syscalls`: gated. Raw `@builtin.syscall0(...)` through
  `@builtin.syscall6(...)` require explicit host effect declarations and syscall
  ABI semantics before promotion, so they currently report host-effect gates.
- `Behavior association`: gated. Associated operations resolve by explicit impl,
  then generated impl, then declared fallback where the spec allows it. Ambiguity
  is a hard diagnostic.
- `AST traversal`: experimental. Raw AST traversal is for tooling and source
  transforms. Typed HIR traversal is required for semantic metaprogramming.
  Neither replaces compiler resolver, typechecker, effect checker, or MIR passes.
  `zen emit-json ast <file>` emits `semantic_status: "unchecked"` to make that
  boundary explicit.
- `Actors in std`: gated. Actors are a stdlib framework first, with promoted
  framework spellings `Actor`, `ActorRef`, `Mailbox`, and `Supervisor` built on
  effect-aware queues and typed allocators. No actor syntax is v1-stable yet,
  and promoted actor framework type spellings report gated diagnostics until
  std actor semantics exist; this includes generic and bare actor framework
  spellings, covered by `actor_framework_types_are_rejected_as_gated_not_unknown`
  and `bare_actor_framework_types_are_rejected_as_gated_not_unknown`. Public
  diagnostics JSON is pinned by
  `emit_json_diagnostics_actor_type_gate_schema_matches_golden` and
  `emit_json_diagnostics_bare_actor_type_gate_schema_matches_golden`.
  Imports of std actor framework sketches are also gated before loading
  aspirational actor source files, covered by
  `stdlib_actor_framework_import_is_gated_before_loading_sketch`,
  `module_graph_gates_stdlib_actor_framework_import_before_loading_sketch`, and
  `emit_json_diagnostics_actor_import_gate_schema_matches_golden`.
  `Channel` remains an experimental stdlib channel sketch until promoted, so it
  is not a global actor builtin type spelling.
- `JSON/YAML IR boundaries`: gated. JSON is the machine-readable exchange format
  for compiler-owned AST, typed HIR, MIR, symbol tables, type layouts,
  diagnostics, and deterministic build graphs. YAML is the human-authored
  format for target descriptions, ABI rules, intrinsic tables, allocator
  templates, backend options, and build graphs.
  Current JSON evidence includes resolved AST graph emission, resolver symbol
  table emission, checked typed program emission, machine-readable diagnostics
  emission, checked declaration-level HIR emission, checked layout emission,
  deterministic build graph emission, and validated target YAML emission through
  `zen emit-json ast <file>`, `zen emit-json symbols <file>`,
  `zen emit-json typed <file>`, `zen emit-json diagnostics <file>`,
  `zen emit-json hir <file>`, `zen emit-json layout <file>`,
  `zen emit-json build-graph <file>`, and
  `zen emit-json target-yaml <file>`. AST JSON includes
  `schema_version: 0` with `semantic_status: "unchecked"`. Hand-authored AST,
  symbols, typed, HIR, and diagnostics JSON inputs are rejected before the
  frontend treats them as Zen source or compiler-produced diagnostics, covered by
  `emit_json_ast_rejects_hand_authored_json_before_unchecked_ir_override`,
  `emit_json_symbols_rejects_hand_authored_json_before_resolver_override`,
  `emit_json_diagnostics_rejects_hand_authored_json_before_diagnostic_override`, and
  `emit_json_typed_rejects_hand_authored_json_before_checked_ir_override`.
  AST and symbols module-graph schemas are pinned by
  `emit_json_ast_module_graph_schema_matches_golden` and
  `emit_json_symbols_module_graph_schema_matches_golden`, covering portable
  module IDs, canonical path shape, imports, declarations, symbol namespaces,
  visibility, spans, and function signatures. Symbols JSON for explicit
  generic behavior association is pinned by
  `emit_json_symbols_generic_behavior_association_schema_matches_golden`,
  covering generic behavior method signatures plus concrete
  `Json<StaticString>` impl/require metadata on `Point`. Symbols JSON for
  generic method templates is pinned by
  `emit_json_symbols_generic_method_schema_matches_golden`, covering the
  resolver-owned `Box<T>` type symbol and `Box.get<T>` method symbol before
  checked typed specialization. Symbols JSON for generic non-behavior impl
  block methods is pinned by
  `emit_json_symbols_generic_type_impl_methods_schema_matches_golden`, covering
  resolver-owned `Box<T>` metadata plus `Box.get<T>` and `Box.replace<T>`
  method symbols restored from `Box<T>.impl`. Symbols JSON for generic
  `Self`-returning methods is pinned by
  `emit_json_symbols_generic_self_method_schema_matches_golden`, covering
  `Box.copy<T>` and `Option.copy<T>` method symbols with `self: Self` and
  return type `Self` on generic struct and enum receivers. Symbols JSON for
  generic method worklist templates is pinned by
  `emit_json_symbols_generic_method_worklist_schema_matches_golden`, covering
  resolver-owned `inner<T>` and `Box.get_inner<T>` symbols before checked
  worklist specialization emits concrete calls. Symbols JSON for generic Option enum templates
  is pinned by `emit_json_symbols_generic_option_schema_matches_golden`,
  covering the resolver-owned `Option<T>` type symbol, `None`/`Some` variant
  symbols, `Some` payload type `T`, and `unwrap_or<T>` signature. Symbols JSON
  for generic `Result<T, E>` enum templates is pinned by
  `emit_json_symbols_generic_result_schema_matches_golden`, covering the
  resolver-owned `Result<T, E>` type symbol, `Ok`/`Err` variant symbols, `Ok`
  payload type `T`, `Err` payload type `E`, and `unwrap_or<T, E>` signature.
  Symbols JSON for generic `Result<T, E>` enum method templates is pinned by
  `emit_json_symbols_generic_result_method_schema_matches_golden`, covering
  the resolver-owned `Result.unwrap_or<T, E>` method symbol with `self: Self`,
  fallback type `T`, and return type `T`.
  Symbols JSON
  for generic behavior-bound UFCS is pinned by
  `emit_json_symbols_generic_behavior_bound_ufcs_schema_matches_golden`,
  covering the generic `encode<T: Json<T>>` symbol, the concrete
  `Point.encode__Json_Point` impl symbol, and `Json<Point>` metadata on
  `Point`.
  Checked typed JSON for generic method specialization is pinned by
  `emit_json_typed_generic_method_schema_matches_golden`, covering the
  specialized `Box_i32` type and `Box.get_i32` method output. Checked typed
  JSON for generic method worklist specialization is pinned by
  `emit_json_typed_generic_method_worklist_schema_matches_golden`, covering
  `Box.get_inner_i32` calling the concrete `inner_i32` specialization from the
  method body. Checked typed
  JSON for generic `Option<T>` enum specialization is pinned by
  `emit_json_typed_generic_option_schema_matches_golden`, covering concrete
  `Option_i32` enum payloads, `unwrap_or_i32`, and typed call sites. Checked
  typed JSON for generic `Result<T, E>` enum specialization is pinned by
  `emit_json_typed_generic_result_schema_matches_golden`, covering concrete
  `Result_i32_StaticString` enum payloads, `unwrap_or_i32_StaticString`, and
  typed call sites. Checked typed JSON for generic `Result<T, E>` enum method
  specialization is pinned by
  `emit_json_typed_generic_result_method_schema_matches_golden`, covering
  concrete `Result.unwrap_or_i32_StaticString` call sites and method-body
  match typing. Nested generic typed JSON is pinned by
  `emit_json_typed_nested_generic_result_schema_matches_golden`, covering
  `Result_Option_i32_StaticString`, dependent `Option_i32`, specialized unwrap
  calls, and nested enum payload typing. Generic behavior association typed
  JSON is pinned by
  `emit_json_typed_generic_behavior_association_schema_matches_golden`,
  covering the concrete `Point.encode__Json_StaticString` association function
  and its typed call site. Generic behavior-bound UFCS typed JSON is pinned by
  `emit_json_typed_generic_behavior_bound_ufcs_schema_matches_golden`,
  covering `encode_Point` dispatch through the substituted `T: Json<T>` bound
  to the concrete `Point.encode__Json_Point` association function.
  Diagnostics JSON carries structured notes, suggested fixes, and context
  frames for agent/editor consumers; the `Type.derive(...)` feature gate now
  reports `context.kind = "feature_gate"` with a note pointing users to
  explicit `Type.implements(Behavior) { ... }` blocks, pinned by
  `emit_json_diagnostics_behavior_derive_gate_schema_matches_golden`. Gated generic
  association targets such as `Type<T>.derive(...)` report the same
  feature-gate context shape with a note to use non-generic explicit behavior associations
  until generic behavior target templates exist, pinned by
  `emit_json_diagnostics_generic_association_gate_schema_matches_golden`.
  Removed `return` keyword diagnostics are pinned by
  `emit_json_diagnostics_removed_return_schema_matches_golden`, including the
  stable code, span, and structured suggested fix payload for agent/editor
  consumers. Non-generic enum annotation type arguments are pinned by
  `emit_json_diagnostics_nongeneric_enum_annotation_type_args_schema_matches_golden`,
  covering the stable `E5002` payload for `Direction<i32>` without dependent
  followups. Non-generic struct constructor type arguments are pinned by
  `emit_json_diagnostics_nongeneric_struct_constructor_type_args_schema_matches_golden`,
  covering the stable `E5002` payload for `Point<i32> { x: 1 }` without field
  mismatch followups. Non-generic enum constructor type arguments are pinned by
  `emit_json_diagnostics_nongeneric_enum_constructor_type_args_schema_matches_golden`,
  covering the stable `E5002` payload for `Direction<i32>.North` without
  payload mismatch followups. Non-generic function call type arguments are
  pinned by `emit_json_diagnostics_nongeneric_function_type_args_schema_matches_golden`,
  covering the stable `E5002` payload for `id<i32>(1)` without argument
  followups. Non-generic module function call type arguments are pinned by
  `emit_json_diagnostics_nongeneric_module_function_type_args_schema_matches_golden`,
  covering the stable `E5002` payload for `io.println<i32>("bad")` without
  argument followups. Non-generic builtin function call type arguments are
  pinned by `emit_json_diagnostics_nongeneric_builtin_function_type_args_schema_matches_golden`,
  covering the stable `E5002` payload for `@builtin.panic<i32>("bad")` without
  argument followups. Non-generic method call type arguments are pinned by
  `emit_json_diagnostics_nongeneric_method_type_args_schema_matches_golden`,
  covering the stable `E5002` payload for `box.get<i32>()` without argument
  followups. `docs/DIAGNOSTICS.md` catalogs JSON-stable public diagnostic codes
  only after a golden fixture pins the code and diagnostic shape.
  Hand-authored build graph JSON inputs are rejected before generic build.zen
  path validation can stand in for the compiler-owned graph boundary, covered by
  `emit_json_build_graph_rejects_hand_authored_json_before_graph_override`.
  `zen emit-json hir <file>` emits `format: "zen.hir.v0"` with
  `schema_version: 0`, `semantic_status: "checked"`, and a declaration graph
  for checked types, enum variants/payloads, function parameters/returns, and
  globals, covered by `emit_json_hir_outputs_checked_declaration_graph` and
  `emit_json_hir_outputs_enum_function_and_global_declarations`. The checked
  declaration schema is also pinned by
  `emit_json_hir_declaration_schema_matches_golden`. HIR output for generic
  `Option<T>` and `Result<T, E>` specializations is pinned by
  `emit_json_hir_generic_option_schema_matches_golden` and
  `emit_json_hir_generic_result_schema_matches_golden`, covering concrete
  `Option_i32` and `Result_i32_StaticString` enum payloads plus specialized
  `unwrap_or_i32` and `unwrap_or_i32_StaticString` function signatures. Generic
  method worklist HIR output is pinned by
  `emit_json_hir_generic_method_worklist_schema_matches_golden`, covering
  concrete `Box_i32`, `inner_i32`, and `Box.get_inner_i32` declarations.
  Generic `Result<T, E>` enum method HIR output is pinned by
  `emit_json_hir_generic_result_method_schema_matches_golden`, covering
  concrete `Result_i32_StaticString` payloads and
  `Result.unwrap_or_i32_StaticString` method signature lowering.
  Nested
  generic HIR specialization is pinned by
  `emit_json_hir_nested_generic_result_schema_matches_golden`, covering
  `Result_Option_i32_StaticString`, `Option_i32`,
  `unwrap_result_Option_i32_StaticString`, and `unwrap_option_i32`.
  `zen emit-json mir <file>`
  emits `format: "zen.mir.v0"` with `schema_version: 0`,
  `semantic_status: "checked"`, and a minimal function/block/terminator graph
  over typed program bodies. The current schema covers locals, returns, calls,
  enum construction, match kind, match arm patterns/bindings, and block result
  summaries, covered by `emit_json_mir_outputs_checked_minimal_function_graph`
  and `emit_json_mir_outputs_match_arm_schema`. The checked minimal function
  schema is pinned by `emit_json_mir_minimal_function_schema_matches_golden`,
  and the checked match schema is pinned by
  `emit_json_mir_match_schema_matches_golden`. MIR output for generic
  `Option<T>` and `Result<T, E>` specializations is pinned by
  `emit_json_mir_generic_option_schema_matches_golden` and
  `emit_json_mir_generic_result_schema_matches_golden`, covering concrete
  `Option_i32` and `Result_i32_StaticString` enum construction plus
  match-arm lowering. Generic method worklist MIR output is pinned by
  `emit_json_mir_generic_method_worklist_schema_matches_golden`, covering
  `Box.get_inner_i32` lowering to a concrete `inner_i32(self.value)` call.
  Generic `Result<T, E>` enum method MIR output is pinned by
  `emit_json_mir_generic_result_method_schema_matches_golden`, covering
  `Result.unwrap_or_i32_StaticString` lowering to an enum match over
  `Result_i32_StaticString.Ok` and `Result_i32_StaticString.Err`.
  Nested generic MIR specialization is pinned by
  `emit_json_mir_nested_generic_result_schema_matches_golden`, covering nested
  `Result_Option_i32_StaticString.Ok(Option_i32.Some(...))`, dependent
  `Option_i32`, specialized calls, and match-arm lowering for both result and
  option unwrap helpers. Generic behavior association MIR is pinned by
  `emit_json_mir_generic_behavior_association_schema_matches_golden`, covering
  `Point.encode__Json_StaticString` return lowering and the typed call nested
  inside `io_println`.
  Generic behavior association HIR is pinned by
  `emit_json_hir_generic_behavior_association_schema_matches_golden`, covering
  the checked `Point` declaration plus concrete `Point.encode__Json_StaticString`
  association function signature in the declaration graph.
  Hand-authored
  JSON IR inputs to `zen emit-json hir <file>` and `zen emit-json mir <file>` are
  rejected at the compiler-owned schema boundary before any type or layout
  override can be accepted, covered by
  `emit_json_hir_rejects_hand_authored_json_before_ir_override` and
  `emit_json_mir_rejects_hand_authored_json_before_ir_override`.
  `zen emit-json layout <file>` emits checked compiler-owned layout JSON with
  `schema_version: 0` for the current stable subset, including primitive sizes,
  baked `StaticString`, pointer/slice/array layout entries, struct field
  offsets, and simple enum variant tags/payloads, covered by
  `emit_json_layout_outputs_checked_type_layouts` and
  `emit_json_layout_outputs_compound_type_layout_entries`. The checked basic
  primitive, `StaticString`, dynamic `String`, and struct field-offset schema is
  pinned by `emit_json_layout_basic_schema_matches_golden`, and the checked
  compound layout schema is pinned by
  `emit_json_layout_compound_schema_matches_golden`. Layout output for generic
  `Option<T>` and `Result<T, E>` specializations is pinned by
  `emit_json_layout_generic_option_schema_matches_golden` and
  `emit_json_layout_generic_result_schema_matches_golden`, covering concrete
  `Option_i32` and `Result_i32_StaticString` enum sizes, alignments, variants,
  and payload field offsets. Nested generic layout is pinned by
  `emit_json_layout_nested_generic_result_schema_matches_golden`, covering
  `Result_Option_i32_StaticString` size/alignment and payload offsets for both
  `Option_i32` and `StaticString` payload variants. Hand-authored layout JSON
  inputs are rejected at the compiler-owned layout schema boundary before any
  ABI override can be accepted, covered by
  `emit_json_layout_rejects_hand_authored_json_before_layout_override`.
  AST JSON is explicitly
  marked unchecked; symbols JSON is explicitly marked resolved;
  typed JSON is explicitly marked checked; diagnostics JSON is explicitly
  marked diagnostic. semantic acceptance must use typed JSON, diagnostics,
  check, build, or test paths. Hand-authored target YAML validates through a
  minimal target schema plus an optional current-backend schema into
  `zen.target.v0` JSON with `schema_version: 0`. The current C backend schema
  accepts optional `backend.c_flags`, rejects empty C flag entries, and rejects
  compiler-owned layout overrides or unsupported backend code generators,
  covered by
  `emit_json_target_yaml_validates_minimal_target_schema`,
  `emit_json_target_yaml_validates_backend_schema`,
  `emit_json_target_yaml_validates_c_backend_flags`,
  `emit_json_target_yaml_backend_schema_matches_golden`,
  `emit_json_target_yaml_rejects_empty_c_backend_flags`,
  `emit_json_target_yaml_rejects_layout_overrides`, and
  `emit_json_target_yaml_rejects_unsupported_backend_codegen`.
- `build.zen`: constrained. `zen check build.zen` validates the deterministic
  graph and verifies declared target sources exist, `zen emit build.zen` emits
  target C for one graph target, `zen build build.zen` compiles executable
  targets through that graph, and direct `zen build.zen` aliases that same graph
  build path. `zen test build.zen` compiles and runs test graph targets.
  `zen emit-json build-graph <build.zen>` emits `format: "zen.build_graph.v0"`,
  `schema_version: 0`, and `semantic_status: "deterministic"` with the
  constrained graph payload. `emit_json_build_graph_project_schema_matches_golden`
  pins the canonical project build graph JSON schema, and
  `emit_json_build_graph_host_effect_schema_matches_golden` pins the
  declared/used host-effect arrays. `emit_json_build_graph_target_metadata_schema_matches_golden`
  pins library exports plus executable dependency and feature metadata.
  Executable target dependencies compile before their dependents. Test targets
  are lowered, emitted in build graph JSON, compiled, and run through the
  constrained test command. Library targets are lowered and emitted in build
  graph JSON, but library execution remains gated. Build and test execution
  reject dependencies on gated target kinds, and reject `packages`/`link`
  fields, until deterministic package/link semantics exist. Target dependency
  and feature metadata arrays are lowered and emitted in build graph JSON.
  Target dependencies must reference known graph
  targets, may not point back to the same target, and may not form dependency
  cycles; dependency cycles are rejected before execution.
  Legacy
  `emit-json ast|symbols|typed|diagnostics` modes for `build.zen` are explicitly
  rejected with a diagnostic that points to `emit-json build-graph`.
- Errors: `Result<T, E>` and `.raise()` are v1 design goals, but `.raise()` is
  gated until typechecked propagation and lowering are implemented.
- ABI: stable layout JSON exists for primitives, baked `StaticString`, pointers,
  slices, arrays, structs, and simple enums. Full options/results, closures,
  and function pointer ABI compatibility remain gated until broader layout
  tests exist.

## Feature Matrix

| Feature | Status | Gate |
|---|---|---|
| Lexer/parser for tested fixtures | implemented | Existing unit and integration tests |
| Local module loading | implemented | Existing integration tests |
| Typechecked C backend for tested fixtures | implemented | `cargo test --tests` |
| README and contributor truth assertions | implemented | `tests/docs_truth.rs` |
| Strict resolver, symbol IDs, privacy | implemented | Resolver/module/privacy tests |
| AST/symbols JSON emission | constrained | `emit_json_ast_command_outputs_resolved_module_graph` checks unchecked AST module-graph output, `emit_json_ast_module_graph_schema_matches_golden` pins the unchecked AST module graph schema, `emit_json_symbols_command_outputs_module_symbol_tables` checks resolved symbol table output, `emit_json_symbols_module_graph_schema_matches_golden` pins the resolved symbols module graph schema, `emit_json_symbols_generic_method_schema_matches_golden` pins resolver-owned `Box<T>` and `Box.get<T>` generic template symbols, `emit_json_symbols_generic_type_impl_methods_schema_matches_golden` pins resolver-owned `Box<T>.impl` method-template symbols for `Box.get<T>` and `Box.replace<T>`, `emit_json_symbols_generic_self_method_schema_matches_golden` pins resolver-owned `Box.copy<T>` and `Option.copy<T>` `Self -> Self` method-template symbols, `emit_json_symbols_generic_method_worklist_schema_matches_golden` pins resolver-owned `inner<T>` and `Box.get_inner<T>` worklist template symbols, `emit_json_symbols_generic_option_schema_matches_golden` pins resolver-owned `Option<T>`, `None`/`Some`, and `unwrap_or<T>` symbols, `emit_json_symbols_generic_result_schema_matches_golden` pins resolver-owned `Result<T, E>`, `Ok`/`Err`, and `unwrap_or<T, E>` symbols, `emit_json_symbols_generic_result_method_schema_matches_golden` pins resolver-owned `Result.unwrap_or<T, E>` method-template symbols, `emit_json_symbols_generic_behavior_association_schema_matches_golden` pins generic behavior method signatures and `Point` association metadata for `Json<StaticString>`, `emit_json_symbols_generic_behavior_bound_ufcs_schema_matches_golden` pins generic behavior-bound UFCS resolver symbols for `encode<T: Json<T>>`, `Point.encode__Json_Point`, and `Json<Point>`, `emit_json_ast_rejects_hand_authored_json_before_unchecked_ir_override`, `emit_json_symbols_rejects_hand_authored_json_before_resolver_override` |
| Diagnostics JSON emission | constrained | `emit_json_diagnostics_command_outputs_machine_readable_errors` checks machine-readable errors, `emit_json_diagnostics_removed_return_schema_matches_golden` pins removed-return suggested fix schema, `emit_json_diagnostics_behavior_derive_gate_schema_matches_golden` pins feature-gate context for reserved generated behavior association, `emit_json_diagnostics_generic_association_gate_schema_matches_golden` pins feature-gate context for reserved generic behavior association targets, `emit_json_diagnostics_typed_allocator_effect_gate_schema_matches_golden` pins the resolver-backed typed allocator/effect gate instead of unknown-type fallbacks, `emit_json_diagnostics_sync_effect_gate_schema_matches_golden` pins direct Sync effect mode spelling as a gated reserved type instead of an unknown type, `emit_json_diagnostics_async_effect_gate_schema_matches_golden` pins direct Async effect mode spelling as a gated reserved type instead of an unknown type, `emit_json_diagnostics_dynamic_string_gate_schema_matches_golden` pins allocator-backed dynamic `String` as a gated reserved type instead of an unknown type, `emit_json_diagnostics_actor_type_gate_schema_matches_golden` pins std actor framework type spelling as a gated reserved type instead of an unknown type, `emit_json_diagnostics_type_match_gate_schema_matches_golden` pins the gated comptime type-match intrinsic diagnostic instead of unknown-builtin fallbacks, `emit_json_diagnostics_async_intrinsic_gate_schema_matches_golden` pins the gated async scheduler intrinsic diagnostic instead of unknown-builtin fallbacks, `emit_json_diagnostics_atomic_gate_schema_matches_golden` pins the gated atomic intrinsic diagnostic instead of unknown-builtin fallbacks, `emit_json_diagnostics_syscall_gate_schema_matches_golden` pins the gated syscall intrinsic diagnostic instead of unknown-builtin fallbacks, `emit_json_diagnostics_raw_allocate_gate_schema_matches_golden` pins the gated raw allocation intrinsic diagnostic instead of unknown-builtin fallbacks, `emit_json_diagnostics_byte_memory_gate_schema_matches_golden` pins the gated byte-memory intrinsic diagnostic instead of unknown-builtin fallbacks, `emit_json_diagnostics_raw_pointer_gate_schema_matches_golden` pins the gated raw pointer intrinsic diagnostic instead of unknown-builtin fallbacks, `emit_json_diagnostics_range_gate_schema_matches_golden` pins the gated range-expression diagnostic until range typing is promoted, `emit_json_diagnostics_raise_gate_schema_matches_golden` pins gated Result propagation through `.raise()` until propagation lowering is promoted, `emit_json_diagnostics_await_gate_schema_matches_golden` pins gated task waiting through `.await()` until Sync/Async effect checking and task lowering are promoted, `emit_json_diagnostics_generic_function_arity_schema_matches_golden` pins a hard generic function call arity diagnostic without followups, `emit_json_diagnostics_generic_function_type_arg_annotation_arity_schema_matches_golden` pins a hard generic function type-argument annotation arity diagnostic without followups, `emit_json_diagnostics_generic_method_type_arg_annotation_arity_schema_matches_golden` pins a hard generic method type-argument annotation arity diagnostic without followups, `emit_json_diagnostics_generic_method_type_arg_annotation_missing_args_schema_matches_golden` pins a hard generic method type-argument annotation missing-arguments diagnostic without followups, `emit_json_diagnostics_closure_param_annotation_type_arg_arity_schema_matches_golden` pins a hard closure parameter generic annotation arity diagnostic, `emit_json_diagnostics_closure_return_annotation_missing_args_schema_matches_golden` pins a hard closure return generic annotation missing-arguments diagnostic, `emit_json_diagnostics_cast_target_annotation_type_arg_arity_schema_matches_golden` pins a hard cast target generic annotation arity diagnostic, `emit_json_diagnostics_cast_target_annotation_missing_args_schema_matches_golden` pins a hard cast target generic annotation missing-arguments diagnostic, `emit_json_diagnostics_nested_generic_annotation_inner_arity_schema_matches_golden` pins a hard nested generic annotation inner arity diagnostic, `emit_json_diagnostics_nested_generic_instantiation_inner_arity_schema_matches_golden` pins a hard nested generic instantiation inner arity diagnostic, `emit_json_diagnostics_function_type_parameter_annotation_arity_schema_matches_golden` pins a hard function type parameter generic annotation arity diagnostic, `emit_json_diagnostics_function_type_return_annotation_missing_args_schema_matches_golden` pins a hard function type return generic annotation missing-arguments diagnostic, `emit_json_diagnostics_pointer_inner_generic_annotation_arity_schema_matches_golden` pins a hard pointer inner generic annotation arity diagnostic, `emit_json_diagnostics_slice_inner_generic_annotation_missing_args_schema_matches_golden` pins a hard slice inner generic annotation missing-arguments diagnostic, `emit_json_diagnostics_array_inner_generic_annotation_arity_schema_matches_golden` pins a hard array inner generic annotation arity diagnostic, `emit_json_diagnostics_generic_struct_local_annotation_arity_schema_matches_golden` pins a hard generic struct local annotation arity diagnostic without variable-mismatch followups, `emit_json_diagnostics_generic_struct_local_annotation_missing_args_schema_matches_golden` pins a hard generic struct local annotation missing-arguments diagnostic without variable-mismatch followups, `emit_json_diagnostics_generic_enum_local_annotation_arity_schema_matches_golden` pins a hard generic enum local annotation arity diagnostic without variable-mismatch followups, `emit_json_diagnostics_generic_enum_local_annotation_missing_args_schema_matches_golden` pins a hard generic enum local annotation missing-arguments diagnostic without variable-mismatch followups, `emit_json_diagnostics_generic_struct_constructor_arity_schema_matches_golden` pins a hard generic struct constructor arity diagnostic without followups, `emit_json_diagnostics_generic_struct_constructor_missing_args_schema_matches_golden` pins a hard generic struct constructor missing-arguments diagnostic without followups, `emit_json_diagnostics_generic_enum_constructor_arity_schema_matches_golden` pins a hard generic enum constructor arity diagnostic without followups, `emit_json_diagnostics_generic_enum_constructor_missing_args_schema_matches_golden` pins a hard generic enum constructor missing-arguments diagnostic without followups, `emit_json_diagnostics_generic_struct_annotation_arity_schema_matches_golden` pins a hard generic struct annotation arity diagnostic without followups, `emit_json_diagnostics_nongeneric_struct_annotation_type_args_schema_matches_golden` pins a hard non-generic struct annotation type-argument diagnostic without followups, `emit_json_diagnostics_generic_enum_annotation_arity_schema_matches_golden` pins a hard generic enum annotation arity diagnostic without followups, `emit_json_diagnostics_generic_struct_annotation_missing_args_schema_matches_golden` pins a hard generic struct annotation missing-arguments diagnostic without followups, `emit_json_diagnostics_generic_enum_annotation_missing_args_schema_matches_golden` pins a hard generic enum annotation missing-arguments diagnostic without followups, `emit_json_diagnostics_generic_result_method_arity_schema_matches_golden` pins a hard generic `Result<T, E>` method arity diagnostic without followups, `emit_json_diagnostics_generic_result_method_bound_schema_matches_golden` pins a hard generic behavior-bound diagnostic without method-body followups, `emit_json_diagnostics_generic_result_method_inference_schema_matches_golden` pins a hard generic inference conflict without argument/return followups, `emit_json_diagnostics_generic_behavior_overlap_schema_matches_golden` pins a behavior implementation coherence diagnostic for overlapping generic parent/child impls, `emit_json_diagnostics_generic_requires_missing_impl_schema_matches_golden` pins an explicit `.requires` missing implementation diagnostic for `Json<StaticString>`, `emit_json_diagnostics_duplicate_generic_requires_schema_matches_golden` pins duplicate `.requires` resolver diagnostics for `Json<StaticString>`, `emit_json_diagnostics_duplicate_generic_impl_schema_matches_golden` pins duplicate `.implements` resolver diagnostics for `Json<StaticString>`, `emit_json_diagnostics_generic_requires_arity_schema_matches_golden` pins generic `.requires` behavior-reference arity diagnostics, `emit_json_diagnostics_generic_impl_arity_schema_matches_golden` pins generic `.implements` behavior-reference arity diagnostics, `emit_json_diagnostics_generic_extends_arity_schema_matches_golden` pins generic `.extends` behavior-reference arity diagnostics, `emit_json_diagnostics_rejects_hand_authored_json_before_diagnostic_override`; `docs/DIAGNOSTICS.md` catalogs the currently JSON-stable public codes, while broader diagnostic-code coverage is still required |
| Typed JSON emission | constrained | `emit_json_typed_command_outputs_checked_program` checks `semantic_status: "checked"`, `emit_json_typed_generic_method_schema_matches_golden` pins generic method specialization for `Box_i32` / `Box.get_i32`, `emit_json_typed_generic_method_worklist_schema_matches_golden` pins checked worklist specialization for `Box.get_inner_i32` calling concrete `inner_i32`, `emit_json_typed_generic_option_schema_matches_golden` pins concrete `Option<i32>` enum specialization and typed `unwrap_or_i32` calls, `emit_json_typed_generic_result_schema_matches_golden` pins concrete `Result<i32, StaticString>` enum specialization and typed `unwrap_or_i32_StaticString` calls, `emit_json_typed_generic_result_method_schema_matches_golden` pins concrete `Result.unwrap_or_i32_StaticString` call sites and method-body match typing, `emit_json_typed_nested_generic_result_schema_matches_golden` pins nested `Result<Option<i32>, StaticString>` and dependent `Option<i32>` typed payloads/calls, `emit_json_typed_generic_behavior_association_schema_matches_golden` pins concrete generic behavior association dispatch to `Point.encode__Json_StaticString`, `emit_json_typed_generic_behavior_bound_ufcs_schema_matches_golden` pins substituted generic behavior-bound UFCS dispatch from `encode_Point` to `Point.encode__Json_Point`, `emit_json_typed_rejects_hand_authored_json_before_checked_ir_override`; broader typed JSON schemas still required |
| HIR JSON emission | constrained | `emit_json_hir_outputs_checked_declaration_graph` checks `schema_version: 0`, `emit_json_hir_outputs_enum_function_and_global_declarations` covers enum variants/payloads, function params/returns, and globals, `emit_json_hir_declaration_schema_matches_golden` pins the checked declaration schema, `emit_json_hir_generic_method_worklist_schema_matches_golden` pins concrete `Box_i32`, `inner_i32`, and `Box.get_inner_i32` worklist declaration output, `emit_json_hir_generic_option_schema_matches_golden` pins concrete `Option<i32>` enum/function specialization output, `emit_json_hir_generic_result_schema_matches_golden` pins concrete `Result<i32, StaticString>` enum/function specialization output, `emit_json_hir_nested_generic_result_schema_matches_golden` pins nested `Result<Option<i32>, StaticString>` and dependent `Option<i32>` specialization output, `emit_json_hir_generic_behavior_association_schema_matches_golden` pins concrete generic behavior association function signatures, `emit_json_hir_generic_behavior_bound_ufcs_schema_matches_golden` pins generic behavior-bound UFCS declaration output for `encode_Point` and `Point.encode__Json_Point`, `emit_json_hir_rejects_hand_authored_json_before_ir_override`; broader golden tests still required |
| MIR JSON emission | constrained | `emit_json_mir_outputs_checked_minimal_function_graph` checks `schema_version: 0`, `emit_json_mir_minimal_function_schema_matches_golden` pins the checked minimal function/block/terminator schema, `emit_json_mir_outputs_match_arm_schema` covers match kind, arm patterns/bindings, and block results, `emit_json_mir_match_schema_matches_golden` pins the checked match schema, `emit_json_mir_generic_method_worklist_schema_matches_golden` pins `Box.get_inner_i32` lowering to a concrete `inner_i32(self.value)` call, `emit_json_mir_generic_option_schema_matches_golden` pins concrete `Option<i32>` enum construction and match lowering, `emit_json_mir_generic_result_schema_matches_golden` pins concrete `Result<i32, StaticString>` enum construction and match lowering, `emit_json_mir_nested_generic_result_schema_matches_golden` pins nested `Result<Option<i32>, StaticString>` construction, calls, and match lowering, `emit_json_mir_generic_behavior_association_schema_matches_golden` pins concrete generic behavior association call lowering, `emit_json_mir_generic_behavior_bound_ufcs_schema_matches_golden` pins lowered generic behavior-bound UFCS dispatch from `encode_Point` to `Point.encode__Json_Point`, `emit_json_mir_rejects_hand_authored_json_before_ir_override`; broader golden tests still required |
| Layout JSON emission | constrained | `emit_json_layout_outputs_checked_type_layouts` checks primitive, `StaticString`, and struct layout facts, `emit_json_layout_basic_schema_matches_golden` pins the checked primitive/static/dynamic string and struct field-offset schema, `emit_json_layout_outputs_compound_type_layout_entries` covers pointer, raw pointer, slice, array, and enum payload entries, `emit_json_layout_compound_schema_matches_golden` pins the checked compound layout schema, `emit_json_layout_generic_option_schema_matches_golden` pins concrete `Option<i32>` enum size/alignment and payload offsets, `emit_json_layout_generic_result_schema_matches_golden` pins concrete `Result<i32, StaticString>` enum size/alignment and payload offsets, `emit_json_layout_nested_generic_result_schema_matches_golden` pins nested `Result<Option<i32>, StaticString>` size/alignment and payload offsets, `emit_json_layout_rejects_hand_authored_json_before_layout_override`; broader ABI schemas still required |
| Target/build YAML validation | constrained | `emit_json_target_yaml_validates_minimal_target_schema` checks `schema_version: 0`, `emit_json_target_yaml_validates_backend_schema`, `emit_json_target_yaml_validates_c_backend_flags`, `emit_json_target_yaml_backend_schema_matches_golden` pins canonical backend target JSON, `emit_json_target_yaml_rejects_empty_c_backend_flags`, `emit_json_target_yaml_rejects_layout_overrides`, `emit_json_target_yaml_rejects_unsupported_backend_codegen`; broader ABI/backend option schemas still required |
| Build graph JSON emission | constrained | `emit_json_build_graph_outputs_project_build_graph` checks top-level deterministic metadata, `emit_json_build_graph_project_schema_matches_golden` pins the canonical project graph JSON schema, `emit_json_build_graph_host_effect_schema_matches_golden` pins declared/used host-effect arrays, `emit_json_build_graph_target_metadata_schema_matches_golden` pins library exports plus dependency/feature arrays, `emit_json_build_graph_outputs_library_target` covers library targets, and `emit_json_build_graph_outputs_target_dependencies_and_features` covers dependency/feature arrays; package/link semantics remain gated |
| Developer UX and Agent UX | constrained/gated | Existing public docs and repo hygiene tests prevent unsupported `zen-lsp` claims, stale generated editor packages, and duplicate public examples. `docs/PHASE_PLAN.md` records the MoonBit-style toolchain integration target, while this spec keeps VS Code extension, `zen lsp`, agent-readable diagnostics, machine-readable project graph, structured fix suggestions, and quiet deterministic commands as explicit promotion criteria before editor/agent workflows can be advertised as implemented |
| Behaviors and type association | gated | Positive/negative behavior solver tests |
| `Sync/Async effects` | gated | `async_scheduler_intrinsics_are_rejected_as_gated_not_unknown`, `effect_await_is_rejected_until_async_lowering_exists`, `atomic_intrinsics_are_rejected_as_effect_gates`; effect checker positive/negative tests still required |
| `Typed allocators` | gated | `dynamic_string_type_is_rejected_as_allocator_backed_gate`, `typed_allocator_type_is_rejected_as_gated_not_unknown`, `sync_and_async_typed_allocator_modes_are_rejected_as_gated_not_unknown`, `raw_memory_intrinsics_are_rejected_as_allocator_gates`, `byte_memory_intrinsics_are_rejected_as_allocator_gates`; positive allocator semantics tests still required |
| Comptime type matching | gated | `comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown`, `primitive_and_enum_type_match_intrinsics_are_rejected_as_gated_not_unknown`; type metadata and derive tests still required |
| Ownership and raw pointer operations | gated | `raw_pointer_intrinsics_are_rejected_as_ownership_gates`; ownership/resource tests still required |
| Host syscalls | gated | `syscall_intrinsics_are_rejected_as_host_effect_gates`; host-effect declaration and ABI tests still required |
| Actors in std | gated | Mailbox, scheduling, supervisor tests |
| `build.zen` check/emit/build/test/direct execution | constrained | Deterministic graph validation, test and library target graph emission, test target execution, target C emission, dependency-ordered multi-executable build tests, and legacy emit-json rejection tests |
| Existing broad stdlib files | experimental | Must compile before promotion |
| Formatter, package manager, alternate backends | removed from v1 claims | Reintroduce only with tests and binaries |

Constrained `build.zen` execution already has positive and negative evidence:
Deterministic build graph compiles executable and test targets, while build
scripts using undeclared host side effects are rejected. AST traversal also has
minimum boundary evidence: AST JSON emits a parsed tooling view and
semantically invalid programs still fail through typed JSON. The remaining
backlog below is for v1 areas without that minimum positive/negative proof.

## Required Test Backlog

Every remaining v1 effect/type-match/allocator/actor/IR claim needs at least one
planned positive test and one planned negative test before implementation.

| Area | Planned Positive Test | Planned Negative Test |
|---|---|---|
| `Sync/Async effects` | Async function may enqueue, yield, and call async operation through checked APIs | Sync function calling async operation without blocking boundary is rejected |
| `Typed allocators` | `Allocator<i32, Sync>` returns a checked pointer result and propagates into a container | `Allocator<i32, Sync>` cannot satisfy an `Allocator<i32, Async>` parameter |
| Type matching | `to_json<T>` derive branches on struct and enum metadata | Ambiguous or unreachable type-match arm is diagnosed |
| Generated/fallback behavior association | Generated `Json<T>` derive fallback is used only when no explicit impl exists | Missing or ambiguous generated/fallback behavior impl is rejected |
| Actors in std | Actor mailbox send/receive works with scheduler and allocator integration | Actor using async mailbox from sync-only context is rejected |
| JSON/YAML IR boundaries | Checked layout JSON, checked MIR JSON, and target YAML validate against schemas | Hand-authored JSON IR cannot override compiler-owned types or layouts |

Generated/fallback behavior association syntax is reserved but not implemented:
`Type.derive(Json)` currently parses into a reserved AST declaration and then
reports an explicit resolver gate, covered by
`parser::tests::parse_generated_behavior_derive_association` and
`resolver_gates_generated_behavior_derive_association`.
The diagnostics JSON path reports that gate over the full
`Type.derive(...)` association call and includes feature-gate context plus an
explicit-impl note, covered by
`emit_json_diagnostics_spans_full_gated_behavior_derive_association`.
Gated generic association targets such as `Type<T>.derive(Json<T>)` are also
localized over the full reserved association target in diagnostics JSON and now
carry feature-gate context plus a non-generic-association note, covered by
`emit_json_diagnostics_spans_full_gated_generic_association_target` and pinned by
`emit_json_diagnostics_generic_association_gate_schema_matches_golden`.

## Stdlib Gate

Files under `stdlib/` are experimental unless a test proves they parse, typecheck,
and build through the same compiler path as user modules. Aspirational stdlib
APIs must not be described as implemented until promoted by tests.
