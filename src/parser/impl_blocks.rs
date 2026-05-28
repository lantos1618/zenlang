use super::*;

impl Parser {
    pub(super) fn parse_impl_block_with_type_params(
        &mut self,
        type_name: String,
        type_params: Vec<TypeParam>,
        name_span: Span,
    ) -> Result<Declaration, CompileError> {
        self.skip_newlines();
        self.expect(&Token::Assign)?;
        self.skip_newlines();
        let (methods, end) = self.parse_impl_block_methods(&type_params)?;

        Ok(Declaration::ImplBlock {
            type_name,
            behavior: None,
            behavior_type_args: Vec::new(),
            type_args: Vec::new(),
            methods,
            span: name_span.merge(end),
        })
    }

    pub(in crate::parser) fn parse_impl_block_methods(
        &mut self,
        type_params: &[TypeParam],
    ) -> Result<(Vec<Declaration>, Span), CompileError> {
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let mut method = self.parse_declaration()?;
            Self::prepend_impl_type_params(&mut method, type_params);
            methods.push(method);
        }
        let end = self.expect(&Token::RBrace)?;

        Ok((methods, end))
    }

    pub(super) fn prepend_impl_type_params(decl: &mut Declaration, impl_type_params: &[TypeParam]) {
        let Declaration::Function { type_params, .. } = decl else {
            return;
        };
        if impl_type_params.is_empty() {
            return;
        }
        let mut merged = impl_type_params.to_vec();
        merged.append(type_params);
        *type_params = merged;
    }
}
