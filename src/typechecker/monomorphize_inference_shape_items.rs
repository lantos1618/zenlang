use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::monomorphize_inference::InferenceConflict;
use super::monomorphize_inference_substitution::{
    ast_type_substitutions, substitute_inference_ast_type,
};
use super::TypeChecker;

impl TypeChecker {
    pub(super) fn match_inference_shape_items<'a>(
        &self,
        shape_params: &[String],
        expected_type_args: &[AstType],
        items: impl IntoIterator<Item = (&'a AstType, &'a Type)>,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        let substitutions = ast_type_substitutions(shape_params, expected_type_args);
        for (expected, actual) in items {
            let expected = substitute_inference_ast_type(expected, &substitutions);
            self.match_type_param(&expected, actual, type_params, map, conflicts);
        }
    }
}
