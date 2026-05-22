//! Integration tests for the Zen compiler pipeline.
//!
//! For each `.zen` file in `tests/zen/`, runs the full pipeline:
//! lex -> parse -> typecheck -> C codegen -> compile with cc -> run -> verify output.

#[path = "integration/support.rs"]
mod support;

use support::*;

#[path = "integration/discovery.rs"]
mod discovery;
#[path = "integration/frontend_diagnostics.rs"]
mod frontend_diagnostics;
#[path = "integration/generated_c_assertions.rs"]
mod generated_c_assertions;
#[path = "integration/import_visibility.rs"]
mod import_visibility;
#[path = "integration/import_visibility_dependencies.rs"]
mod import_visibility_dependencies;
#[path = "integration/import_visibility_private_methods.rs"]
mod import_visibility_private_methods;
#[path = "integration/multi_file_fixtures.rs"]
mod multi_file_fixtures;
#[path = "integration/multi_file_phase5_fixtures.rs"]
mod multi_file_phase5_fixtures;
#[path = "integration/public_examples.rs"]
mod public_examples;
#[path = "integration/runtime_fixtures.rs"]
mod runtime_fixtures;
#[path = "integration/single_file_fixtures.rs"]
mod single_file_fixtures;

#[path = "integration/generic_specializations.rs"]
mod generic_specializations;

#[path = "integration/cli_build.rs"]
mod cli_build;
