use super::*;
use crate::parser::keywords::ParserModuleRoot;

impl Parser {
    pub(super) fn parse_builtin_module_call_expr(&mut self) -> Result<Expression, CompileError> {
        let (_, span) = self.advance();
        self.expect(&Token::Dot)?;
        let (func_name, _) = self.expect_identifier()?;

        let type_args = if matches!(self.peek(), Token::Lt) {
            self.parse_type_arg_list()?
        } else {
            Vec::new()
        };

        self.expect(&Token::LParen)?;
        let args = self.parse_arg_list()?;
        let end = self.expect(&Token::RParen)?;

        Ok(Expression::FunctionCall {
            name: func_name,
            module: Some(ParserModuleRoot::AtBuiltin.as_str().to_string()),
            type_args,
            args,
            span: span.merge(end),
        })
    }

    pub(super) fn parse_std_module_root_expr(&mut self) -> Result<Expression, CompileError> {
        let (_, span) = self.advance();
        self.expect(&Token::Dot)?;
        let mut module_parts = Vec::new();
        let (first, _) = self.expect_identifier()?;
        module_parts.push(first);

        while matches!(self.peek(), Token::Dot) {
            let saved = self.pos;
            self.advance();
            if let Token::Identifier(_) = self.peek() {
                let (part, _) = self.expect_identifier()?;
                module_parts.push(part);
            } else {
                self.pos = saved;
                break;
            }
        }

        let func_name = module_parts.pop().unwrap();
        let module = ParserModuleRoot::AtStd.join_module_parts(&module_parts);

        if matches!(self.peek(), Token::LParen) {
            self.advance();
            let args = self.parse_arg_list()?;
            let end = self.expect(&Token::RParen)?;
            Ok(Expression::FunctionCall {
                name: func_name,
                module: Some(module),
                type_args: Vec::new(),
                args,
                span: span.merge(end),
            })
        } else {
            Ok(Expression::MemberAccess {
                object: Box::new(Expression::Identifier { name: module, span }),
                field: func_name,
                span: span.merge(self.prev_span()),
            })
        }
    }
}
