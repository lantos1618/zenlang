use super::*;

mod core_semantics;
mod focused_tests;
mod generic_diagnostics;
mod integration;
mod intrinsics;
mod ir_json;
mod lexer_and_monomorphize;
mod lexer_tests;
mod parser_enums;
mod parser_keywords;
mod resolver_declaration_validation;
mod resolver_metadata_helpers;
mod resolver_phase2;
mod resolver_validation;
mod resolver_validation_support;
mod thresholds;
mod typechecker_expressions;
mod typechecker_program;
mod typechecker_resolver_metadata_collection;

#[test]
fn generic_diagnostics_file_size_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/generic_diagnostics.rs");
    let annotations =
        read("tests/docs_truth/repo_hygiene/file_size/generic_diagnostics/annotations.rs");
    let bounds = read("tests/docs_truth/repo_hygiene/file_size/generic_diagnostics/bounds.rs");
    let call_sites =
        read("tests/docs_truth/repo_hygiene/file_size/generic_diagnostics/call_sites.rs");
    let constructors =
        read("tests/docs_truth/repo_hygiene/file_size/generic_diagnostics/constructors.rs");
    let method_type_args =
        read("tests/docs_truth/repo_hygiene/file_size/generic_diagnostics/method_type_args.rs");

    assert!(
        root.lines().count() < 80,
        "generic_diagnostics.rs should route focused file-size guard modules"
    );
    for module_name in [
        "annotations",
        "bounds",
        "call_sites",
        "constructors",
        "method_type_args",
    ] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "generic_diagnostics.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        annotations.contains("fn generic_composite_annotation_tests_stay_split_by_type_shape"),
        "annotation guards should live in generic_diagnostics/annotations.rs"
    );
    assert!(
        bounds.contains("fn generic_bound_diagnostic_tests_stay_split_by_bound_surface"),
        "bound guards should live in generic_diagnostics/bounds.rs"
    );
    assert!(
        call_sites
            .contains("fn generic_call_site_annotation_tests_stay_split_by_annotation_surface"),
        "call-site guards should live in generic_diagnostics/call_sites.rs"
    );
    assert!(
        constructors
            .contains("fn generic_constructor_diagnostic_tests_stay_split_by_aggregate_kind"),
        "constructor guards should live in generic_diagnostics/constructors.rs"
    );
    assert!(
        method_type_args.contains("fn generic_method_type_arg_tests_stay_split_by_call_surface"),
        "method type-arg guards should live in generic_diagnostics/method_type_args.rs"
    );
}
