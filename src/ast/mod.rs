pub mod declarations;
pub mod expressions;
pub mod patterns;
pub mod statements;
pub mod typed;
pub mod types;

// Re-export the most commonly used types at the `ast` level.
pub use declarations::{BehaviorMethod, Declaration, EnumVariant, StructField, TypeParam};
pub use expressions::{BinaryOp, Expression, MatchArm, StringPart, UnaryOp};
pub use patterns::Pattern;
pub use statements::Statement;
pub(crate) use types::{behavior_type_args_match_target_params, named_type_arg_names};
pub use types::{
    gated_builtin_type_name, is_builtin_type_name, AstType, BuiltinGenericTypeName,
    BuiltinTypeName, Param,
};

use crate::error::FileId;
use serde::Serialize;

/// Program — the top-level container produced by the parser for a single file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub file_id: FileId,
}
