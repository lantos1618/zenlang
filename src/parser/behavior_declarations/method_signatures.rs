use crate::ast::{AstType, Expression, Param};
use crate::error::{CompileError, Span};
use crate::lexer::Token;
use crate::parser::core::Parser;

type BehaviorMethodSignature = (Vec<Param>, Option<AstType>, Option<Expression>);

impl Parser {
    pub(super) fn parse_behavior_method_signature(
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
}
