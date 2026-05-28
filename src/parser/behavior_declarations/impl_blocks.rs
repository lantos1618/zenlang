use super::*;

impl Parser {
    pub(in crate::parser) fn parse_behavior_impl_block_with_type_params(
        &mut self,
        type_name: String,
        type_params: Vec<TypeParam>,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        let (behavior, behavior_type_args, _) = self.parse_parenthesized_behavior_ref()?;
        self.skip_newlines();

        let type_args = type_params
            .iter()
            .map(|param| AstType::Named(param.name.clone()))
            .collect();
        let (methods, end) = self.parse_impl_block_methods(&type_params)?;
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

        Ok((behavior, behavior_type_args, behavior_span.merge(end)))
    }
}
