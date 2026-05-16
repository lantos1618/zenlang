use super::*;
use crate::ast::BehaviorMethod;

type BehaviorMethodSignature = (Vec<Param>, Option<AstType>, Option<Expression>);

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
        if keyword != "behavior" {
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

    fn parse_behavior_method_signature(
        &mut self,
        method_start: Span,
    ) -> Result<BehaviorMethodSignature, CompileError> {
        self.skip_newlines();
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        let mut index = 0usize;
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RParen) {
                break;
            }

            let param_span = self.peek_span();
            let (name, ty) = if matches!(
                (self.peek(), self.tokens.get(self.pos + 1).map(|(t, _)| t)),
                (Token::Identifier(_), Some(Token::Colon))
            ) {
                let (name, _) = self.expect_identifier()?;
                self.expect(&Token::Colon)?;
                (name, self.parse_type()?)
            } else {
                let ty = self.parse_type()?;
                let name = format!("__arg{index}");
                index += 1;
                (name, ty)
            };
            params.push(Param {
                name,
                ty,
                mutable: false,
                span: param_span.merge(self.prev_span()),
            });

            self.skip_newlines();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RParen)?;
        self.skip_newlines();

        let return_type = if matches!(self.peek(), Token::LBrace | Token::Comma | Token::RBrace) {
            None
        } else {
            Some(self.parse_type()?)
        };
        self.skip_newlines();

        let default_body = if matches!(self.peek(), Token::LBrace) {
            Some(self.parse_block_expression()?)
        } else {
            None
        };

        if params.is_empty() && return_type.is_none() && default_body.is_none() {
            return Err(CompileError::Syntax(
                "behavior method must include a signature".to_string(),
                Some(method_start),
            ));
        }

        Ok((params, return_type, default_body))
    }

    pub(super) fn parse_impl_block(
        &mut self,
        type_name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.skip_newlines();
        self.expect(&Token::Assign)?;
        self.skip_newlines();
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let decl = self.parse_declaration()?;
            methods.push(decl);
        }
        let end = self.expect(&Token::RBrace)?;

        Ok(Declaration::ImplBlock {
            type_name,
            behavior: None,
            behavior_type_args: Vec::new(),
            type_args: Vec::new(),
            methods,
            span: name_span.merge(end),
        })
    }

    pub(super) fn parse_behavior_impl_block(
        &mut self,
        type_name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.skip_newlines();
        self.expect(&Token::LParen)?;
        self.skip_newlines();
        let (behavior, _) = self.expect_identifier()?;
        let behavior_type_args = if matches!(self.peek(), Token::Lt) {
            self.parse_type_arg_list()?
        } else {
            Vec::new()
        };
        self.skip_newlines();
        self.expect(&Token::RParen)?;
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

    pub(super) fn parse_behavior_requires(
        &mut self,
        type_name: String,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
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
        Ok(Declaration::Requires {
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
        self.skip_newlines();
        self.expect(&Token::LParen)?;
        self.skip_newlines();
        let (parent, parent_span) = self.expect_identifier()?;
        let parent_type_args = if matches!(self.peek(), Token::Lt) {
            self.parse_type_arg_list()?
        } else {
            Vec::new()
        };
        self.skip_newlines();
        let end = self.expect(&Token::RParen)?;
        Ok(Declaration::BehaviorExtends {
            behavior,
            parent,
            parent_type_args,
            span: name_span.merge(parent_span).merge(end),
        })
    }
}
