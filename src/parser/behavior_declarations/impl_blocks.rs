use super::*;

impl Parser {
    pub(in crate::parser) fn parse_behavior_impl_block(
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

    pub(in crate::parser) fn parse_behavior_impl_block_with_type_params(
        &mut self,
        type_name: String,
        type_params: Vec<TypeParam>,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        let (behavior, _, behavior_type_args, _) = self.parse_parenthesized_behavior_ref()?;
        self.skip_newlines();
        self.expect(&Token::LBrace)?;

        let type_args = type_params
            .iter()
            .map(|param| AstType::Named(param.name.clone()))
            .collect();
        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let mut method = self.parse_declaration()?;
            Self::prepend_impl_type_params(&mut method, &type_params);
            methods.push(method);
        }

        let end = self.expect(&Token::RBrace)?;
        Ok(Declaration::ImplBlock {
            type_name,
            behavior: Some(behavior),
            behavior_type_args,
            type_args,
            methods,
            span: name_span.merge(end),
        })
    }

    pub(super) fn parse_parenthesized_behavior_ref(
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
}
