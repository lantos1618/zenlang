use super::core::Parser;
use crate::ast::{primitive_from_str, AstType};
use crate::error::Result;
use crate::lexer::Token;
use crate::well_known::well_known;

impl<'a> Parser<'a> {
    pub fn parse_type(&mut self) -> Result<AstType> {
        match &self.current_token {
            Token::Identifier(type_name) => {
                let mut type_name = type_name.clone();
                self.next_token();

                // Check for member access in type names (e.g., FFI.Library)
                while self.try_consume_symbol('.') {
                    let member = self.expect_identifier("identifier after '.' in type name")?;
                    type_name = format!("{}.{}", type_name, member);
                }

                // Check for primitive types using centralized lookup
                if let Some(prim_type) = primitive_from_str(&type_name) {
                    return Ok(prim_type);
                }

                // Handle special cases not in PRIMITIVE_TYPE_MAP
                match type_name.as_str() {
                    "StringLiteral" => Ok(AstType::StaticString), // Alias for StaticString
                    // String is a struct type - return as Generic to be resolved later
                    // IMPORTANT: Do NOT call resolve_string_struct_type() here as it causes
                    // circular dependency with stdlib_types() OnceLock initialization
                    "String" => Ok(AstType::Generic {
                        name: "String".to_string(),
                        type_args: vec![],
                    }),
                    "ptr" => Ok(AstType::ptr(AstType::Void)),
                    // Well-known pointer types (Ptr, MutPtr, RawPtr)
                    _ if well_known().is_ptr(&type_name) => {
                        let wk = well_known();
                        if self.try_consume_operator("<") {
                            let inner_type = self.parse_type()?;
                            self.expect_operator(">")?;
                            if wk.is_immutable_ptr(&type_name) {
                                Ok(AstType::ptr(inner_type))
                            } else if wk.is_mutable_ptr(&type_name) {
                                Ok(AstType::mut_ptr(inner_type))
                            } else {
                                Ok(AstType::raw_ptr(inner_type))
                            }
                        } else {
                            Err(self.syntax_error(format!(
                                "{} type requires type argument: {}<T>",
                                type_name, type_name
                            )))
                        }
                    }
                    // Vec<T> and DynVec<T> are stdlib generic types, not special AST types
                    // They fall through to the generic type handling below
                    "str" | "string" => {
                        // Provide helpful error for common mistake
                        Err(self.syntax_error(
                            "Use 'StringLiteral' for string literals or 'String' for dynamic strings ('str' and 'string' are not valid types)"
                        ))
                    }
                    _ => {
                        // Check for generic type instantiation (e.g., List<T>)
                        if self.try_consume_operator("<") {
                            let mut type_args = Vec::new();

                            loop {
                                type_args.push(self.parse_type()?);

                                if self.try_consume_operator(">") {
                                    break;
                                } else if !self.try_consume_symbol(',') {
                                    return Err(
                                        self.syntax_error("Expected ',' or '>' in type arguments")
                                    );
                                }
                            }

                            Ok(AstType::Generic {
                                name: type_name,
                                type_args,
                            })
                        } else {
                            // Could be a custom type (struct, enum, etc.) or type parameter
                            Ok(AstType::Generic {
                                name: type_name,
                                type_args: vec![],
                            })
                        }
                    }
                }
            }
            Token::Symbol('[') => {
                // Array type: [T] (dynamic array) or [T; N] (fixed-size array)
                self.next_token();
                let element_type = self.parse_type()?;

                if self.try_consume_symbol(';') {
                    // Parse the size (must be an integer literal for now)
                    match &self.current_token {
                        Token::Integer(size_str) => {
                            let size = size_str.parse::<usize>().map_err(|_| {
                                self.syntax_error(format!("Invalid array size: {}", size_str))
                            })?;
                            self.next_token();

                            self.expect_symbol(']')?;
                            Ok(AstType::FixedArray {
                                element_type: Box::new(element_type),
                                size,
                            })
                        }
                        _ => Err(self.syntax_error("Expected integer literal for array size")),
                    }
                } else if self.try_consume_symbol(']') {
                    // Slice type [T] - pointer + length
                    Ok(AstType::Slice(Box::new(element_type)))
                } else {
                    Err(self.syntax_error("Expected ']' or ';' in array type"))
                }
            }
            Token::Symbol('*') => {
                // Pointer type: *T or function pointer *(params) return_type
                self.next_token();
                self.parse_pointer_or_function_pointer()
            }
            Token::Operator(op) if op == "*" => {
                // Pointer type: *T (operator version)
                self.next_token();
                self.parse_pointer_or_function_pointer()
            }
            Token::Symbol('(') => {
                // Check for unit type () vs function type (params) ReturnType
                self.next_token();

                // If immediately followed by ')', check if there's a return type
                if self.current_token == Token::Symbol(')') {
                    self.next_token(); // consume ')'

                    // Check if there's a return type following: () ReturnType
                    // This is a function type with no parameters
                    if matches!(
                        self.current_token,
                        Token::Identifier(_) | Token::Symbol('(') | Token::Symbol('[')
                    ) {
                        let return_type = self.parse_type()?;
                        return Ok(AstType::FunctionPointer {
                            param_types: vec![],
                            return_type: Box::new(return_type),
                        });
                    }

                    // Otherwise it's just the unit type ()
                    return Ok(AstType::Void);
                }

                // Otherwise it's a function type: (params) ReturnType (for behaviors)
                let mut param_types = Vec::new();

                // Parse parameter types
                while self.current_token != Token::Symbol(')') {
                    // Check if it's a parameter with name (name: Type) or just a type
                    if let Token::Identifier(param_name) = &self.current_token {
                        let name = param_name.clone();

                        // Special case for "self" parameter - doesn't need a type
                        if name == "self" {
                            self.next_token();
                            // "self" is implicitly the type being defined
                            param_types.push(AstType::Generic {
                                name: "Self".to_string(),
                                type_args: vec![],
                            });
                        } else {
                            // Look ahead to see if this is "name: Type" or just "Type"
                            let saved = self.save_state();

                            self.next_token();

                            // Check if next token is colon (named parameter)
                            if self.try_consume_symbol(':') {
                                param_types.push(self.parse_type()?);
                            } else {
                                // Not a named parameter, restore and parse as type
                                self.restore_state(saved);
                                param_types.push(self.parse_type()?);
                            }
                        }
                    } else {
                        // Just parse the type directly
                        param_types.push(self.parse_type()?);
                    }

                    if !self.try_consume_symbol(',') && self.current_token != Token::Symbol(')') {
                        return Err(
                            self.syntax_error("Expected ',' or ')' in function type parameters")
                        );
                    }
                }

                self.next_token(); // Skip ')'

                // Parse return type
                let return_type = self.parse_type()?;

                Ok(AstType::FunctionPointer {
                    param_types,
                    return_type: Box::new(return_type),
                })
            }
            Token::Symbol('&') => {
                // Reference type: &T
                self.next_token();
                let referenced_type = self.parse_type()?;
                Ok(AstType::Ref(Box::new(referenced_type)))
            }
            Token::Symbol('{') => {
                // Anonymous struct type: {field1: Type1, field2: Type2}
                self.next_token(); // consume '{'

                let mut fields = Vec::new();

                while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
                    // Parse field name
                    let field_name =
                        self.expect_identifier("field name in anonymous struct type")?;

                    // Expect ':'
                    self.expect_symbol(':')?;

                    // Parse field type
                    let field_type = self.parse_type()?;
                    fields.push((field_name, field_type));

                    // Check for comma or end of struct
                    if !self.try_consume_symbol(',') && self.current_token != Token::Symbol('}') {
                        return Err(
                            self.syntax_error("Expected ',' or '}' in anonymous struct type")
                        );
                    }
                }

                if self.current_token != Token::Symbol('}') {
                    return Err(self.syntax_error("Expected '}' to close anonymous struct type"));
                }
                self.next_token(); // consume '}'

                Ok(AstType::Struct {
                    name: String::new(), // Anonymous struct has no name
                    fields,
                })
            }
            _ => Err(self.syntax_error(format!(
                "Unexpected token in type: {:?}",
                self.current_token
            ))),
        }
    }

    /// Helper for parsing pointer or function pointer types
    fn parse_pointer_or_function_pointer(&mut self) -> Result<AstType> {
        // Check if it's a function pointer
        if self.current_token == Token::Symbol('(') {
            self.next_token();
            let mut param_types = Vec::new();

            // Parse parameter types
            while self.current_token != Token::Symbol(')') {
                param_types.push(self.parse_type()?);

                if !self.try_consume_symbol(',') && self.current_token != Token::Symbol(')') {
                    return Err(
                        self.syntax_error("Expected ',' or ')' in function pointer parameters")
                    );
                }
            }

            self.next_token(); // Skip ')'

            // Parse return type
            let return_type = self.parse_type()?;

            Ok(AstType::FunctionPointer {
                param_types,
                return_type: Box::new(return_type),
            })
        } else {
            // Regular pointer
            let pointee_type = self.parse_type()?;
            Ok(AstType::ptr(pointee_type))
        }
    }

    pub fn parse_type_alias(&mut self) -> Result<crate::ast::TypeAlias> {
        use crate::ast::{TypeAlias, TypeParameter};

        // Capture span at the start of the type alias definition
        let start_span = self.current_span.clone();

        // Skip 'type' keyword
        self.next_token();

        // Get alias name
        let name = self.expect_identifier("type alias name")?;

        // Parse optional generic type parameters
        let mut type_params = Vec::new();
        if self.try_consume_operator("<") {
            loop {
                let param_name = self.expect_identifier("type parameter name")?;
                type_params.push(TypeParameter {
                    name: param_name,
                    constraints: Vec::new(),
                });

                if self.try_consume_operator(">") {
                    break;
                } else if !self.try_consume_symbol(',') {
                    return Err(self.syntax_error("Expected ',' or '>' in type parameters"));
                }
            }
        }

        // Expect ':' for type definition
        self.expect_symbol(':')?;

        // Parse the target type
        let target_type = self.parse_type()?;

        Ok(TypeAlias {
            name,
            type_params,
            target_type,
            span: Some(start_span),
        })
    }
}
