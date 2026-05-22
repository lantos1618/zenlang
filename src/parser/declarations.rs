use super::*;
use crate::parser::keywords::ParserBehaviorKeyword;

mod function_forms;
mod generic;

impl Parser {
    pub(super) fn consume_mutability_keyword(&mut self) -> bool {
        use crate::parser::keywords::ParserMutabilityKeyword;

        if let Token::Identifier(ref name) = self.peek() {
            if name.parse::<ParserMutabilityKeyword>().is_ok() {
                self.advance();
                return true;
            }
        }
        false
    }

    // ── Declarations ──────────────────────────────────────────

    pub(super) fn parse_declaration(&mut self) -> Result<Declaration, CompileError> {
        self.skip_newlines();

        // pub prefix
        let public = if matches!(self.peek(), Token::Pub) {
            self.advance();
            self.skip_newlines();
            true
        } else {
            false
        };

        // Import: `{ names } = module.path`
        if matches!(self.peek(), Token::LBrace) && self.is_import() {
            return self.parse_import();
        }

        // Must be identifier-led
        let (name, name_span) = self.expect_identifier()?;

        self.skip_newlines();

        match self.peek() {
            // Behavior: `Name: behavior { method: (Self) Return }`
            Token::Colon
                if self
                    .colon_is_followed_by_identifier(ParserBehaviorKeyword::Behavior.as_str()) =>
            {
                self.parse_behavior_def(name, Vec::new(), public, name_span)
            }

            // Struct: `Name: { fields }`
            Token::Colon if self.is_struct_def() => self.parse_struct_def(name, public, name_span),

            // Enum: `Name: Variant1, Variant2` OR `Name:\n  Variant1,\n  Variant2`
            Token::Colon if self.is_enum_def() => self.parse_enum_def(name, public, name_span),

            // Generic type/function: `Name<T>...`
            Token::Lt => {
                let type_params = self.parse_type_params()?;
                self.parse_generic_declaration(name, type_params, public, name_span)
            }

            // Method: `Type.method = ...`
            Token::Dot => {
                self.advance(); // consume .
                let (method_name, _method_span) = self.expect_identifier()?;
                self.skip_newlines();

                if let Ok(keyword) = method_name.parse::<TypeDeclarationKeyword>() {
                    return match keyword {
                        TypeDeclarationKeyword::Impl => self.parse_impl_block(name, name_span),
                        TypeDeclarationKeyword::Implements => {
                            self.parse_behavior_impl_block(name, name_span)
                        }
                        TypeDeclarationKeyword::Requires => {
                            self.parse_behavior_requires(name, name_span)
                        }
                        TypeDeclarationKeyword::Extends => {
                            self.parse_behavior_extends(name, name_span)
                        }
                        TypeDeclarationKeyword::Derive => {
                            self.parse_behavior_derive(name, name_span)
                        }
                    };
                }

                // Type.method<T> = (params) ret { body }
                let type_params = if matches!(self.peek(), Token::Lt) {
                    self.parse_type_params()?
                } else {
                    Vec::new()
                };

                self.skip_newlines();
                self.expect(&Token::Assign)?;
                self.skip_newlines();

                let (params, return_type, body) = self.parse_function_signature_and_body()?;
                let span = name_span.merge(body.span());
                Ok(Declaration::Method {
                    type_name: name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    public,
                    span,
                })
            }

            // Function: `name = (params) ret { body }` or const `name = expr`
            Token::Assign => self.parse_function_def(name, Vec::new(), public, name_span),

            // Const assignment at top level: `name := expr`
            Token::ConstAssign => self.parse_const_decl(name, name_span),

            // Mutable decl assignment at top level: `name ::= expr`
            Token::DeclareAssign => self.parse_var_decl_toplevel(name, name_span),

            _ => Err(CompileError::Syntax(
                format!(
                    "unexpected token {:?} after identifier '{}'",
                    self.peek(),
                    name
                ),
                Some(self.peek_span()),
            )),
        }
    }
}
