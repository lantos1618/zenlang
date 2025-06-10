use crate::ast::{self, BinaryOperator, Expression, Statement, AstType as AstType};
use crate::error::{CompileError, Result};

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, BasicMetadataTypeEnum, FunctionType, StructType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
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
            _ => Err(CompileError::InternalError("Expected basic type".to_string())),
        }
    }

    fn into_function_type(self) -> Result<FunctionType<'ctx>> {
        match self {
            Type::Function(t) => Ok(t),
            _ => Err(CompileError::InternalError("Expected function type".to_string())),
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

        self.module.add_function(&ext_func.name, function_type, None);
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

        self.module.add_function(&function.name, function_type, None);
        Ok(())
    }

    /// Compiles a single function.
    pub fn compile_function(&mut self, function: &ast::Function) -> Result<()> {
        let function_value = self.module.get_function(&function.name)
            .ok_or_else(|| CompileError::InternalError("Function not declared".to_string()))?;
            
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

        // Only emit return if not already returned
        if let AstType::Void = function.return_type {
            self.builder.build_return(None)?;
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
                    Type::Function(f) => self.context.ptr_type(AddressSpace::default()).into(),
                    Type::Void => {
                        return Err(CompileError::InternalError(
                            "Cannot declare variable of type void".to_string(),
                        ))
                    }
                };
                let alloca = self.builder.build_alloca(llvm_type, name)?;
                self.variables.insert(name.clone(), (alloca, type_.clone()));

                if let Some(init) = initializer {
                    let value = self.compile_expression(init)?;
                    self.builder.build_store(alloca, value)?;
                }

                Ok(())
            }
            Statement::VariableAssignment { name, value } => {
                let (alloca, type_) = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CompileError::UndefinedVariable(name.clone()))?;

                let value = self.compile_expression(value)?;
                self.builder.build_store(*alloca, value)?;
                Ok(())
            }
            Statement::PointerAssignment { pointer, value } => {
                let ptr = self.compile_expression(pointer)?;
                let val = self.compile_expression(value)?;
                
                match ptr {
                    BasicValueEnum::PointerValue(ptr) => {
                        self.builder.build_store(ptr, val)?;
                        Ok(())
                    }
                    _ => Err(CompileError::InvalidPointerOperation("Expected pointer type".to_string())),
                }
            }
            Statement::Loop { condition, body } => {
                // Create the loop blocks
                let function = self.current_function
                    .ok_or_else(|| CompileError::InternalError("No current function".to_string()))?;
                let loop_cond = self.context.append_basic_block(function, "loop_cond");
                let loop_body = self.context.append_basic_block(function, "loop_body");
                let after_loop = self.context.append_basic_block(function, "after_loop");

                // Branch to the loop header
                self.builder.build_unconditional_branch(loop_cond)?;

                // Set up the loop header
                self.builder.position_at_end(loop_cond);
                let cond = self.compile_expression(condition)?;
                self.builder.build_conditional_branch(cond.into_int_value(), loop_body, after_loop)?;

                // Compile the loop body
                self.builder.position_at_end(loop_body);
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                self.builder.build_unconditional_branch(loop_cond)?;

                // Set up the exit block
                self.builder.position_at_end(after_loop);
                Ok(())
            }
        }
    }

    /// Compiles a single expression.
    fn compile_expression(&self, expr: &Expression) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expression::Integer8(n) => Ok(self.context.i8_type().const_int(*n as u64, true).into()),
            Expression::Integer32(n) => Ok(self.context.i32_type().const_int(*n as u64, true).into()),
            Expression::Integer64(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
            Expression::Float(n) => Ok(self.context.f64_type().const_float(*n).into()),
            Expression::String(s) => {
                let string_value = self.context.const_string(s.as_bytes(), false);
                let global = self.module.add_global(
                    string_value.get_type(),
                    None,
                    "string",
                );
                global.set_initializer(&string_value);
                global.set_constant(true);
                Ok(global.as_pointer_value().into())
            }
            Expression::Identifier(name) => {
                // Check for function name first
                if let Some(func) = self.module.get_function(name) {
                    // A function name used as a value is its pointer
                    if !self.variables.contains_key(name) {
                        return Ok(func.as_global_value().as_pointer_value().into());
                    }
                }

                // Check for variable
                let (alloca, type_) = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CompileError::UndefinedVariable(name.clone()))?;

                let load_type = match self.to_llvm_type(type_)? {
                    Type::Basic(b) => b,
                    Type::Function(f) => self.context.ptr_type(AddressSpace::default()).into(),
                    Type::Void => {
                        return Err(CompileError::InternalError(
                            "Cannot load void type".to_string(),
                        ))
                    }
                };

                Ok(self.builder.build_load(load_type, *alloca, name)?)
            }
            Expression::BinaryOp { left, op, right } => {
                let l = self.compile_expression(left)?;
                let r = self.compile_expression(right)?;

                match (l, r) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => match op {
                        BinaryOperator::Add => Ok(self.builder.build_int_add(l, r, "add")?.into()),
                        BinaryOperator::Subtract => Ok(self.builder.build_int_sub(l, r, "sub")?.into()),
                        BinaryOperator::Multiply => Ok(self.builder.build_int_mul(l, r, "mul")?.into()),
                        BinaryOperator::Divide => Ok(self.builder.build_int_signed_div(l, r, "div")?.into()),
                        BinaryOperator::Equals => Ok(self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eq")?.into()),
                        BinaryOperator::NotEquals => Ok(self.builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "ne")?.into()),
                        BinaryOperator::LessThan => Ok(self.builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lt")?.into()),
                        BinaryOperator::GreaterThan => Ok(self.builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "gt")?.into()),
                        BinaryOperator::LessThanEquals => Ok(self.builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, "le")?.into()),
                        BinaryOperator::GreaterThanEquals => Ok(self.builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, "ge")?.into()),
                    },
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => match op {
                        BinaryOperator::Add => Ok(self.builder.build_float_add(l, r, "add")?.into()),
                        BinaryOperator::Subtract => Ok(self.builder.build_float_sub(l, r, "sub")?.into()),
                        BinaryOperator::Multiply => Ok(self.builder.build_float_mul(l, r, "mul")?.into()),
                        BinaryOperator::Divide => Ok(self.builder.build_float_div(l, r, "div")?.into()),
                        BinaryOperator::Equals => Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eq")?.into()),
                        BinaryOperator::NotEquals => Ok(self.builder.build_float_compare(inkwell::FloatPredicate::ONE, l, r, "ne")?.into()),
                        BinaryOperator::LessThan => Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OLT, l, r, "lt")?.into()),
                        BinaryOperator::GreaterThan => Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OGT, l, r, "gt")?.into()),
                        BinaryOperator::LessThanEquals => Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OLE, l, r, "le")?.into()),
                        BinaryOperator::GreaterThanEquals => Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OGE, l, r, "ge")?.into()),
                    },
                    (l, r) => Err(CompileError::InvalidBinaryOperation {
                        op: format!("{:?}", op),
                        left: format!("{:?}", l),
                        right: format!("{:?}", r),
                    }),
                }
            }
            Expression::FunctionCall { name, args } => {
                let compiled_args: Vec<BasicValueEnum> = args
                    .iter()
                    .map(|arg| self.compile_expression(arg))
                    .collect::<Result<Vec<_>>>()?;
                let metadata_args: Vec<_> = compiled_args.iter().map(|arg| (*arg).into()).collect();

                // If name is in variables, it's a function pointer call
                if let Some((var_ptr, type_)) = self.variables.get(name) {
                    if let AstType::Function { .. } = type_ {
                        // It's a function pointer variable. Load the pointer.
                        let func_type = self.to_llvm_type(type_)?.into_function_type()?;
                        let func_ptr_type = self.context.ptr_type(AddressSpace::default());

                        let func_ptr = self
                            .builder
                            .build_load(func_ptr_type, *var_ptr, &format!("{}_ptr", name))?
                            .into_pointer_value();

                        let call = self.builder.build_indirect_call(func_type, func_ptr, &metadata_args, "call")?;
                        return Ok(call.try_as_basic_value().left().unwrap());
                    }
                }

                // Otherwise, it's a direct function call
                let function = self.module.get_function(name).ok_or_else(|| {
                    CompileError::UndefinedFunction(name.clone())
                })?;
                let call = self.builder.build_call(function, &metadata_args, "call")?;

                Ok(call.try_as_basic_value().left().unwrap())
            }
            Expression::Conditional { scrutinee, arms } => {
                let scrutinee_value = self.compile_expression(scrutinee)?;
                let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                
                // We need to generate actual comparisons and branches
                let merge_block = self.context.append_basic_block(current_function, "merge");
                
                // Prepare phi node - but we need to know the type
                let result_type = if !arms.is_empty() {
                    self.compile_expression(&arms[0].1)?.get_type()
                } else {
                    return Err(CompileError::InternalError("Empty conditional arms".to_string()));
                };
                
                // Return to where we were
                let entry_block = self.builder.get_insert_block().unwrap();
                self.builder.position_at_end(merge_block);
                let phi = self.builder.build_phi(result_type, "result")?;
                
                // Go back to entry to generate comparisons
                self.builder.position_at_end(entry_block);
                
                let mut incoming: Vec<(BasicValueEnum<'ctx>, BasicBlock<'ctx>)> = Vec::new();
                let mut next_test_block = None;
                
                for (i, (pattern, body)) in arms.iter().enumerate() {
                    let is_last = i == arms.len() - 1;
                    
                    // Create blocks for this arm
                    let test_block = if i == 0 {
                        entry_block
                    } else {
                        next_test_block.unwrap()
                    };
                    
                    let then_block = self.context.append_basic_block(current_function, &format!("then{}", i));
                    next_test_block = if !is_last {
                        Some(self.context.append_basic_block(current_function, &format!("test{}", i + 1)))
                    } else {
                        None
                    };
                    
                    self.builder.position_at_end(test_block);
                    
                    // Generate comparison for pattern
                    let pattern_value = self.compile_expression(pattern)?;
                    
                    // Compare scrutinee with pattern
                    let condition = match (scrutinee_value, pattern_value) {
                        (BasicValueEnum::IntValue(s), BasicValueEnum::IntValue(p)) => {
                            self.builder.build_int_compare(inkwell::IntPredicate::EQ, s, p, "cmp")?
                        }
                        (BasicValueEnum::FloatValue(s), BasicValueEnum::FloatValue(p)) => {
                            self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, s, p, "cmp")?
                        }
                        _ => return Err(CompileError::InvalidPatternMatching("Pattern type mismatch".to_string())),
                    };
                    
                    // Branch based on comparison
                    if let Some(next) = next_test_block {
                        self.builder.build_conditional_branch(condition, then_block, next)?;
                    } else {
                        // Last pattern - if not matched, still go to merge (or could error)
                        self.builder.build_conditional_branch(condition, then_block, merge_block)?;
                    }
                    
                    // Generate body
                    self.builder.position_at_end(then_block);
                    let body_value = self.compile_expression(body)?;
                    incoming.push((body_value, self.builder.get_insert_block().unwrap()));
                    self.builder.build_unconditional_branch(merge_block)?;
                }
                
                // Add all incoming values to phi
                self.builder.position_at_end(merge_block);
                for (value, block) in &incoming {
                    phi.add_incoming(&[(value, *block)]);
                }
                
                Ok(phi.as_basic_value())
            }
            Expression::AddressOf(expr) => {
                let value = self.compile_expression(expr)?;
                // For variables, we can get their alloca directly
                if let Expression::Identifier(name) = expr.as_ref() {
                    if let Some((alloca, _)) = self.variables.get(name) {
                        return Ok((*alloca).into());
                    }
                }
                // For other expressions, we need to create a temporary
                let temp = self.builder.build_alloca(value.get_type(), "temp")?;
                self.builder.build_store(temp, value)?;
                Ok(temp.into())
            }
            Expression::Dereference(expr) => {
                let ptr = self.compile_expression(expr)?;
                match ptr {
                    BasicValueEnum::PointerValue(ptr_val) => {
                        // To load from a pointer, we need to know what type it points to.
                        let pointee_ast_type = match expr.as_ref() {
                            Expression::Identifier(name) => {
                                let (_, var_ast_type) = self
                                    .variables
                                    .get(name)
                                    .ok_or_else(|| CompileError::UndefinedVariable(name.clone()))?;

                                if let AstType::Pointer(pointee) = var_ast_type {
                                    *pointee.clone()
                                } else {
                                    return Err(CompileError::InvalidPointerOperation(format!(
                                        "Variable '{}' is not a pointer type",
                                        name
                                    )));
                                }
                            }
                            _ => {
                                return Err(CompileError::InvalidPointerOperation(
                                    "Dereferencing complex expressions is not yet supported".to_string(),
                                ))
                            }
                        };
                        let ptr_type = self.to_llvm_type(&pointee_ast_type)?.into_basic_type()?;
                        Ok(self.builder.build_load(ptr_type, ptr_val, "deref")?)
                    }
                    _ => Err(CompileError::InvalidPointerOperation(
                        "Cannot dereference non-pointer type".to_string(),
                    )),
                }
            }
            Expression::PointerOffset { pointer, offset } => {
                let ptr = self.compile_expression(pointer)?;
                let off = self.compile_expression(offset)?;

                match (ptr, off) {
                    (BasicValueEnum::PointerValue(ptr), BasicValueEnum::IntValue(off)) => {
                        // To calculate pointer offsets, we need the pointee type.
                        let pointee_ast_type = match pointer.as_ref() {
                            Expression::Identifier(name) => {
                                let (_, var_ast_type) = self
                                    .variables
                                    .get(name)
                                    .ok_or_else(|| CompileError::UndefinedVariable(name.clone()))?;

                                if let AstType::Pointer(pointee) = var_ast_type {
                                    *pointee.clone()
                                } else {
                                    return Err(CompileError::InvalidPointerOperation(format!(
                                        "Variable '{}' is not a pointer type",
                                        name
                                    )));
                                }
                            }
                            _ => return Err(CompileError::InvalidPointerOperation(
                                "Offsetting complex pointer expressions is not supported".to_string()
                            ))
                        };
                        let pointee_llvm_type = self.to_llvm_type(&pointee_ast_type)?.into_basic_type()?;
                        let offset_ptr = unsafe { self.builder.build_gep(pointee_llvm_type, ptr, &[off], "offset_ptr")? };
                        Ok(offset_ptr.into())
                    }
                    _ => Err(CompileError::InvalidPointerOperation(
                        "Invalid pointer offset types".to_string(),
                    )),
                }
            }
            Expression::StructLiteral { name, fields } => {
                let struct_type = self.module.get_struct_type(name)
                    .ok_or_else(|| CompileError::InternalError(format!("Struct type {} not found", name)))?;
                
                let field_values: Result<Vec<BasicValueEnum>> = fields
                    .iter()
                    .map(|(_, expr)| self.compile_expression(expr))
                    .collect();
                let field_values = field_values?;
                
                Ok(self.context.const_struct(&field_values, false).into())
            }
            Expression::StructField { struct_, field } => {
                // To access a struct field, we need a pointer to the struct.
                // We will inspect the `struct_` expression to get this pointer,
                // rather than compiling it to a value. This avoids loading the whole struct.

                let (struct_ptr, struct_ast_type) =
                    if let Expression::Identifier(name) = struct_.as_ref() {
                        // Case 1: `variable.field`. The variable is an alloca, which is a pointer.
                        let (ptr, ast_type) = self.variables.get(name)
                            .ok_or_else(|| CompileError::UndefinedVariable(name.clone()))?;
                        (*ptr, ast_type.clone())
                    } else if let Expression::Dereference(ptr_expr) = struct_.as_ref() {
                        // Case 2: `(*pointer).field`. Compile the inner expression to get the pointer.
                        let ptr_to_struct = self.compile_expression(ptr_expr)?.into_pointer_value();

                        // We need the AST type of what `ptr_expr` points to.
                        // This logic is fragile and only handles identifiers for now.
                        let pointee_type = if let Expression::Identifier(ptr_name) = ptr_expr.as_ref() {
                            let (_, var_type) = self.variables.get(ptr_name)
                                .ok_or_else(|| CompileError::UndefinedVariable(ptr_name.clone()))?;
                            if let AstType::Pointer(pointee) = var_type {
                                *pointee.clone()
                            } else {
                                return Err(CompileError::InvalidPointerOperation(format!("Variable '{}' is not a pointer", ptr_name)));
                            }
                        } else {
                            return Err(CompileError::InternalError("Field access on complex pointer expressions not yet supported".to_string()));
                        };
                        (ptr_to_struct, pointee_type)
                    } else {
                        // Other cases (like r-value `function().field`) would need `extractvalue`
                        // and a more robust way to get an expression's type. Not supported yet.
                        return Err(CompileError::InternalError("Field access on r-values is not supported".to_string()));
                    };

                // Common path for l-values (cases 1 and 2): use GEP and load.
                let (field_index, field_llvm_type) =
                    if let AstType::Struct { fields, .. } = &struct_ast_type {
                        let index = fields.iter()
                            .position(|(f_name, _)| f_name == field)
                            .map(|i| i as u32)
                            .ok_or_else(|| CompileError::InternalError(format!("Field '{}' not found in struct", field)))?;

                        let field_ast_type = &fields[index as usize].1;
                        let llvm_type = self.to_llvm_type(field_ast_type)?.into_basic_type()?;
                        (index, llvm_type)
                    } else {
                        return Err(CompileError::InternalError("Expression is not a struct type".to_string()));
                    };

                let llvm_struct_type = self.to_llvm_type(&struct_ast_type)?.into_basic_type()?.into_struct_type();

                let field_ptr = unsafe {
                    self.builder.build_struct_gep(llvm_struct_type, struct_ptr, field_index, field)?
                };

                Ok(self.builder.build_load(field_llvm_type, field_ptr, field)?)
            }
        }
    }

    /// Converts a Lynlang type to an LLVM type or void
    fn to_llvm_type(&self, type_: &AstType) -> Result<Type<'ctx>> {
        match type_ {
            AstType::Int8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::Int32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::Int64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::Float => Ok(Type::Basic(self.context.f64_type().into())),
            AstType::String => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())),
            AstType::Void => Ok(Type::Void),
            AstType::Function { args, return_type } => {
                let ret_type = self.to_llvm_type(return_type)?;
                let param_types: Result<Vec<BasicMetadataTypeEnum>> = args
                    .iter()
                    .map(|t| {
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

                Ok(Type::Function(function_type))
            }
            AstType::Pointer(pointee_type) => {
                if let AstType::Void = **pointee_type {
                    return Err(CompileError::InvalidPointerOperation(
                        "Cannot create a pointer to void".to_string(),
                    ));
                }
                // Pointers are opaque, so they are all just `ptr`.
                // The type of what they point to is tracked by our compiler, not LLVM's type system.
                let _ = self.to_llvm_type(pointee_type)?; // ensure pointee is a valid type
                Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into()))
            }
            AstType::Struct { name, fields } => {
                if let Some(existing_type) = self.module.get_struct_type(name) {
                    if !existing_type.is_opaque() {
                        return Ok(Type::Basic(existing_type.into()));
                    }
                    // If it exists but is opaque, we will set its body below.
                }

                let struct_type = self.module.get_struct_type(name)
                    .unwrap_or_else(|| self.context.opaque_struct_type(name));

                let field_types: Result<Vec<BasicTypeEnum>> = fields
                    .iter()
                    .map(|(_, t)| self.to_llvm_type(t).and_then(|t| t.into_basic_type()))
                    .collect();
                let field_types = field_types?;
                struct_type.set_body(&field_types, false);
                Ok(Type::Basic(struct_type.into()))
            }
        }
    }

    /// Helper to get just the BasicTypeEnum, or error if void
    fn expect_basic_type<'a>(&self, t: Type<'a>) -> Result<BasicTypeEnum<'a>> {
        match t {
            Type::Basic(b) => Ok(b),
            Type::Function(_) => Err(CompileError::InvalidFunctionType("Expected non-function type".to_string())),
            Type::Void => Err(CompileError::InvalidFunctionType("Expected non-void type".to_string())),
        }
    }

    /// Safely get a struct field pointer, validating the type and field index
    fn get_struct_field_ptr(
        &self,
        struct_type: StructType<'ctx>,
        struct_value: BasicValueEnum<'ctx>,
        field_name: &str,
    ) -> Result<PointerValue<'ctx>> {
        // Find the field index by name
        let field_index = struct_type.get_field_type_at_index(0)
            .ok_or_else(|| CompileError::InternalError(format!("Field {} not found", field_name)))?;
        
        // Get the field pointer using GEP
        let field_ptr = unsafe {
            self.builder.build_struct_gep(struct_type, struct_value.into_pointer_value(), 0, field_name)?
        };
        Ok(field_ptr)
    }

    /// Safely load a struct field value
    fn load_struct_field(
        &self,
        struct_type: StructType<'ctx>,
        struct_value: BasicValueEnum<'ctx>,
        field_name: &str,
    ) -> Result<BasicValueEnum<'ctx>> {
        let field_ptr = self.get_struct_field_ptr(struct_type, struct_value, field_name)?;
        // With opaque pointers, we need to get the field type from the struct type
        let field_type = struct_type.get_field_type_at_index(0)
            .ok_or_else(|| CompileError::InternalError(format!("Field {} not found", field_name)))?;
        Ok(self.builder.build_load(field_type, field_ptr, field_name)?)
    }

    /// Compile a struct field access expression
    fn compile_struct_field_access(
        &self,
        struct_value: BasicValueEnum<'ctx>,
        field_name: &str,
    ) -> Result<BasicValueEnum<'ctx>> {
        let struct_type = match struct_value.get_type() {
            BasicTypeEnum::StructType(t) => t,
            _ => return Err(CompileError::InternalError("Expected struct type".to_string())),
        };
        self.load_struct_field(struct_type, struct_value, field_name)
    }
} 