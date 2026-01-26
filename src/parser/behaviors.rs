use crate::ast::{
    AstType as Type, BehaviorDefinition, BehaviorMethod, ImplBlock, Parameter, TraitConstraint,
    TraitDefinition, TraitImplementation, TraitMethod, TraitRequirement, TypeParameter,
};
use crate::error::Result;
use crate::lexer::Token;
use crate::parser::core::Parser;

impl<'a> Parser<'a> {
    pub fn parse_type_parameters(&mut self) -> Result<Vec<TypeParameter>> {
        // Parse generic type parameters if present: <T: Trait1 + Trait2, U, ...>
        let mut type_params = Vec::new();
        if self.current_token == Token::Operator("<".to_string()) {
            self.next_token();
            loop {
                if let Token::Identifier(gen) = &self.current_token {
                    let name = gen.clone();
                    self.next_token();

                    // Parse optional trait constraints: T: Trait1 + Trait2
                    let mut constraints = Vec::new();
                    if self.current_token == Token::Symbol(':') {
                        self.next_token(); // consume ':'

                        // Parse first trait constraint
                        if let Token::Identifier(trait_name) = &self.current_token {
                            constraints.push(TraitConstraint {
                                trait_name: trait_name.clone(),
                            });
                            self.next_token();

                            // Parse additional constraints with '+' operator
                            while self.try_consume_operator("+") {
                                let constraint_name =
                                    self.expect_identifier("trait name after '+'")?;
                                constraints.push(TraitConstraint {
                                    trait_name: constraint_name,
                                });
                            }
                        } else {
                            return Err(self.syntax_error("Expected trait name after ':'"));
                        }
                    }

                    type_params.push(TypeParameter { name, constraints });

                    if self.try_consume_operator(">") {
                        break;
                    } else if self.try_consume_symbol(',') {
                        // continue to next parameter
                    } else {
                        return Err(self.syntax_error("Expected ',' or '>' in generic parameters"));
                    }
                } else {
                    return Err(self.syntax_error("Expected generic parameter name"));
                }
            }
        }
        Ok(type_params)
    }

    fn parse_parameter(&mut self) -> Result<Parameter> {
        // Check for mutability
        let is_mutable = self.try_consume_keyword("mut");

        // Parse parameter name
        let name = self.expect_identifier("parameter name")?;

        // Special handling for 'self' parameter - type is optional
        let type_ = if name == "self" && self.current_token != Token::Symbol(':') {
            // For 'self' without explicit type, use a placeholder type
            // This will be resolved during type checking based on the implementing type
            Type::Generic {
                name: "Self".to_string(),
                type_args: Vec::new(),
            }
        } else {
            // Expect ':' for type annotation
            self.expect_symbol(':')?;
            // Parse parameter type
            self.parse_type()?
        };

        Ok(Parameter {
            name,
            type_,
            is_mutable,
        })
    }

    pub fn parse_behavior(&mut self) -> Result<BehaviorDefinition> {
        // Parse behavior name
        let name = self.expect_identifier("behavior name")?;

        // Parse optional type parameters
        let type_params = self.parse_type_parameters()?;

        // Expect ':' for type definition
        self.expect_symbol(':')?;

        // Expect 'behavior' identifier
        if !self.try_consume_keyword("behavior") {
            return Err(self.syntax_error("Expected 'behavior' identifier"));
        }

        // Expect '{'
        self.expect_symbol('{')?;

        // Parse behavior methods
        let mut methods = Vec::new();

        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            let method = self.parse_behavior_method()?;
            methods.push(method);

            // Handle comma separator
            self.try_consume_symbol(',');
        }

        // Expect '}'
        if self.current_token != Token::Symbol('}') {
            return Err(self.syntax_error("Expected '}' to close behavior definition"));
        }
        self.next_token();

        Ok(BehaviorDefinition {
            name,
            type_params,
            methods,
        })
    }

    fn parse_behavior_method(&mut self) -> Result<BehaviorMethod> {
        // Parse method name
        let name = self.expect_identifier("method name")?;

        // Expect ':' for method type definition
        self.expect_symbol(':')?;

        // Expect '('
        self.expect_symbol('(')?;

        // Parse parameters
        let mut params = Vec::new();

        if self.current_token != Token::Symbol(')') {
            loop {
                let param = self.parse_parameter()?;
                params.push(param);

                if !self.try_consume_symbol(',') {
                    break;
                }
            }
        }

        // Expect ')'
        self.expect_symbol(')')?;

        // Parse return type
        let return_type = self.parse_type()?;

        Ok(BehaviorMethod {
            name,
            params,
            return_type,
        })
    }

    pub fn parse_trait(&mut self) -> Result<TraitDefinition> {
        let start_span = self.current_span.clone();

        let name = self.expect_identifier("trait name")?;

        // Parse optional type parameters
        let type_params = self.parse_type_parameters()?;

        // Expect ':' for type definition
        self.expect_symbol(':')?;

        // Expect '{'
        self.expect_symbol('{')?;

        // Parse trait methods
        let mut methods = Vec::new();

        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            let method = self.parse_trait_method()?;
            methods.push(method);

            // Handle comma separator
            self.try_consume_symbol(',');
        }

        // Expect '}'
        if self.current_token != Token::Symbol('}') {
            return Err(self.syntax_error("Expected '}' to close trait definition"));
        }
        self.next_token();

        Ok(TraitDefinition {
            name,
            type_params,
            methods,
            span: Some(start_span),
        })
    }

    fn parse_trait_method(&mut self) -> Result<TraitMethod> {
        // Parse method name
        let name = self.expect_identifier("method name")?;

        // Expect ':' for method type definition
        self.expect_symbol(':')?;

        // Expect '('
        self.expect_symbol('(')?;

        // Parse parameters
        let mut params = Vec::new();

        if self.current_token != Token::Symbol(')') {
            loop {
                let param = self.parse_parameter()?;
                params.push(param);

                if !self.try_consume_symbol(',') {
                    break;
                }
            }
        }

        // Expect ')'
        self.expect_symbol(')')?;

        // Parse return type
        let return_type = self.parse_type()?;

        Ok(TraitMethod {
            name,
            params,
            return_type,
        })
    }

    /// Parse a function within an impl block context with a pre-parsed name
    pub fn parse_impl_function_with_name(&mut self, name: String) -> Result<crate::ast::Function> {
        use crate::ast::Function;

        // Parse generic type parameters if present: <T: Constraint, U, ...>
        let type_params = self.parse_type_parameters()?;

        // Parameters
        self.expect_symbol('(')?;

        let mut args = vec![];
        if self.current_token != Token::Symbol(')') {
            loop {
                // Parameter name
                let param_name = self.expect_identifier("parameter name")?;

                // Parameter type - special handling for 'self'
                let param_type = if param_name == "self" && self.current_token != Token::Symbol(':')
                {
                    // For 'self' without explicit type, use a placeholder type
                    Type::Generic {
                        name: "Self".to_string(),
                        type_args: Vec::new(),
                    }
                } else {
                    self.expect_symbol(':')?;
                    self.parse_type()?
                };
                args.push((param_name, param_type));

                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if !self.try_consume_symbol(',') {
                    return Err(self.syntax_error("Expected ',' or ')' in parameter list"));
                }
            }
        }
        self.next_token(); // consume ')'

        // Check for return type (it should be present for impl functions)
        let return_type = if self.current_token != Token::Symbol('{') {
            // If it's not '{', then we have a return type
            self.parse_type()?
        } else {
            // Default to void/unit type if no return type specified
            crate::ast::AstType::Void
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

        // Check visibility before moving name
        let is_public = !name.starts_with("__");

        Ok(Function {
            name,
            type_params,
            args,
            return_type,
            body,
            is_varargs: false,
            is_public,
        })
    }

    /// Parse a function within an impl block context
    /// This is different from parse_function() because the function name and '=' have already been consumed
    #[allow(dead_code)]
    pub fn parse_impl_function(&mut self) -> Result<crate::ast::Function> {
        use crate::ast::Function;

        // Function name
        let name = self.expect_identifier("function name")?;

        // Parse generic type parameters if present: <T: Constraint, U, ...>
        let type_params = self.parse_type_parameters()?;

        // Expect '=' (function signature separator)
        self.expect_operator("=")?;

        // Parameters
        self.expect_symbol('(')?;

        let mut args = vec![];
        if self.current_token != Token::Symbol(')') {
            loop {
                // Parameter name
                let param_name = self.expect_identifier("parameter name")?;

                // Parameter type - special handling for 'self'
                let param_type = if param_name == "self" && self.current_token != Token::Symbol(':')
                {
                    // For 'self' without explicit type, use a placeholder type
                    Type::Generic {
                        name: "Self".to_string(),
                        type_args: Vec::new(),
                    }
                } else {
                    self.expect_symbol(':')?;
                    self.parse_type()?
                };
                args.push((param_name, param_type));

                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if !self.try_consume_symbol(',') {
                    return Err(self.syntax_error("Expected ',' or ')' in parameter list"));
                }
            }
        }
        self.next_token(); // consume ')'

        // Check for return type (it should be present for impl functions)
        let return_type = if self.current_token != Token::Symbol('{') {
            // If it's not '{', then we have a return type
            self.parse_type()?
        } else {
            // Default to void/unit type if no return type specified
            crate::ast::AstType::Void
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

        // Check visibility before moving name
        let is_public = !name.starts_with("__");

        Ok(Function {
            name,
            type_params,
            args,
            return_type,
            body,
            is_varargs: false,
            is_public,
        })
    }

    pub fn parse_trait_implementation(&mut self, type_name: String) -> Result<TraitImplementation> {
        // Parse: Type.implements(Trait, { methods })
        // Current token is '(' after 'implements'
        self.expect_symbol('(')?;

        // Parse trait name
        let trait_name = self.expect_identifier("trait name")?;

        // Expect comma
        self.expect_symbol(',')?;

        // Expect opening brace for methods
        self.expect_symbol('{')?;

        // Parse methods
        let mut methods = Vec::new();
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            // Parse method: name = (params) return_type { body }
            let method_name = self.expect_identifier("method name")?;

            // Expect '='
            self.expect_operator("=")?;

            // Parse method as a function (name has already been parsed)
            let func = self.parse_impl_function_with_name(method_name)?;
            methods.push(func);

            // Check for comma between methods
            self.try_consume_symbol(',');
        }

        // Expect closing brace
        if self.current_token != Token::Symbol('}') {
            return Err(self.syntax_error("Expected '}' to close trait methods"));
        }
        self.next_token(); // consume '}'

        // Expect closing parenthesis
        self.expect_symbol(')')?;

        Ok(TraitImplementation {
            type_name,
            trait_name,
            type_params: Vec::new(), // Generic type params not yet parsed
            methods,
        })
    }

    pub fn parse_impl_block(&mut self, type_name: String) -> Result<ImplBlock> {
        // Parse: Type.impl = { methods }
        // Current token is '{' after '='
        self.expect_symbol('{')?;

        // Parse methods (type parameters are parsed from the type_name itself, e.g., Option<T>)
        let type_params = Vec::new(); // Type params come from the type_name, not here

        // Parse methods
        let mut methods = Vec::new();
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            // Parse method: name = (params) return_type { body }
            let method_name = self.expect_identifier("method name")?;

            // Expect '='
            self.expect_operator("=")?;

            // Parse method as a function (name has already been parsed)
            let func = self.parse_impl_function_with_name(method_name)?;
            methods.push(func);

            // Check for comma between methods (optional)
            self.try_consume_symbol(',');
        }

        // Expect closing brace
        if self.current_token != Token::Symbol('}') {
            return Err(self.syntax_error("Expected '}' to close impl methods"));
        }
        self.next_token(); // consume '}'

        Ok(ImplBlock {
            type_name,
            type_params,
            methods,
        })
    }

    pub fn parse_trait_requirement(&mut self, type_name: String) -> Result<TraitRequirement> {
        // Parse: Type.requires(Trait)
        // Current token is '(' after 'requires'
        self.expect_symbol('(')?;

        // Parse trait name
        let trait_name = self.expect_identifier("trait name")?;

        // Expect closing parenthesis
        self.expect_symbol(')')?;

        Ok(TraitRequirement {
            type_name,
            trait_name,
        })
    }
}
