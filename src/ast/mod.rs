pub mod declarations;
pub mod expressions;
pub mod patterns;
pub mod statements;
pub mod typed;
pub mod types;

pub(crate) use declarations::type_param_names;
pub use declarations::{BehaviorMethod, Declaration, EnumVariant, StructField, TypeParam};
pub use expressions::{BinaryOp, Expression, MatchArm, StringPart, UnaryOp};
pub use patterns::Pattern;
pub use statements::Statement;
pub(crate) use types::{
    behavior_impl_method_symbol_key, behavior_ref_display, method_symbol_key, named_type_arg_names,
    named_type_arg_params, symbol_key_part, type_params_from_names,
};
pub use types::{AstType, BuiltinGenericTypeName, BuiltinTypeName, Param};

use crate::error::FileId;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub file_id: FileId,
}
