use crate::ast::{self, BinaryOperator, Expression, Statement, AstType as AstType};
use crate::error::{CompileError, Result, Span};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, BasicMetadataTypeEnum, FunctionType},
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue, IntValue, FloatValue},
    IntPredicate,
    FloatPredicate,
    AddressSpace,
};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum Type<'ctx> {
    Basic(BasicTypeEnum<'ctx>),
    Function(FunctionType<'ctx>),
    Void,
}

impl<'ctx> Type<'ctx> {
    fn into_basic_type(self) -> Result<BasicTypeEnum<'ctx>> {
        match self {
            Type::Basic(t) => Ok(t),
            _ => Err(CompileError::InternalError("Expected basic type".to_string(), None)),
        }
    }

    fn into_function_type(self) -> Result<FunctionType<'ctx>> {
        match self {
            Type::Function(t) => Ok(t),
            _ => Err(CompileError::InternalError("Expected function type".to_string(), None)),
        }
    }
}

/// The `Compiler` struct is responsible for compiling a Lynlang AST into LLVM IR.
pub struct Compiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    current_function: Option<FunctionValue<'ctx>>,
    variables: HashMap<String, (PointerValue<'ctx>, AstType)>,
}

impl<'ctx> Compiler<'ctx> {
    /// Creates a new `Compiler` instance.
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("main");
        let builder = context.create_builder();
        Self {
            context,
            builder,
            module,
            current_function: None,
            variables: HashMap::new(),
        }
    }

    /// Compiles a Lynlang module.
    /// This is the main entry point for compilation.
    pub fn compile_program(&mut self, program: &ast::Program) -> Result<()> {
        // First pass: declare all functions (including external)
        for declaration in &program.declarations {
            match declaration {
                ast::Declaration::ExternalFunction(ext_func) => {
                    self.declare_external_function(ext_func)?;
                }
                ast::Declaration::Function(func) => {
                    self.declare_function(func)?;
                }
            }
        }
        
        // Second pass: compile function bodies
        for declaration in &program.declarations {
            if let ast::Declaration::Function(func) = declaration {
                self.compile_function(func)?;
            }
        }
        Ok(())
    }

    /// Declares an external function (C FFI)
    fn declare_external_function(&mut self, ext_func: &ast::ExternalFunction) -> Result<()> {
        let ret_type = self.to_llvm_type(&ext_func.return_type)?;
        let param_types: Result<Vec<BasicMetadataTypeEnum>> = ext_func.args
            .iter()
            .map(|t| {
                self.to_llvm_type(t)
                    .and_then(|lyn_type| self.expect_basic_type(lyn_type))
                    .map(|basic_type| basic_type.into())
            })
            .collect();
        let param_types = param_types?;

        let function_type = match ret_type {
            Type::Basic(b) => b.fn_type(&param_types, ext_func.is_varargs),
            Type::Function(f) => f,
            Type::Void => self.context.void_type().fn_type(&param_types, ext_func.is_varargs),
        };

        // Only declare if not already declared
        if self.module.get_function(&ext_func.name).is_none() {
            self.module.add_function(&ext_func.name, function_type, None);
        }
        Ok(())
    }

    /// Declares a function (without compiling its body)
    fn declare_function(&mut self, function: &ast::Function) -> Result<()> {
        let ret_type = self.to_llvm_type(&function.return_type)?;
        let param_types: Result<Vec<BasicMetadataTypeEnum>> = function
            .args
            .iter()
            .map(|(_, t)| {
                self.to_llvm_type(t)
                    .and_then(|lyn_type| self.expect_basic_type(lyn_type))
                    .map(|basic_type| basic_type.into())
            })
            .collect();
        let param_types = param_types?;

        let function_type = match ret_type {
            Type::Basic(b) => b.fn_type(&param_types, false),
            Type::Function(f) => f,
            Type::Void => self.context.void_type().fn_type(&param_types, false),
        };

        // Only declare if not already declared
        if self.module.get_function(&function.name).is_none() {
            self.module.add_function(&function.name, function_type, None);
        }
        Ok(())
    }

    /// Compiles a single function.
    pub fn compile_function(&mut self, function: &ast::Function) -> Result<()> {
        let function_value = self.module.get_function(&function.name)
            .ok_or_else(|| CompileError::InternalError("Function not declared".to_string(), None))?;
            
        let entry_block = self.context.append_basic_block(function_value, "entry");
        self.builder.position_at_end(entry_block);
        self.current_function = Some(function_value);

        // Clear variables from previous function
        self.variables.clear();

        // Store function parameters in variables
        for (i, (name, type_)) in function.args.iter().enumerate() {
            let param = function_value.get_nth_param(i as u32).unwrap();
            let alloca = self.builder.build_alloca(param.get_type(), name)?;
            self.builder.build_store(alloca, param)?;
            self.variables.insert(name.clone(), (alloca, type_.clone()));
        }

        for statement in &function.body {
            self.compile_statement(statement)?;
        }

        // Check if we need to add a return statement
        if let Some(block) = self.builder.get_insert_block() {
            if block.get_terminator().is_none() {
                if let AstType::Void = function.return_type {
                    self.builder.build_return(None)?;
                } else {
                    return Err(CompileError::MissingReturnStatement(function.name.clone(), None));
                }
            }
        }

        self.current_function = None;
        Ok(())
    }

    /// Compiles a single statement.
    fn compile_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                Ok(())
            }
            Statement::Return(expr) => {
                let value = self.compile_expression(expr)?;
                self.builder.build_return(Some(&value))?;
                Ok(())
            }
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
            } => {
                let llvm_type = match self.to_llvm_type(type_)? {
                    Type::Basic(b) => b,
                    Type::Function(_f) => self.context.ptr_type(AddressSpace::default()).into(),
                    Type::Void => {
                        return Err(CompileError::InternalError(
                            "Cannot declare variable of type void".to_string(),
                            None,
                        ));
                    }
                };

                let alloca = self.builder.build_alloca(llvm_type, name)?;
                if let Some(init) = initializer {
                    let value = self.compile_expression(init)?;
                    self.builder.build_store(alloca, value)?;
                }
                self.variables.insert(name.clone(), (alloca, type_.clone()));
                Ok(())
            }
            Statement::VariableAssignment { name, value } => {
                let (alloca, _type_) = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CompileError::UndeclaredVariable(name.clone(), None))?;

                let value = self.compile_expression(value)?;
                self.builder.build_store(*alloca, value)?;
                Ok(())
            }
            Statement::PointerAssignment { pointer, value } => {
                let ptr = self.compile_expression(&pointer)?;
                let val = self.compile_expression(&value)?;
                
                match ptr {
                    BasicValueEnum::PointerValue(ptr) => {
                        self.builder.build_store(ptr, val)?;
                        Ok(())
                    }
                    _ => Err(CompileError::TypeMismatch {
                        expected: "pointer".to_string(),
                        found: format!("{:?}", ptr.get_type()),
                        span: None,
                    }),
                }
            }
            Statement::Loop { condition, body } => {
                // Create the loop blocks
                let function = self.current_function
                    .ok_or_else(|| CompileError::InternalError("No current function".to_string(), None))?;
                let loop_cond = self.context.append_basic_block(function, "loop_cond");
                let loop_body = self.context.append_basic_block(function, "loop_body");
                let after_loop = self.context.append_basic_block(function, "after_loop");

                // Branch to the loop header
                self.builder.build_unconditional_branch(loop_cond)?;

                // Set up the loop header
                self.builder.position_at_end(loop_cond);
                let cond = self.compile_expression(condition)?;
                
                // Ensure condition is a boolean
                if !cond.is_int_value() || !cond.into_int_value().get_type().is_int_type() || 
                   cond.into_int_value().get_type().get_bit_width() != 1 {
                    return Err(CompileError::InvalidLoopCondition(
                        "Loop condition must be a boolean (i1) value".to_string(),
                        None,
                    ));
                }
                
                self.builder.build_conditional_branch(cond.into_int_value(), loop_body, after_loop)?;

                // Compile the loop body
                self.builder.position_at_end(loop_body);
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                self.builder.build_unconditional_branch(loop_cond)?;

                // Set up the after loop block
                self.builder.position_at_end(after_loop);
                Ok(())
            }
        }
    }

    /// Compiles a single expression.
    fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expression::Integer8(n) => {
                Ok(self.context.i8_type().const_int(*n as u64, false).into())
            }
            Expression::Integer16(n) => {
                Ok(self.context.i16_type().const_int(*n as u64, false).into())
            }
            Expression::Integer32(n) => {
                Ok(self.context.i32_type().const_int(*n as u64, false).into())
            }
            Expression::Integer64(n) => {
                Ok(self.context.i64_type().const_int(*n as u64, false).into())
            }
            Expression::Float(n) => {
                Ok(self.context.f64_type().const_float(*n).into())
            }
            Expression::String(val) => {
                let ptr = self.builder.build_global_string_ptr(val, "str")?;
                Ok(ptr.as_pointer_value().into())
            }
            Expression::Identifier(name) => {
                let (alloca, _) = self.variables.get(name)
                    .ok_or_else(|| CompileError::UndeclaredVariable(name.clone(), None))?;
                let load = self.builder.build_load(alloca.get_type(), *alloca, name)?;
                Ok(load)
            }
            Expression::BinaryOp { op, left, right } => {
                let left_val = self.compile_expression(left)?;
                let right_val = self.compile_expression(right)?;

                match op {
                    BinaryOperator::Add => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_add(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "addtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_add(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "addtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Subtract => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_sub(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "subtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_sub(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "subtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Multiply => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_mul(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "multmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_mul(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "multmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Divide => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_signed_div(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "divtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_div(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "divtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::Equals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::EQ,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "eqtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OEQ,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "eqtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::NotEquals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::NE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "netmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::ONE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "netmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::LessThan => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SLT,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "lttmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OLT,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "lttmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::GreaterThan => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SGT,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "gttmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OGT,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "gttmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::LessThanEquals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SLE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "letmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OLE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "letmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::GreaterThanEquals => {
                        if left_val.is_int_value() && right_val.is_int_value() {
                            let result = self.builder.build_int_compare(
                                IntPredicate::SGE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "getmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            let result = self.builder.build_float_compare(
                                FloatPredicate::OGE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "getmp"
                            )?;
                            Ok(result.into())
                        } else {
                            Err(CompileError::TypeMismatch {
                                expected: "int or float".to_string(),
                                found: "mixed types".to_string(),
                                span: None,
                            })
                        }
                    }
                    BinaryOperator::StringConcat => {
                        Err(CompileError::UnsupportedFeature(
                            "String concatenation not yet implemented".to_string(),
                            None,
                        ))
                    }
                }
            }
            Expression::FunctionCall { name, args } => {
                let function = self.module.get_function(name)
                    .ok_or_else(|| CompileError::UndeclaredFunction(name.clone(), None))?;

                let compiled_args: Result<Vec<BasicValueEnum>> = args
                    .iter()
                    .map(|arg| self.compile_expression(arg))
                    .collect();
                let compiled_args = compiled_args?;

                let call_result = self.builder.build_call(
                    function,
                    &compiled_args.iter().map(|arg| arg.as_basic_value()).collect::<Vec<_>>(),
                    "calltmp"
                )?;

                if function.get_type().get_return_type().is_none() {
                    Ok(self.context.void_type().const_void().into())
                } else {
                    let basic_value = call_result.try_as_basic_value()?.left()
                        .ok_or_else(|| CompileError::InternalError(
                            "Function call did not return a basic value".to_string(),
                            None,
                        ))?;
                    Ok(basic_value)
                }
            }
            Expression::Conditional { scrutinee, arms } => {
                let parent_function = self.current_function
                    .ok_or_else(|| CompileError::InternalError("No current function for conditional".to_string(), None))?;

                let cond_val = self.compile_expression(scrutinee)?;
                if !cond_val.is_int_value() || !cond_val.into_int_value().get_type().is_int_type() || 
                   cond_val.into_int_value().get_type().get_bit_width() != 1 {
                    return Err(CompileError::TypeMismatch {
                        expected: "boolean (i1) for conditional expression".to_string(),
                        found: format!("{:?}", cond_val.get_type()),
                        span: None,
                    });
                }

                let then_bb = self.context.append_basic_block(parent_function, "then");
                let else_bb = self.context.append_basic_block(parent_function, "else");
                let merge_bb = self.context.append_basic_block(parent_function, "ifcont");

                self.builder.build_conditional_branch(cond_val.into_int_value(), then_bb, else_bb)?;

                // Emit 'then' block
                self.builder.position_at_end(then_bb);
                let then_val = self.compile_expression(&arms[0].1)?;
                self.builder.build_unconditional_branch(merge_bb)?;

                let then_bb_end = self.builder.get_insert_block().unwrap();

                // Emit 'else' block
                self.builder.position_at_end(else_bb);
                let else_val = self.compile_expression(&arms[1].1)?;
                self.builder.build_unconditional_branch(merge_bb)?;

                let else_bb_end = self.builder.get_insert_block().unwrap();

                // Emit 'merge' block
                self.builder.position_at_end(merge_bb);
                let phi = self.builder.build_phi(
                    then_val.get_type(),
                    "iftmp"
                )?;
                phi.add_incoming(&[(&then_val, then_bb_end), (&else_val, else_bb_end)]);

                Ok(phi.as_basic_value())
            }
            Expression::AddressOf(expr) => {
                match &**expr {
                    Expression::Identifier(name) => {
                        let (alloca, _type_) = self
                            .variables
                            .get(name)
                            .ok_or_else(|| CompileError::UndeclaredVariable(name.clone(), None))?;
                        Ok(alloca.as_basic_value_enum())
                    }
                    _ => Err(CompileError::UnsupportedFeature(
                        "AddressOf only supported for identifiers".to_string(),
                        None,
                    )),
                }
            }
            Expression::Dereference(expr) => {
                let ptr_val = self.compile_expression(expr)?;
                if !ptr_val.is_pointer_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "pointer".to_string(),
                        found: format!("{:?}", ptr_val.get_type()),
                        span: None,
                    });
                }
                Ok(self.builder.build_load(ptr_val.into_pointer_value(), "deref_tmp"))
            }
            Expression::PointerOffset { pointer, offset } => {
                let base_val = self.compile_expression(pointer)?;
                let offset_val = self.compile_expression(offset)?;

                if !base_val.is_pointer_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "pointer for pointer offset base".to_string(),
                        found: format!("{:?}", base_val.get_type()),
                        span: None,
                    });
                }
                if !offset_val.is_int_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "integer for pointer offset value".to_string(),
                        found: format!("{:?}", offset_val.get_type()),
                        span: None,
                    });
                }

                unsafe {
                    Ok(self.builder.build_gep(
                        base_val.into_pointer_value(),
                        &[offset_val.into_int_value()],
                        "gep_tmp"
                    )?.into())
                }
            }
            Expression::StructLiteral { .. } => Err(CompileError::UnsupportedFeature(
                "Struct literals not yet implemented".to_string(),
                None,
            )),
            Expression::StructField { .. } => Err(CompileError::UnsupportedFeature(
                "Struct field access not yet implemented".to_string(),
                None,
            )),
            Expression::StringLength(_) => Err(CompileError::UnsupportedFeature(
                "String length not yet implemented".to_string(),
                None,
            )),
        }
    }

    /// Converts a Lynlang type to an LLVM type.
    fn to_llvm_type(&self, type_: &AstType) -> Result<Type<'ctx>> {
        match type_ {
            AstType::Int8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::Int32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::Int64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::Float => Ok(Type::Basic(self.context.f64_type().into())),
            AstType::Void => Ok(Type::Void),
            AstType::Pointer(inner_type) => {
                let inner_llvm_type = self.to_llvm_type(inner_type)?;
                Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into()))
            }
            AstType::String => Ok(Type::Basic(self.context.i8_type().ptr_type(AddressSpace::default()).into())),
            AstType::Function { args, return_type } => {
                let param_types: Result<Vec<BasicMetadataTypeEnum>> = args
                    .iter()
                    .map(|ty| {
                        self.to_llvm_type(ty)
                            .and_then(|lyn_type| self.expect_basic_type(lyn_type))
                            .map(|basic_type| basic_type.into())
                    })
                    .collect();
                let param_types = param_types?;
                let ret_type = self.to_llvm_type(return_type)?;

                let fn_type = match ret_type {
                    Type::Basic(b) => b.fn_type(&param_types, false),
                    Type::Function(f) => f,
                    Type::Void => self.context.void_type().fn_type(&param_types, false),
                };
                Ok(Type::Function(fn_type))
            }
            AstType::Struct { name, fields } => {
                // Check if struct type already exists
                if let Some(struct_type) = self.module.get_struct_type(name) {
                    Ok(Type::Basic(struct_type.into()))
                } else {
                    // Define the struct type (opaque first, then set body)
                    let struct_type = self.context.opaque_struct_type(name);
                    let field_types: Result<Vec<BasicTypeEnum>> = fields
                        .iter()
                        .map(|(_, ty)| {
                            self.to_llvm_type(ty)
                                .and_then(|lyn_type| self.expect_basic_type(lyn_type))
                        })
                        .collect();
                    let field_types = field_types?;
                    struct_type.set_body(&field_types, false); // false for not packed
                    Ok(Type::Basic(struct_type.into()))
                }
            }
        }
    }

    /// Expects a basic type, returning an error if not.
    fn expect_basic_type<'a>(&self, t: Type<'a>) -> Result<BasicTypeEnum<'a>> {
        match t {
            Type::Basic(b) => Ok(b),
            _ => Err(CompileError::InternalError("Expected basic type".to_string(), None)),
        }
    }
} 