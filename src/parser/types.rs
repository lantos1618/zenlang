use super::core::Parser;
use crate::ast::AstType;
use crate::error::{CompileError, Result};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    pub fn parse_type(&mut self) -> Result<AstType> {
        match &self.current_token {
            Token::Identifier(type_name) => {
                let type_name = type_name.clone();
                self.next_token();
                
                match type_name.as_str() {
                    "i8" => Ok(AstType::I8),
                    "i16" => Ok(AstType::I16),
                    "i32" => Ok(AstType::I32),
                    "i64" => Ok(AstType::I64),
                    "u8" => Ok(AstType::U8),
                    "u16" => Ok(AstType::U16),
                    "u32" => Ok(AstType::U32),
                    "u64" => Ok(AstType::U64),
                    "f32" => Ok(AstType::F32),
                    "f64" => Ok(AstType::F64),
                    "bool" => Ok(AstType::Bool),
                    "String" | "string" => Ok(AstType::String),
                    "void" => Ok(AstType::Void),
                    "ptr" => Ok(AstType::Pointer(Box::new(AstType::Void))),
                    _ => {
                        // Check for generic type instantiation (e.g., List<T>)
                        if self.current_token == Token::Operator("<".to_string()) {
                            self.next_token();
                            let mut type_args = Vec::new();
                            
                            loop {
                                type_args.push(self.parse_type()?);
                                
                                if self.current_token == Token::Operator(">".to_string()) {
                                    self.next_token();
                                    break;
                                } else if self.current_token == Token::Symbol(',') {
                                    self.next_token();
                                } else {
                                    return Err(CompileError::SyntaxError(
                                        "Expected ',' or '>' in type arguments".to_string(),
                                        Some(self.current_span.clone()),
                                    ));
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
                
                // Check for semicolon to determine if it's a fixed-size array
                if self.current_token == Token::Symbol(';') {
                    self.next_token();
                    
                    // Parse the size (must be an integer literal for now)
                    match &self.current_token {
                        Token::Integer(size_str) => {
                            let size = size_str.parse::<usize>().map_err(|_| {
                                CompileError::SyntaxError(
                                    format!("Invalid array size: {}", size_str),
                                    Some(self.current_span.clone()),
                                )
                            })?;
                            self.next_token();
                            
                            if self.current_token != Token::Symbol(']') {
                                return Err(CompileError::SyntaxError(
                                    "Expected ']' after array size".to_string(),
                                    Some(self.current_span.clone()),
                                ));
                            }
                            self.next_token();
                            Ok(AstType::FixedArray { 
                                element_type: Box::new(element_type),
                                size,
                            })
                        }
                        _ => Err(CompileError::SyntaxError(
                            "Expected integer literal for array size".to_string(),
                            Some(self.current_span.clone()),
                        ))
                    }
                } else if self.current_token == Token::Symbol(']') {
                    // Dynamic array [T]
                    self.next_token();
                    Ok(AstType::Array(Box::new(element_type)))
                } else {
                    Err(CompileError::SyntaxError(
                        "Expected ']' or ';' in array type".to_string(),
                        Some(self.current_span.clone()),
                    ))
                }
            }
            Token::Symbol('*') => {
                // Pointer type: *T or function pointer *(params) return_type
                self.next_token();
                
                // Check if it's a function pointer
                if self.current_token == Token::Symbol('(') {
                    // Function pointer: *(param_types) return_type
                    self.next_token();
                    let mut param_types = Vec::new();
                    
                    // Parse parameter types
                    while self.current_token != Token::Symbol(')') {
                        param_types.push(self.parse_type()?);
                        
                        if self.current_token == Token::Symbol(',') {
                            self.next_token();
                        } else if self.current_token != Token::Symbol(')') {
                            return Err(CompileError::SyntaxError(
                                "Expected ',' or ')' in function pointer parameters".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                    }
                    
                    // Skip ')'
                    self.next_token();
                    
                    // Parse return type
                    let return_type = self.parse_type()?;
                    
                    Ok(AstType::FunctionPointer {
                        param_types,
                        return_type: Box::new(return_type),
                    })
                } else {
                    // Regular pointer
                    let pointee_type = self.parse_type()?;
                    Ok(AstType::Pointer(Box::new(pointee_type)))
                }
            }
            Token::Operator(op) if op == "*" => {
                // Pointer type: *T (operator version)
                self.next_token();
                
                // Check if it's a function pointer
                if self.current_token == Token::Symbol('(') {
                    // Function pointer: *(param_types) return_type
                    self.next_token();
                    let mut param_types = Vec::new();
                    
                    // Parse parameter types
                    while self.current_token != Token::Symbol(')') {
                        param_types.push(self.parse_type()?);
                        
                        if self.current_token == Token::Symbol(',') {
                            self.next_token();
                        } else if self.current_token != Token::Symbol(')') {
                            return Err(CompileError::SyntaxError(
                                "Expected ',' or ')' in function pointer parameters".to_string(),
                                Some(self.current_span.clone()),
                            ));
                        }
                    }
                    
                    // Skip ')'
                    self.next_token();
                    
                    // Parse return type
                    let return_type = self.parse_type()?;
                    
                    Ok(AstType::FunctionPointer {
                        param_types,
                        return_type: Box::new(return_type),
                    })
                } else {
                    // Regular pointer
                    let pointee_type = self.parse_type()?;
                    Ok(AstType::Pointer(Box::new(pointee_type)))
                }
            }
            Token::Symbol('&') => {
                // Reference type: &T
                self.next_token();
                let referenced_type = self.parse_type()?;
                Ok(AstType::Ref(Box::new(referenced_type)))
            }
            _ => Err(CompileError::SyntaxError(
                format!("Unexpected token in type: {:?}", self.current_token),
                Some(self.current_span.clone()),
            )),
        }
    }
    
    pub fn parse_type_alias(&mut self) -> Result<crate::ast::TypeAlias> {
        use crate::ast::{TypeAlias, TypeParameter};
        
        // Skip 'type' keyword
        self.next_token();
        
        // Get alias name
        let name = if let Token::Identifier(name) = &self.current_token {
            name.clone()
        } else {
            return Err(CompileError::SyntaxError(
                "Expected type alias name".to_string(),
                Some(self.current_span.clone()),
            ));
        };
        self.next_token();
        
        // Parse optional generic type parameters
        let mut type_params = Vec::new();
        if self.current_token == Token::Operator("<".to_string()) {
            self.next_token();
            loop {
                if let Token::Identifier(param_name) = &self.current_token {
                    type_params.push(TypeParameter {
                        name: param_name.clone(),
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
                            "Expected ',' or '>' in type parameters".to_string(),
                            Some(self.current_span.clone()),
                        ));
                    }
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected type parameter name".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
            }
        }
        
        // Expect '='
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '=' in type alias definition".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();
        
        // Parse the target type
        let target_type = self.parse_type()?;
        
        Ok(TypeAlias {
            name,
            type_params,
            target_type,
        })
    }
}
