use super::core::Parser;
use crate::ast::Function;
use crate::error::Result;
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_function(&mut self) -> Result<Function> {
        // Function name
        let name = self.expect_identifier("function name")?;

        // Parse generic type parameters if present: <T: Trait1 + Trait2, U, ...>
        let type_params = self.parse_type_parameters()?;

        // Check for ':' or '=' for function definition OR '(' for direct parameters
        // Valid syntaxes:
        // - name = (params) return_type { ... }
        // - name : (params) return_type = { ... }
        // - name<T>(params) return_type { ... }  (generic function direct syntax)
        let uses_equals_syntax = if self.current_token == Token::Symbol('(') {
            // Direct parameter syntax (no : or = before params)
            true // treat like equals syntax for simplicity
        } else if self.try_consume_symbol(':') {
            false
        } else if self.try_consume_operator("=") {
            true
        } else {
            return Err(self.syntax_error(
                "Expected ':' or '=' after function name for type definition, or '(' for parameters"
            ));
        };

        // Parameters
        self.expect_symbol('(')?;

        let mut args = vec![];
        let mut is_varargs = false;

        if self.current_token != Token::Symbol(')') {
            loop {
                // Check for variadic function syntax: ... at the end
                if self.try_consume_operator("...") {
                    is_varargs = true;
                    // After ..., expect closing paren
                    if self.current_token != Token::Symbol(')') {
                        return Err(
                            self.syntax_error("Expected ')' after '...' in variadic function")
                        );
                    }
                    break;
                }

                // Parameter name
                let param_name = self.expect_identifier("parameter name")?;

                // Parameter type - support both : and :: for now
                // Special handling for 'self' parameter - type is optional
                let param_type = if param_name == "self"
                    && self.current_token != Token::Symbol(':')
                    && self.current_token != Token::Operator("::".to_string())
                {
                    // For 'self' without explicit type, use a placeholder type
                    crate::ast::AstType::Generic {
                        name: "Self".to_string(),
                        type_args: Vec::new(),
                    }
                } else {
                    let _is_double_colon = if self.try_consume_symbol(':') {
                        false
                    } else if self.try_consume_operator("::") {
                        true
                    } else {
                        return Err(self.syntax_error("Expected ':' or '::' after parameter name"));
                    };

                    self.parse_type()?
                };
                args.push((param_name, param_type));

                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if !self.try_consume_symbol(',') {
                    return Err(self.syntax_error("Expected ',' or ')' in parameter list"));
                }

                // After consuming comma, check for variadic syntax before looping
                if self.try_consume_operator("...") {
                    is_varargs = true;
                    // After ..., expect closing paren
                    if self.current_token != Token::Symbol(')') {
                        return Err(
                            self.syntax_error("Expected ')' after '...' in variadic function")
                        );
                    }
                    break;
                }
            }
        }
        self.next_token(); // consume ')'

        // Parse return type (required in zen, comes directly after parentheses)
        let return_type = if uses_equals_syntax {
            // For '=' syntax, return type comes directly after ')'
            if self.current_token == Token::Symbol('{') {
                // No return type specified, default to void
                crate::ast::AstType::Void
            } else {
                // Parse the return type
                self.parse_type()?
            }
        } else {
            // For ':' syntax, we may have '=' before the body
            if self.try_consume_operator("=") {
                // If we see '=' immediately, default to void
                crate::ast::AstType::Void
            } else {
                // Parse the return type
                let ret_type = self.parse_type()?;
                // After return type, expect '=' before body
                self.expect_operator("=")?;
                ret_type
            }
        };

        // Function body
        self.expect_symbol('{')?;

        let mut body = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            body.push(self.parse_statement()?);
        }

        if self.current_token != Token::Symbol('}') {
            return Err(self.syntax_error("Expected '}' to close function body"));
        }
        self.next_token();

        // Functions starting with __ are private, everything else is public
        let is_public = !name.starts_with("__");

        Ok(Function {
            name,
            type_params,
            args,
            return_type,
            body,
            is_varargs,
            is_public,
        })
    }
}
