use crate::ast::{gated_builtin_type_name, is_builtin_type_name, AstType, TypeParam};
use crate::error::{Diagnostic, Span};
use crate::resolver::{Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(in crate::resolver) fn validate_type_ref(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        ast_type: &AstType,
        span: Span,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if let Some(gated) = gated_builtin_type_name(name) {
                    diagnostics.push(Diagnostic::error("E0202", gated.gate_message(), span));
                    return;
                }
                if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
            }
            AstType::Generic { name, type_args } => {
                if let Some(gated) = gated_builtin_type_name(name) {
                    diagnostics.push(Diagnostic::error("E0202", gated.gate_message(), span));
                    return;
                }
                if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        span,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            AstType::Array { elem, .. }
            | AstType::Slice(elem)
            | AstType::Ptr(elem)
            | AstType::MutPtr(elem)
            | AstType::RawPtr(elem) => {
                self.validate_type_ref(
                    table,
                    type_params,
                    elem,
                    span,
                    allow_self_type,
                    diagnostics,
                );
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_type_ref(
                        table,
                        type_params,
                        param,
                        span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                self.validate_type_ref(table, type_params, ret, span, allow_self_type, diagnostics);
            }
            AstType::SelfType => {
                if !allow_self_type {
                    diagnostics.push(Diagnostic::error(
                        "E0204",
                        "Self type is only valid in method or behavior contexts",
                        span,
                    ));
                }
            }
            AstType::I8
            | AstType::I16
            | AstType::I32
            | AstType::I64
            | AstType::U8
            | AstType::U16
            | AstType::U32
            | AstType::U64
            | AstType::Usize
            | AstType::F32
            | AstType::F64
            | AstType::Bool
            | AstType::Void
            | AstType::Str
            | AstType::String
            | AstType::Inferred => {}
        }
    }

    pub(in crate::resolver) fn is_known_type_name(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        name: &str,
    ) -> bool {
        if is_builtin_type_name(name) {
            return true;
        }
        table.lookup(Namespace::Type, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
            || type_params.iter().any(|type_param| type_param.name == name)
    }
}
