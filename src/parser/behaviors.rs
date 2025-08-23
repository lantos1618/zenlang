use crate::ast::{AstType, BehaviorDefinition, BehaviorMethod, ImplBlock, Parameter, TypeParameter, Function};
use crate::lexer::{Keyword, Token};
use crate::parser::core::Parser;
use crate::parser::types::parse_type;
use crate::parser::functions::parse_function;

impl<'a> Parser<'a> {
    pub fn parse_behavior(&mut self) -> Result<BehaviorDefinition, String> {
        // Expect 'behavior' keyword
        self.expect_keyword(Keyword::Behavior)?;
        
        // Parse behavior name
        let name = self.expect_identifier()?;
        
        // Parse optional type parameters
        let type_params = if self.current_token_is(&Token::Symbol('<')) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };
        
        // Expect '='
        self.expect_symbol('=')?;
        
        // Expect 'behavior' keyword again
        self.expect_keyword(Keyword::Behavior)?;
        
        // Expect '{'
        self.expect_symbol('{')?;
        
        // Parse behavior methods
        let mut methods = Vec::new();
        
        while !self.current_token_is(&Token::Symbol('}')) {
            let method = self.parse_behavior_method()?;
            methods.push(method);
            
            // Handle comma separator
            if self.current_token_is(&Token::Symbol(',')) {
                self.advance();
            }
        }
        
        // Expect '}'
        self.expect_symbol('}')?;
        
        Ok(BehaviorDefinition {
            name,
            type_params,
            methods,
        })
    }
    
    fn parse_behavior_method(&mut self) -> Result<BehaviorMethod, String> {
        // Parse method name
        let name = self.expect_identifier()?;
        
        // Expect '='
        self.expect_symbol('=')?;
        
        // Expect '('
        self.expect_symbol('(')?;
        
        // Parse parameters
        let mut params = Vec::new();
        
        if !self.current_token_is(&Token::Symbol(')')) {
            loop {
                let param = self.parse_parameter()?;
                params.push(param);
                
                if self.current_token_is(&Token::Symbol(',')) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        // Expect ')'
        self.expect_symbol(')')?;
        
        // Parse return type
        let return_type = parse_type(self)?;
        
        Ok(BehaviorMethod {
            name,
            params,
            return_type,
        })
    }
    
    pub fn parse_impl_block(&mut self) -> Result<ImplBlock, String> {
        // Parse type name
        let type_name = self.expect_identifier()?;
        
        // Parse optional type parameters  
        let type_params = if self.current_token_is(&Token::Symbol('<')) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };
        
        // Expect '.'
        self.expect_symbol('.')?;
        
        // Expect 'impl'
        self.expect_keyword(Keyword::Impl)?;
        
        // Expect '='
        self.expect_symbol('=')?;
        
        // Expect '{'
        self.expect_symbol('{')?;
        
        let mut behavior_name = None;
        let mut methods = Vec::new();
        
        // Check if this is a behavior implementation
        if let Token::Identifier(name) = &self.current_token {
            let saved_name = name.clone();
            self.advance();
            
            if self.current_token_is(&Token::Symbol(':')) {
                // This is a behavior implementation
                behavior_name = Some(saved_name);
                self.advance();
                
                // Expect '{'
                self.expect_symbol('{')?;
                
                // Parse methods for this behavior
                while !self.current_token_is(&Token::Symbol('}')) {
                    let method = parse_function(self)?;
                    methods.push(method);
                    
                    // Handle comma separator
                    if self.current_token_is(&Token::Symbol(',')) {
                        self.advance();
                    }
                }
                
                // Expect '}'
                self.expect_symbol('}')?;
            } else {
                // This was actually a method, rewind
                self.rewind_token();
                
                // Parse inherent methods
                while !self.current_token_is(&Token::Symbol('}')) {
                    let method = parse_function(self)?;
                    methods.push(method);
                    
                    // Handle comma separator
                    if self.current_token_is(&Token::Symbol(',')) {
                        self.advance();
                    }
                }
            }
        } else {
            // Parse inherent methods
            while !self.current_token_is(&Token::Symbol('}')) {
                let method = parse_function(self)?;
                methods.push(method);
                
                // Handle comma separator
                if self.current_token_is(&Token::Symbol(',')) {
                    self.advance();
                }
            }
        }
        
        // Expect '}'
        self.expect_symbol('}')?;
        
        Ok(ImplBlock {
            type_name,
            behavior_name,
            type_params,
            methods,
        })
    }
    
    fn parse_parameter(&mut self) -> Result<Parameter, String> {
        let name = self.expect_identifier()?;
        self.expect_symbol(':')?;
        let type_ = parse_type(self)?;
        
        Ok(Parameter {
            name,
            type_,
            is_mutable: false, // For now, behaviors don't specify mutability
        })
    }
}