//! Typechecker — transforms untyped AST → TypedProgram.
//!
//! Pipeline:
//! 1. **Collect**: Register all struct/enum/function/behavior signatures
//! 2. **Resolve**: Resolve type references (Named("Foo") → Struct fields)
//! 3. **Check**: Type-check function bodies, produce TypedExpression
//!
//! The typechecker NEVER defaults unknown types to I32. If a type can't be
//! resolved, it's an error.

mod behavior_associations;
mod behavior_impl_methods;
mod behavior_impl_signature_collection;
mod behavior_impl_support;
mod behavior_impl_validation;
mod behavior_ref_metadata;
mod closures;
mod declaration_collection;
mod declaration_collection_ast;
mod declaration_collection_ast_behaviors;
mod declaration_collection_ast_callables;
mod declaration_collection_resolver_semantic_tasks;
mod declaration_collection_resolver_tasks;
mod environment;
mod expressions;
mod gated_intrinsics;
mod generic_bound_validation;
mod generic_type_reference_walker;
mod generic_type_validation;
mod import_roots;
mod monomorphize;
mod monomorphize_dependencies;
mod monomorphize_inference;
mod monomorphize_inference_shapes;
mod monomorphize_method_self;
mod monomorphize_names;
mod monomorphize_specialized_type_names;
mod monomorphize_specialized_type_refs;
mod monomorphize_specialized_types;
mod monomorphize_substitution;
mod monomorphize_types;
mod patterns;
mod program_checking;
mod program_globals;
mod program_impl_blocks;
mod program_module_graph;
mod program_type_defs;
mod resolve;
mod resolve_binary_ops;
mod resolver_backed_collection;
mod resolver_lookup;
mod resolver_metadata_collection;
mod resolver_validation;
mod scope_management;
mod self_type_validation;
mod semantic_validation;
mod semantic_validation_struct_defaults;
mod statements;
mod std_runtime_calls;

use std::collections::{HashMap, HashSet, VecDeque};

use crate::ast::typed::*;
use crate::ast::{
    self, behavior_type_args_match_target_params, named_type_arg_names, AstType, BehaviorMethod,
    Declaration, EnumVariant, Expression, Param, StructField, TypeParam,
};
use crate::error::{Diagnostic, Span};
use crate::module_system::{ResolvedModule, ResolvedModuleGraph};
use crate::resolver::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, MethodSignatureMetadata, Namespace, Symbol,
    SymbolTable, TypeParameterBoundMetadata, TypeParameterBoundRefMetadata,
};

pub use environment::{BehaviorBound, BehaviorInfo, EnumInfo, FuncInfo, StructInfo};
pub(crate) use environment::{
    GenericBehaviorImplTemplate, GenericFunctionTemplate, SourceModuleDependencies,
    TemplateDependencyEntry, TemplateDependencyState,
};

include!("declaration_tasks.rs");
include!("state.rs");

impl TypeChecker {
    fn behavior_type_arg_substitutions(
        &mut self,
        behavior: &str,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) -> Option<HashMap<String, AstType>> {
        let Some(info) = self.behaviors.get(behavior).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                "E6006",
                format!("undefined behavior `{}`", behavior),
                span,
            ));
            return None;
        };

        if info.type_params.is_empty() && !type_args.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic behavior `{}` does not accept type arguments",
                    behavior
                ),
                span,
            ));
            return None;
        }

        if info.type_params.len() != type_args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic behavior `{}` expects {} type arguments, found {}",
                    behavior,
                    info.type_params.len(),
                    type_args.len()
                ),
                span,
            ));
            return None;
        }

        let ast_substitutions: HashMap<String, AstType> = info
            .type_params
            .iter()
            .cloned()
            .zip(type_args.iter().cloned())
            .collect();
        let type_substitutions: HashMap<String, Type> = info
            .type_params
            .iter()
            .zip(type_args.iter())
            .filter_map(|(param, arg)| {
                if ast_type_references_type_param(arg, scoped_type_params) {
                    None
                } else {
                    Some((param.clone(), self.resolve_type(arg)))
                }
            })
            .collect();
        let error_count = self
            .diagnostics
            .iter()
            .filter(|diag| diag.is_error())
            .count();
        self.check_generic_bounds(&info.type_param_bounds, &type_substitutions, span);
        if self
            .diagnostics
            .iter()
            .filter(|diag| diag.is_error())
            .count()
            > error_count
        {
            return None;
        }

        Some(ast_substitutions)
    }
}

include!("resolver_callable_signatures.rs");

#[cfg(test)]
mod tests;
