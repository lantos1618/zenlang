use super::core::Parser;
use crate::ast::ExternalFunction;
use crate::error::Result;
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_external_function(&mut self) -> Result<ExternalFunction> {
        let name = self.expect_identifier("external function name")?;

        self.expect_symbol(':')?;

        self.expect_symbol('(')?;

        let mut args = vec![];
        let mut is_varargs = false;

        if self.current_token != Token::Symbol(')') {
            loop {
                if self.try_consume_operator("...") {
                    is_varargs = true;
                    break;
                }

                // Check if we have a parameter name (optional)
                if let Token::Identifier(_param_name) = &self.current_token {
                    // Check if the next token is ':'
                    if self.peek_token == Token::Symbol(':') {
                        // Skip the parameter name and ':'
                        self.next_token(); // skip param name
                        self.next_token(); // skip ':'
                    }
                }

                let arg_type = self.parse_type()?;
                args.push(arg_type);

                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if !self.try_consume_symbol(',') {
                    return Err(self
                        .syntax_error("Expected ',' or ')' in external function parameter list"));
                }
            }
        }
        self.next_token(); // consume ')'

        let return_type = self.parse_type()?;

        Ok(ExternalFunction {
            name,
            args,
            return_type,
            is_varargs,
        })
    }
}
