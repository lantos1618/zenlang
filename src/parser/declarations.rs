use super::*;
use crate::parser::keywords::ParserBehaviorKeyword;

mod function_forms;

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
                self.skip_newlines();
                match self.peek() {
                    Token::Assign => {
                        // Generic function: `name<T> = (params) ret { body }`
                        self.parse_function_def(name, type_params, public, name_span)
                    }
                    Token::Dot => {
                        // Generic receiver method: `Type<T>.method = (self: Type<T>) ...`
                        self.advance(); // consume .
                        let (method_name, _method_span) = self.expect_identifier()?;
                        self.skip_newlines();

                        if let Ok(keyword) = method_name.parse::<TypeDeclarationKeyword>() {
                            if matches!(keyword, TypeDeclarationKeyword::Impl) {
                                return self.parse_impl_block_with_type_params(
                                    name,
                                    type_params,
                                    name_span,
                                );
                            }
                            if matches!(keyword, TypeDeclarationKeyword::Implements) {
                                return self.parse_behavior_impl_block_with_type_params(
                                    name,
                                    type_params,
                                    name_span,
                                );
                            }
                            return self
                                .reject_gated_generic_association_target(keyword, name_span);
                        }

                        let mut all_type_params = type_params;
                        if matches!(self.peek(), Token::Lt) {
                            all_type_params.extend(self.parse_type_params()?);
                        }

                        self.skip_newlines();
                        self.expect(&Token::Assign)?;
                        self.skip_newlines();

                        let (params, return_type, body) =
                            self.parse_function_signature_and_body()?;
                        let span = name_span.merge(body.span());
                        Ok(Declaration::Method {
                            type_name: name,
                            method_name,
                            type_params: all_type_params,
                            params,
                            return_type,
                            body,
                            public,
                            span,
                        })
                    }
                    Token::Colon if self.is_struct_def() => {
                        self.parse_struct_def_with_params(name, type_params, public, name_span)
                    }
                    Token::Colon
                        if self.colon_is_followed_by_identifier(
                            ParserBehaviorKeyword::Behavior.as_str(),
                        ) =>
                    {
                        self.parse_behavior_def(name, type_params, public, name_span)
                    }
                    Token::Colon => {
                        self.parse_enum_def_with_params(name, type_params, public, name_span)
                    }
                    _ => Err(CompileError::Syntax(
                        "expected '=' or ':' after generic type parameters".to_string(),
                        Some(self.peek_span()),
                    )),
                }
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

    // ── Function ──────────────────────────────────────────────

    fn reject_gated_generic_association_target<T>(
        &mut self,
        keyword: TypeDeclarationKeyword,
        name_span: Span,
    ) -> Result<T, CompileError> {
        let span = self.gated_association_call_span(name_span);
        Err(CompileError::Syntax(
            format!(
                "generic association target `Type<T>.{keyword}` is gated; use non-generic `{keyword}` associations or keep the generic behavior target deferred to docs/V1_SPEC.md"
            ),
            Some(span),
        ))
    }

    fn gated_association_call_span(&mut self, name_span: Span) -> Span {
        if !matches!(self.peek(), Token::LParen) {
            return name_span.merge(self.prev_span());
        }

        self.advance();
        let behavior_span = self.expect_identifier().map(|(_, span)| span).ok();
        if matches!(self.peek(), Token::Lt) {
            let _ = self.parse_type_arg_list();
        }
        self.skip_newlines();
        match self.expect(&Token::RParen) {
            Ok(end) => name_span.merge(end),
            Err(_) => behavior_span
                .map(|span| name_span.merge(span))
                .unwrap_or_else(|| name_span.merge(self.prev_span())),
        }
    }
}
