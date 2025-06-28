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
                    "String" => Ok(AstType::String),
                    "void" => Ok(AstType::Void),
                    _ => {
                        // Could be a custom type (struct, enum, etc.)
                        // For now, treat as a generic type
                        Ok(AstType::Generic {
                            name: type_name,
                            type_args: vec![],
                        })
                    }
                }
            }
            Token::Symbol('[') => {
                // Array type: [T] (dynamic array)
                self.next_token();
                let element_type = self.parse_type()?;
                
                if self.current_token != Token::Symbol(']') {
                    return Err(CompileError::SyntaxError(
                        "Expected ']' in array type".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();
                Ok(AstType::Array(Box::new(element_type)))
            }
            Token::Symbol('*') => {
                // Pointer type: *T
                self.next_token();
                let pointee_type = self.parse_type()?;
                Ok(AstType::Pointer(Box::new(pointee_type)))
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
}
