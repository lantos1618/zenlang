pub mod c;

use crate::ast::typed::TypedProgram;

/// Trait for code generation backends.
pub trait Backend {
    /// Generate output from a typed program. Returns the generated source code.
    fn generate(&self, program: &TypedProgram) -> Result<String, String>;
}
