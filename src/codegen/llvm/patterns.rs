use super::LLVMCompiler;
use crate::ast::{Pattern, Expression};
use crate::error::CompileError;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use inkwell::IntPredicate;
use std::collections::HashMap;

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_pattern_test(
        &mut self,
        scrutinee_val: &BasicValueEnum<'ctx>,
        pattern: &Pattern,
    ) -> Result<(IntValue<'ctx>, Vec<(String, BasicValueEnum<'ctx>)>), CompileError> {
        let mut bindings = Vec::new();
        
        let matches = match pattern {
            Pattern::Literal(lit_expr) => {
                let pattern_val = self.compile_expression(lit_expr)?;
                self.values_equal(scrutinee_val, &pattern_val)?
            }
            
            Pattern::Wildcard => {
                self.context.bool_type().const_int(1, false)
            }
            
            Pattern::Identifier(name) => {
                bindings.push((name.clone(), *scrutinee_val));
                self.context.bool_type().const_int(1, false)
            }
            
            Pattern::Range { start, end, inclusive } => {
                let start_val = self.compile_expression(start)?;
                let end_val = self.compile_expression(end)?;
                
                if !scrutinee_val.is_int_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "integer for range pattern".to_string(),
                        found: format!("{:?}", scrutinee_val.get_type()),
                        span: None,
                    });
                }
                
                let scrutinee_int = scrutinee_val.into_int_value();
                let start_int = start_val.into_int_value();
                let end_int = end_val.into_int_value();
                
                let ge_start = self.builder.build_int_compare(
                    IntPredicate::SGE,
                    scrutinee_int,
                    start_int,
                    "range_ge"
                )?;
                
                let le_end = if *inclusive {
                    self.builder.build_int_compare(
                        IntPredicate::SLE,
                        scrutinee_int,
                        end_int,
                        "range_le"
                    )?
                } else {
                    self.builder.build_int_compare(
                        IntPredicate::SLT,
                        scrutinee_int,
                        end_int,
                        "range_lt"
                    )?
                };
                
                self.builder.build_and(ge_start, le_end, "range_match")?
            }
            
            Pattern::Or(patterns) => {
                let mut result = self.context.bool_type().const_int(0, false);
                
                for sub_pattern in patterns {
                    let (sub_match, sub_bindings) = self.compile_pattern_test(scrutinee_val, sub_pattern)?;
                    if !sub_bindings.is_empty() {
                        return Err(CompileError::UnsupportedFeature(
                            "Bindings in or-patterns not yet supported".to_string(),
                            None
                        ));
                    }
                    result = self.builder.build_or(result, sub_match, "or_match")?;
                }
                
                result
            }
            
            Pattern::Binding { name, pattern } => {
                bindings.push((name.clone(), *scrutinee_val));
                let (sub_match, mut sub_bindings) = self.compile_pattern_test(scrutinee_val, pattern)?;
                bindings.append(&mut sub_bindings);
                sub_match
            }
            
            Pattern::Struct { .. } | Pattern::EnumVariant { .. } => {
                return Err(CompileError::UnsupportedFeature(
                    "Struct and enum patterns not yet implemented".to_string(),
                    None
                ));
            }
        };
        
        Ok((matches, bindings))
    }
    
    fn values_equal(
        &mut self,
        val1: &BasicValueEnum<'ctx>,
        val2: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, CompileError> {
        if val1.is_int_value() && val2.is_int_value() {
            Ok(self.builder.build_int_compare(
                IntPredicate::EQ,
                val1.into_int_value(),
                val2.into_int_value(),
                "int_eq"
            )?)
        } else if val1.is_float_value() && val2.is_float_value() {
            Ok(self.builder.build_float_compare(
                inkwell::FloatPredicate::OEQ,
                val1.into_float_value(),
                val2.into_float_value(),
                "float_eq"
            )?)
        } else if val1.is_pointer_value() && val2.is_pointer_value() {
            Ok(self.builder.build_int_compare(
                IntPredicate::EQ,
                val1.into_pointer_value(),
                val2.into_pointer_value(),
                "ptr_eq"
            )?)
        } else {
            Err(CompileError::TypeMismatch {
                expected: "matching types for comparison".to_string(),
                found: format!("{:?} vs {:?}", val1.get_type(), val2.get_type()),
                span: None,
            })
        }
    }
    
    pub fn apply_pattern_bindings(
        &mut self,
        bindings: &[(String, BasicValueEnum<'ctx>)]
    ) -> HashMap<String, (PointerValue<'ctx>, crate::ast::AstType)> {
        let mut saved = HashMap::new();
        
        for (name, value) in bindings {
            if let Some(existing) = self.variables.get(name) {
                saved.insert(name.clone(), existing.clone());
            }
            
            let alloca = self.builder.build_alloca(value.get_type(), name).unwrap();
            self.builder.build_store(alloca, *value).unwrap();
            
            let ast_type = match value {
                BasicValueEnum::IntValue(iv) => {
                    match iv.get_type().get_bit_width() {
                        8 => crate::ast::AstType::I8,
                        16 => crate::ast::AstType::I16,
                        32 => crate::ast::AstType::I32,
                        64 => crate::ast::AstType::I64,
                        _ => crate::ast::AstType::I64,
                    }
                }
                BasicValueEnum::FloatValue(fv) => {
                    if fv.get_type() == self.context.f32_type() {
                        crate::ast::AstType::F32
                    } else {
                        crate::ast::AstType::F64
                    }
                }
                _ => crate::ast::AstType::I64,
            };
            
            self.variables.insert(name.clone(), (alloca, ast_type));
        }
        
        saved
    }
    
    pub fn restore_variables(
        &mut self,
        saved: HashMap<String, (PointerValue<'ctx>, crate::ast::AstType)>
    ) {
        for (name, value) in saved {
            self.variables.insert(name, value);
        }
    }
}