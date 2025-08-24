use crate::ast::{BehaviorDefinition, BehaviorMethod, ImplBlock, Parameter, TypeParameter};
use crate::lexer::{Keyword, Token};
use crate::parser::core::Parser;
use crate::error::{CompileError, Result};

impl<'a> Parser<'a> {
    pub fn parse_behavior(&mut self) -> Result<BehaviorDefinition> {
        // Parse behavior name
        let name = if let Token::Identifier(n) = &self.current_token {
            n.clone()
        } else {
            return Err(CompileError::SyntaxError(
                format!("Expected behavior name, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Parse optional type parameters
        let type_params = if self.current_token == Token::Operator("<".to_string()) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };
        
        // Expect '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                format!("Expected '=' after behavior name, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Expect 'behavior' keyword
        if self.current_token != Token::Keyword(Keyword::Behavior) {
            return Err(CompileError::SyntaxError(
                format!("Expected 'behavior' keyword, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Expect '{'
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                format!("Expected '{{' after 'behavior', got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parse behavior methods
        let mut methods = Vec::new();
        
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            let method = self.parse_behavior_method()?;
            methods.push(method);
            
            // Handle comma separator
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            }
        }
        
        // Expect '}'
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError(
                "Expected '}' to close behavior definition".to_string(),
                Some(self.current_span.clone()),
            ));
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
        let name = if let Token::Identifier(n) = &self.current_token {
            n.clone()
        } else {
            return Err(CompileError::SyntaxError(
                format!("Expected method name, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Expect '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                format!("Expected '=' after method name, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Expect '('
        if self.current_token != Token::Symbol('(') {
            return Err(CompileError::SyntaxError(
                format!("Expected '(' after '=', got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parse parameters
        let mut params = Vec::new();
        
        if self.current_token != Token::Symbol(')') {
            loop {
                let param = self.parse_parameter()?;
                params.push(param);
                
                if self.current_token == Token::Symbol(',') {
                    self.next_token();
                } else {
                    break;
                }
            }
        }
        
        // Expect ')'
        if self.current_token != Token::Symbol(')') {
            return Err(CompileError::SyntaxError(
                "Expected ')' after parameters".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parse return type
        let return_type = self.parse_type()?;
        
        Ok(BehaviorMethod {
            name,
            params,
            return_type,
        })
    }
    
    pub fn parse_impl_block_from_type(&mut self, type_name: String) -> Result<ImplBlock> {
        // 'impl' keyword already consumed, expect '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                format!("Expected '=' after 'impl', got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parse optional type parameters  
        let type_params = Vec::new(); // TODO: support generics on impl blocks
        
        // Expect '{'
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                format!("Expected '{{' after '=', got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut behavior_name = None;
        let mut methods = Vec::new();
        
        // Check if this is a behavior implementation
        if let Token::Identifier(name) = &self.current_token {
            let saved_name = name.clone();
            let saved_position = self.lexer.position;
            let saved_read_position = self.lexer.read_position;
            let saved_current_char = self.lexer.current_char;
            let saved_current_token = self.current_token.clone();
            let saved_peek_token = self.peek_token.clone();
            
            self.next_token();
            
            if self.current_token == Token::Symbol(':') {
                // This is a behavior implementation
                behavior_name = Some(saved_name);
                self.next_token();
                
                // Expect '{'
                if self.current_token != Token::Symbol('{') {
                    return Err(CompileError::SyntaxError(
                        format!("Expected '{{' after ':', got {:?}", self.current_token),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
                
                // Parse methods for this behavior
                while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
                    let method = self.parse_impl_function()?;
                    methods.push(method);
                    
                    // Handle comma separator
                    if self.current_token == Token::Symbol(',') {
                        self.next_token();
                    }
                }
                
                // Expect '}'
                if self.current_token != Token::Symbol('}') {
                    return Err(CompileError::SyntaxError(
                        "Expected '}' to close behavior implementation".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
            } else {
                // This was actually a method, rewind
                self.lexer.position = saved_position;
                self.lexer.read_position = saved_read_position;
                self.lexer.current_char = saved_current_char;
                self.current_token = saved_current_token;
                self.peek_token = saved_peek_token;
                
                // Parse inherent methods
                while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
                    let method = self.parse_impl_function()?;
                    methods.push(method);
                    
                    // Handle comma separator
                    if self.current_token == Token::Symbol(',') {
                        self.next_token();
                    }
                }
            }
        } else {
            // Parse inherent methods
            while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
                let method = self.parse_impl_function()?;
                methods.push(method);
                
                // Handle comma separator
                if self.current_token == Token::Symbol(',') {
                    self.next_token();
                }
            }
        }
        
        // Expect '}'
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError(
                "Expected '}' to close impl block".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        Ok(ImplBlock {
            type_name,
            behavior_name,
            type_params,
            methods,
        })
    }
    
    fn parse_parameter(&mut self) -> Result<Parameter> {
        let name = if let Token::Identifier(n) = &self.current_token {
            n.clone()
        } else {
            return Err(CompileError::SyntaxError(
                format!("Expected parameter name, got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Special case for 'self' parameter - type annotation is optional
        let type_ = if name == "self" && self.current_token != Token::Symbol(':') {
            // Infer as pointer to the implementing type (will be resolved later)
            crate::ast::AstType::Pointer(Box::new(crate::ast::AstType::Generic {
                name: "Self".to_string(),
                type_args: Vec::new(),
            }))
        } else {
            if self.current_token != Token::Symbol(':') {
                return Err(CompileError::SyntaxError(
                    format!("Expected ':' after parameter name, got {:?}", self.current_token),
                    Some(self.current_span.clone()),
                ));
            }
            self.next_token();
            
            self.parse_type()?
        };
        
        Ok(Parameter {
            name,
            type_,
            is_mutable: false, // For now, behaviors don't specify mutability
        })
    }
    
    pub fn parse_type_parameters(&mut self) -> Result<Vec<TypeParameter>> {
        // Expect '<'
        if self.current_token != Token::Operator("<".to_string()) {
            return Err(CompileError::SyntaxError(
                format!("Expected '<', got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut type_params = Vec::new();
        
        while self.current_token != Token::Operator(">".to_string()) && self.current_token != Token::Eof {
            let name = if let Token::Identifier(n) = &self.current_token {
                n.clone()
            } else {
                return Err(CompileError::SyntaxError(
                    format!("Expected type parameter name, got {:?}", self.current_token),
                    Some(self.current_span.clone()),
                ));
            };
            self.next_token();
            
            // TODO: Parse constraints (: BehaviorName)
            let constraints = Vec::new();
            
            type_params.push(TypeParameter { name, constraints });
            
            if self.current_token == Token::Symbol(',') {
                self.next_token();
            }
        }
        
        // Expect '>'
        if self.current_token != Token::Operator(">".to_string()) {
            return Err(CompileError::SyntaxError(
                format!("Expected '>', got {:?}", self.current_token),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        Ok(type_params)
    }
    
    /// Parse a function within an impl block context
    /// This is different from parse_function() because the function name and '=' have already been consumed
    pub fn parse_impl_function(&mut self) -> Result<crate::ast::Function> {
        use crate::ast::{Function, TypeParameter};
        
        // Function name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected function name".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Parse generic type parameters if present: <T, U, ...>
        let mut type_params = Vec::new();
        if self.current_token == Token::Operator("<".to_string()) {
            self.next_token();
            loop {
                if let Token::Identifier(gen) = &self.current_token {
                    type_params.push(TypeParameter {
                        name: gen.clone(),
                        constraints: Vec::new(),
                    });
                    self.next_token();
                    
                    if self.current_token == Token::Operator(">".to_string()) {
                        self.next_token();
                        break;
                    } else if self.current_token == Token::Symbol(',') {
                        self.next_token();
                    } else {
                        return Err(CompileError::SyntaxError(
                            "Expected ',' or '>' in generic parameters".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected generic parameter name".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
            }
        }
        
        // Expect '=' (function signature separator)
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '=' after function name".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parameters
        if self.current_token != Token::Symbol('(') {
            return Err(CompileError::SyntaxError(
                "Expected '(' for function parameters".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut args = vec![];
        if self.current_token != Token::Symbol(')') {
            loop {
                // Parameter name
                let param_name = if let Token::Identifier(name) = &self.current_token {
                    name.clone()
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected parameter name".to_string(),
                        Some(self.current_span.clone()),
                    ));
                };
                self.next_token();
                
                // Parameter type
                if self.current_token != Token::Symbol(':') {
                    return Err(CompileError::SyntaxError(
                        "Expected ':' after parameter name".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
                
                let param_type = self.parse_type()?;
                args.push((param_name, param_type));
                
                if self.current_token == Token::Symbol(')') {
                    break;
                }
                if self.current_token != Token::Symbol(',') {
                    return Err(CompileError::SyntaxError(
                        "Expected ',' or ')' in parameter list".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
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
        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' for function body".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        let mut body = vec![];
        while self.current_token != Token::Symbol('}') && self.current_token != Token::Eof {
            body.push(self.parse_statement()?);
        }
        
        if self.current_token != Token::Symbol('}') {
            return Err(CompileError::SyntaxError(
                "Expected '}' to close function body".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        Ok(Function {
            name,
            type_params,
            args,
            return_type,
            body,
            is_async: false, // TODO: Support async functions
        })
    }
}