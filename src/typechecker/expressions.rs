//! Expression checking — check_function and check_expr.
#![allow(clippy::result_large_err)]

mod aggregate_constructors;
mod aggregate_support;
mod call_support;
mod call_validation;
mod closure_forms;
mod control_flow_support;
mod dispatch;
mod enum_variant;
mod function_checking;
mod gated_methods;
mod generic_call_validation;
mod method_call_support;
mod return_flow;
mod simple_forms;
mod struct_literal;

use crate::ast::expressions::StringPart;
use crate::ast::typed::*;
use crate::ast::{AstType, Expression, Param};
use crate::error::{Diagnostic, Span};

use super::closures::collect_captures;
use super::monomorphize_inference::InferenceConflict;
use super::monomorphize_types::concrete_name_matches_generic;
use super::{BehaviorBound, FuncInfo, TypeChecker};
