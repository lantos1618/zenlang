use super::core::Parser;
use crate::ast::{StructDefinition, StructField};
use crate::error::Result;
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_struct(&mut self) -> Result<StructDefinition> {
        // Capture span at the start of the struct definition
        let start_span = self.current_span.clone();

        // Struct name
        let name = self.expect_identifier("struct name")?;

        // Parse generics if present: <T: Trait1 + Trait2, U, ...>
        let type_params = self.parse_type_parameters()?;

        // Expect and consume ':' for type definition
        self.expect_symbol(':')?;

        // Check if they're trying to use enum syntax (comma-separated) for a struct
        if matches!(&self.current_token, Token::Identifier(_))
            || self.current_token == Token::Symbol('.')
        {
            return Err(self.syntax_error(
                "Structs use curly braces for fields, not comma-separated variants. Use `MyStruct: { field1: Type1, field2: Type2 }` instead of `MyStruct: Field1, Field2`"
            ));
        }

        // Opening brace
        if self.current_token != Token::Symbol('{') {
            return Err(self.syntax_error(
                "Expected '{' for struct fields. Structs use curly braces: `MyStruct: { field: Type }`"
            ));
        }
        self.next_token();

        let mut fields = vec![];

        // Parse fields
        while self.current_token != Token::Symbol('}') {
            if self.current_token == Token::Eof {
                return Err(self.syntax_error("Unexpected end of file in struct definition"));
            }

            // Field name
            let field_name = self.expect_identifier("field name")?;

            // Check for mutability modifier (:: for mutable) or regular type annotation (:)
            let is_mutable = if self.try_consume_operator("::") {
                true
            } else if self.try_consume_symbol(':') {
                false
            } else {
                return Err(self.syntax_error("Expected ':' or '::' after field name"));
            };

            // Field type
            let field_type = self.parse_type()?;

            // Optional default value
            let default_value = if self.try_consume_operator("=") {
                Some(self.parse_expression()?)
            } else {
                None
            };

            fields.push(StructField {
                name: field_name,
                type_: field_type,
                is_mutable,
                default_value,
            });

            // Comma separator (except for last field)
            if !self.try_consume_symbol(',') && self.current_token != Token::Symbol('}') {
                return Err(self.syntax_error("Expected ',' or '}' after field"));
            }
        }

        // Closing brace
        self.next_token();

        // Methods are defined via `impl` blocks, not inline in struct definitions.
        // This matches Rust's syntax: `impl MyStruct { fn method(&self) { ... } }`
        // The methods field is kept for AST compatibility but remains empty during parsing.
        let methods = Vec::new();

        Ok(StructDefinition {
            name,
            type_params,
            fields,
            methods,
            span: Some(start_span),
        })
    }
}
