// Navigation module - LSP navigation handlers (go-to-definition, references, etc.)

pub mod definition;
pub mod highlight;
pub mod imports;
pub mod references;
pub mod scope;
pub mod struct_fields;
pub mod type_definition;
pub mod ufc;
pub mod utils;

// Re-export all handlers
pub use definition::handle_definition;
pub use highlight::handle_document_highlight;
pub use references::handle_references;
pub use type_definition::handle_type_definition;

// Re-export utility functions for other modules
pub use utils::{find_symbol_at_position, find_symbol_definition_in_content};
