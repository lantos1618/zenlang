use super::*;

#[test]
fn generated_c_support_scanners_stay_split_by_responsibility() {
    let support = read("tests/integration/support.rs");
    let root = read("tests/integration/support/generated_c.rs");
    let definitions = read("tests/integration/support/generated_c/definitions.rs");
    let calls = read("tests/integration/support/generated_c/calls.rs");
    let public_helpers = [
        "assert_c_call_resolves_to_single_definition",
        "assert_generated_c_calls_resolve_to_definitions",
        "assert_generated_c_function_definitions_are_unique",
        "has_c_call_outside_signature",
        "undefined_generated_c_calls",
    ];
    let private_helpers = [
        "assert_c_function_definition",
        "assert_c_function_definition_count",
        "assert_c_call_resolves_to_definition",
        "c_function_definitions",
        "c_function_definition_name",
        "generated_c_calls_on_line",
        "is_any_c_function_signature_line",
        "is_c_function_signature_line",
        "is_tracked_c_function_name",
        "is_untracked_c_call_name",
    ];

    for module in ["mod calls;", "mod definitions;"] {
        assert!(
            root.contains(module),
            "generated-C support should load focused scanner module `{module}`"
        );
    }

    assert_helpers_live_in_focused_module(
        &root,
        &definitions,
        &[
            "fn c_function_definitions(",
            "fn c_function_definition_name(",
        ],
        "definition scanner",
    );
    assert_helpers_live_in_focused_module(
        &root,
        &calls,
        &[
            "fn generated_c_calls_on_line(",
            "fn is_any_c_function_signature_line(",
            "fn is_untracked_c_call_name(",
        ],
        "call scanner",
    );

    let mut root_public_helpers = public_functions(&root);
    root_public_helpers.sort_unstable();
    let mut expected_public_helpers = public_helpers;
    expected_public_helpers.sort_unstable();
    assert_eq!(
        root_public_helpers, expected_public_helpers,
        "generated-C support root should expose only the stable facade helpers"
    );

    let support_export_block = support
        .split("pub use generated_c::{")
        .nth(1)
        .and_then(|tail| tail.split("};").next())
        .expect("integration support should re-export generated-C helpers from one block");
    let mut support_exports: Vec<_> = support_export_block
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    support_exports.sort_unstable();
    assert_eq!(
        support_exports, expected_public_helpers,
        "integration support should re-export only the generated-C facade helpers"
    );

    for helper in public_helpers {
        assert!(
            support.contains(helper),
            "integration support should re-export generated-C facade helper `{helper}`"
        );
    }
    for helper in private_helpers {
        assert!(
            !support_export_block
                .split(',')
                .map(str::trim)
                .any(|export| export == helper),
            "integration support should not re-export generated-C scanner/internal helper `{helper}`"
        );
    }
}

fn assert_helpers_live_in_focused_module(
    root: &str,
    focused: &str,
    helpers: &[&str],
    responsibility: &str,
) {
    for helper in helpers {
        assert!(
            !root.contains(helper),
            "generated-C support root should not own {responsibility} `{helper}`"
        );
        assert!(
            focused.contains(helper),
            "generated-C {responsibility} should own `{helper}`"
        );
    }
}

fn public_functions(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("pub fn "))
        .filter_map(|line| line.split('(').next())
        .collect()
}
