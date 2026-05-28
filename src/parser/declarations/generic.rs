use super::*;
use crate::parser::keywords::BEHAVIOR_KEYWORD;

impl Parser {
    pub(in crate::parser) fn parse_generic_declaration(
        &mut self,
        name: String,
        type_params: Vec<TypeParam>,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.skip_newlines();
        match self.peek() {
            Token::Assign => self.parse_function_def(name, type_params, public, name_span),
            Token::Dot => self.parse_dot_declaration(name, type_params, public, name_span),
            Token::Colon if self.is_struct_def() => {
                self.parse_struct_def_with_params(name, type_params, public, name_span)
            }
            Token::Colon if self.colon_is_followed_by_identifier(BEHAVIOR_KEYWORD) => {
                self.parse_behavior_def(name, type_params, public, name_span)
            }
            Token::Colon => self.parse_enum_def_with_params(name, type_params, public, name_span),
            _ => Err(CompileError::Syntax(
                "expected '=' or ':' after generic type parameters".to_string(),
                Some(self.peek_span()),
            )),
        }
    }

    pub(super) fn parse_dot_declaration(
        &mut self,
        name: String,
        type_params: Vec<TypeParam>,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.advance();
        let (method_name, _method_span) = self.expect_identifier()?;
        self.skip_newlines();

        if let Ok(keyword) = method_name.parse::<TypeDeclarationKeyword>() {
            return match keyword {
                TypeDeclarationKeyword::Impl => {
                    self.parse_impl_block_with_type_params(name, type_params, name_span)
                }
                TypeDeclarationKeyword::Implements => {
                    self.parse_behavior_impl_block_with_type_params(name, type_params, name_span)
                }
                TypeDeclarationKeyword::Requires if type_params.is_empty() => {
                    self.parse_behavior_requires(name, name_span)
                }
                TypeDeclarationKeyword::Extends if type_params.is_empty() => {
                    self.parse_behavior_extends(name, name_span)
                }
                TypeDeclarationKeyword::Derive if type_params.is_empty() => {
                    self.parse_behavior_derive(name, name_span)
                }
                keyword => self.reject_gated_generic_association_target(keyword, name_span),
            };
        }

        self.parse_method_declaration(name, method_name, type_params, public, name_span)
    }

    fn reject_gated_generic_association_target<T>(
        &mut self,
        keyword: TypeDeclarationKeyword,
        name_span: Span,
    ) -> Result<T, CompileError> {
        let span = if matches!(self.peek(), Token::LParen) {
            self.advance();
            let behavior_span = self.expect_identifier().map(|(_, span)| span).ok();
            if matches!(self.peek(), Token::Lt) {
                let _ = self.parse_type_arg_list();
            }
            self.skip_newlines();
            match self.expect(&Token::RParen) {
                Ok(end) => name_span.merge(end),
                Err(_) => behavior_span
                    .map(|span| name_span.merge(span))
                    .unwrap_or_else(|| name_span.merge(self.prev_span())),
            }
        } else {
            name_span.merge(self.prev_span())
        };
        Err(CompileError::Syntax(
            format!(
                "generic association target `Type<T>.{keyword}` is gated; use non-generic `{keyword}` associations or keep the generic behavior target deferred to docs/V1_SPEC.md"
            ),
            Some(span),
        ))
    }
}
