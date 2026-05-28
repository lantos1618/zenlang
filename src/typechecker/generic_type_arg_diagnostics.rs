use crate::ast::AstType;
use crate::error::Span;

use super::TypeChecker;

impl TypeChecker {
    pub(in crate::typechecker) fn reject_nongeneric_type_args(
        &mut self,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) {
        if type_args.is_empty() {
            return;
        }

        self.push_error(
            crate::error::CompilerDiagnosticCode::E5002,
            format!("non-generic {kind} `{name}` does not accept type arguments"),
            span,
        );
    }

    pub(in crate::typechecker) fn report_generic_type_arg_arity(
        &mut self,
        kind: &str,
        name: &str,
        expected: usize,
        found: usize,
        span: Span,
    ) {
        self.push_error(
            crate::error::CompilerDiagnosticCode::E5001,
            format!("generic {kind} `{name}` expects {expected} type arguments, found {found}"),
            span,
        );
    }

    pub(in crate::typechecker) fn validate_type_arg_arity(
        &mut self,
        kind: &str,
        name: &str,
        expected: usize,
        type_args: &[AstType],
        span: Span,
    ) -> bool {
        if expected == type_args.len() {
            return true;
        }

        if expected == 0 {
            self.reject_nongeneric_type_args(kind, name, type_args, span);
        } else {
            self.report_generic_type_arg_arity(kind, name, expected, type_args.len(), span);
        }
        false
    }
}
