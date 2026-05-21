use super::*;
use crate::ast::BehaviorMethod;
use crate::parser::keywords::ParserBehaviorKeyword;

mod method_signatures;

type ParenthesizedBehaviorRef = (String, Span, Vec<AstType>, Span);

impl Parser {
    pub(super) fn parse_behavior_def(
        &mut self,
        name: String,
        type_params: Vec<TypeParam>,
        public: bool,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.expect(&Token::Colon)?;
        self.skip_newlines();
        let (keyword, keyword_span) = self.expect_identifier()?;
        if keyword.parse::<ParserBehaviorKeyword>() != Ok(ParserBehaviorKeyword::Behavior) {
            return Err(CompileError::Syntax(
                format!("expected behavior declaration, found `{keyword}`"),
                Some(keyword_span),
            ));
        }
        self.skip_newlines();
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let method_start = self.peek_span();
            let (method_name, _) = self.expect_identifier()?;
            self.expect(&Token::Colon)?;
            let (params, return_type, default_body) =
                self.parse_behavior_method_signature(method_start)?;
            methods.push(BehaviorMethod {
                name: method_name,
                params,
                return_type,
                default_body,
                span: method_start.merge(self.prev_span()),
            });
            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }

        let end = self.expect(&Token::RBrace)?;
        Ok(Declaration::Behavior {
            name,
            type_params,
            methods,
            public,
            span: name_span.merge(end),
        })
    }

    pub(super) fn parse_behavior_impl_block(
        &mut self,
        type_name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        let (behavior, _, behavior_type_args, _) = self.parse_parenthesized_behavior_ref()?;
        self.skip_newlines();
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            methods.push(self.parse_declaration()?);
        }

        let end = self.expect(&Token::RBrace)?;
        Ok(Declaration::ImplBlock {
            type_name,
            behavior: Some(behavior),
            behavior_type_args,
            type_args: Vec::new(),
            methods,
            span: name_span.merge(end),
        })
    }

    fn parse_parenthesized_behavior_ref(
        &mut self,
    ) -> Result<ParenthesizedBehaviorRef, CompileError> {
        self.skip_newlines();
        self.expect(&Token::LParen)?;
        self.skip_newlines();
        let (behavior, behavior_span) = self.expect_identifier()?;
        let behavior_type_args = if matches!(self.peek(), Token::Lt) {
            self.parse_type_arg_list()?
        } else {
            Vec::new()
        };
        self.skip_newlines();
        let end = self.expect(&Token::RParen)?;

        Ok((behavior, behavior_span, behavior_type_args, end))
    }

    pub(super) fn parse_behavior_requires(
        &mut self,
        type_name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        let (behavior, behavior_span, behavior_type_args, end) =
            self.parse_parenthesized_behavior_ref()?;
        Ok(Declaration::Requires {
            type_name,
            behavior,
            behavior_type_args,
            span: name_span.merge(behavior_span).merge(end),
        })
    }

    pub(super) fn parse_behavior_derive(
        &mut self,
        type_name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        let (behavior, behavior_span, behavior_type_args, end) =
            self.parse_parenthesized_behavior_ref()?;
        Ok(Declaration::Derive {
            type_name,
            behavior,
            behavior_type_args,
            span: name_span.merge(behavior_span).merge(end),
        })
    }

    pub(super) fn parse_behavior_extends(
        &mut self,
        behavior: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        let (parent, parent_span, parent_type_args, end) =
            self.parse_parenthesized_behavior_ref()?;
        Ok(Declaration::BehaviorExtends {
            behavior,
            parent,
            parent_type_args,
            span: name_span.merge(parent_span).merge(end),
        })
    }
}
