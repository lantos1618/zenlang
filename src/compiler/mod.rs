pub mod symbols;

// Re-export the symbols module for easier access
pub use symbols::{Symbol, SymbolTable};

// Rest of the compiler code will go here
// ...
use crate::ast::{self, BinaryOperator, Expression, Statement, AstType as AstType};
use crate::error::CompileError;
use inkwell::{
    context::Context,
    types::AnyType,
    module::Module,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
    builder::Builder,
};
use inkwell::{
    types::{
        AnyTypeEnum, ArrayType, AsTypeRef, BasicType, BasicTypeEnum, FunctionType, IntType,
        PointerType, StructType, VoidType,
    },
    IntPredicate, FloatPredicate,
};
use inkwell::values::BasicValue as _;
use inkwell::AddressSpace;
use inkwell::Either;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum Type<'ctx> {
    Basic(BasicTypeEnum<'ctx>),
    Function(FunctionType<'ctx>),
    Void,
}

impl<'ctx> Type<'ctx> {
    fn into_basic_type(self) -> Result<BasicTypeEnum<'ctx>, CompileError> {
        match self {
            Type::Basic(t) => Ok(t),
            _ => Err(CompileError::InternalError("Expected basic type".to_string(), None)),
        }
    }

    fn into_function_type(self) -> Result<FunctionType<'ctx>, CompileError> {
        match self {
            Type::Function(t) => Ok(t),
            _ => Err(CompileError::InternalError("Expected function type".to_string(), None)),
        }
    }
}

/// The `Compiler` struct is responsible for compiling a Zen AST into LLVM IR.
/// Information about a struct type, including its fields and their indices
#[derive(Debug, Clone)]
struct StructTypeInfo<'ctx> {
    /// The LLVM struct type
    llvm_type: StructType<'ctx>,
    /// Mapping from field name to (index, type)
    fields: HashMap<String, (usize, AstType)>,
}

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub variables: HashMap<String, (PointerValue<'ctx>, AstType)>,
    pub functions: HashMap<String, FunctionValue<'ctx>>,
    pub current_function: Option<FunctionValue<'ctx>>,
    pub symbols: SymbolTable<'ctx>,
    /// Map from struct name to its type information
    struct_types: HashMap<String, StructTypeInfo<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    /// Creates a new `Compiler` instance.
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("main");
        let builder = context.create_builder();
        let mut symbols = SymbolTable::new();
        
        // Add basic types to the symbol table
        let i64_type = context.i64_type();
        let i32_type = context.i32_type();
        let float_type = context.f64_type();
        let bool_type = context.bool_type();
        
        // Add basic types to the symbol table
        symbols.insert("i64", Symbol::Type(i64_type.as_basic_type_enum()));
        symbols.insert("i32", Symbol::Type(i32_type.as_basic_type_enum()));
        symbols.insert("f64", Symbol::Type(float_type.as_basic_type_enum()));
        symbols.insert("bool", Symbol::Type(bool_type.as_basic_type_enum()));
        
        // Create the compiler instance
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            current_function: None,
            symbols,
            struct_types: HashMap::new(),
        }
    }

    /// Gets a type from the symbol table
    fn get_type(&self, name: &str) -> Result<BasicTypeEnum<'ctx>, CompileError> {
        self.symbols.lookup(name)
            .and_then(|sym| match sym {
                Symbol::Type(ty) => Some(*ty),
                _ => None,
            })
            .ok_or_else(|| CompileError::UndeclaredVariable(name.to_string(), None))
    }

    /// Declares a variable in the current scope
    fn declare_variable(&mut self, name: &str, ty: AstType, ptr: PointerValue<'ctx>) -> Result<(), CompileError> {
        let symbol = Symbol::Variable(ptr);
        if self.symbols.exists_in_current_scope(name) {
            return Err(CompileError::UndeclaredVariable(name.to_string(), None));
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }

    /// Looks up a variable and returns its pointer
    fn get_variable(&self, name: &str) -> Result<PointerValue<'ctx>, CompileError> {
        self.symbols.lookup(name)
            .and_then(|sym| match sym {
                Symbol::Variable(ptr) => Some(*ptr),
                _ => None,
            })
            .ok_or_else(|| CompileError::UndeclaredVariable(name.to_string(), None))
    }

    /// Compiles a Zen module.
    /// This is the main entry point for compilation.
    pub fn compile_program(&mut self, program: &ast::Program) -> Result<(), CompileError> {
        println!("Compiling program with {} declarations", program.declarations.len());
        
        // First pass: declare all functions (including external)
        for declaration in &program.declarations {
            match declaration {
                ast::Declaration::ExternalFunction(ext_func) => {
                    println!("Declaring external function: {}", ext_func.name);
                    self.declare_external_function(ext_func)?;
                }
                ast::Declaration::Function(func) => {
                    println!("Declaring function: {}", func.name);
                    self.declare_function(func)?;
                }
            }
        }
        
        // Second pass: compile function bodies
        for declaration in &program.declarations {
            if let ast::Declaration::Function(func) = declaration {
                println!("Compiling function: {}", func.name);
                self.compile_function(func)?;
            }
        }
        
        // Debug: Print all functions in the module
        println!("Functions in module after compilation:");
        for func in self.module.get_functions() {
            println!("  - {}", func.get_name().to_str().unwrap_or("<invalid>"));
        }
        
        Ok(())
    }

    /// Declares an external function (C FFI)
    fn declare_external_function(&mut self, ext_func: &ast::ExternalFunction) -> Result<(), CompileError> {
        let ret_type = self.to_llvm_type(&ext_func.return_type)?;
        
        // First get the basic types for the parameters
        let param_basic_types: Result<Vec<BasicTypeEnum>, CompileError> = ext_func.args
            .iter()
            .map(|t| self.to_llvm_type(t).and_then(|t| t.into_basic_type()))
            .collect();
        
        // Convert basic types to metadata types for the function signature
        let param_metadata: Result<Vec<BasicMetadataTypeEnum>, CompileError> = param_basic_types?
            .into_iter()
            .map(|ty| {
                match ty {
                    BasicTypeEnum::ArrayType(t) => Ok(t.into()),
                    BasicTypeEnum::FloatType(t) => Ok(t.into()),
                    BasicTypeEnum::IntType(t) => Ok(t.into()),
                    BasicTypeEnum::PointerType(t) => Ok(t.into()),
                    BasicTypeEnum::StructType(t) => Ok(t.into()),
                    BasicTypeEnum::VectorType(t) => Ok(t.into()),
                    BasicTypeEnum::ScalableVectorType(t) => Ok(t.into()),
                }
            })
            .collect();
            
        let param_metadata = param_metadata?;

        // Create the function type with the metadata types
        let function_type = match ret_type {
            Type::Basic(b) => {
                let ret_metadata: BasicMetadataTypeEnum = match b {
                    BasicTypeEnum::ArrayType(t) => t.into(),
                    BasicTypeEnum::FloatType(t) => t.into(),
                    BasicTypeEnum::IntType(t) => t.into(),
                    BasicTypeEnum::PointerType(t) => t.into(),
                    BasicTypeEnum::StructType(t) => t.into(),
                    BasicTypeEnum::VectorType(t) => t.into(),
                    BasicTypeEnum::ScalableVectorType(t) => t.into(),
                };
                ret_metadata.fn_type(&param_metadata, ext_func.is_varargs)
            },
            Type::Function(f) => f,
            Type::Void => self.context.void_type().fn_type(&param_metadata, ext_func.is_varargs),
        };

        // Only declare if not already declared
        if self.module.get_function(&ext_func.name).is_none() {
            self.module.add_function(&ext_func.name, function_type, None);
        }
        Ok(())
    }

    /// Declares a function (without compiling its body)
    fn declare_function(&mut self, function: &ast::Function) -> Result<(), CompileError> {
        println!("Declaring function: {}", function.name);
        
        // First, get the return type
        let return_type = self.to_llvm_type(&function.return_type)?;
        println!("  Return type: {:?}", return_type);
        
        // Get parameter basic types with their names
        let param_basic_types: Result<Vec<BasicTypeEnum>, CompileError> = function
            .args
            .iter()
            .map(|(name, t)| {
                println!("  Param: {}: {:?}", name, t);
                self.to_llvm_type(t)
                    .and_then(|lyn_type| {
                        println!("    LLVM type: {:?}", lyn_type);
                        self.expect_basic_type(lyn_type)
                    })
            })
            .collect();
            
        // Convert basic types to metadata types for the function signature
        let param_metadata: Result<Vec<BasicMetadataTypeEnum>, CompileError> = param_basic_types?
            .into_iter()
            .map(|ty| {
                match ty {
                    BasicTypeEnum::ArrayType(t) => Ok(t.into()),
                    BasicTypeEnum::FloatType(t) => Ok(t.into()),
                    BasicTypeEnum::IntType(t) => Ok(t.into()),
                    BasicTypeEnum::PointerType(t) => Ok(t.into()),
                    BasicTypeEnum::StructType(t) => Ok(t.into()),
                    BasicTypeEnum::VectorType(t) => Ok(t.into()),
                    BasicTypeEnum::ScalableVectorType(t) => Ok(t.into()),
                }
            })
            .collect();
            
        let param_metadata = param_metadata?;
        println!("  Param metadata types: {:?}", param_metadata);
        
        // Create the function type with the metadata types
        let function_type = match return_type {
            Type::Basic(b) => {
                let ret_metadata: BasicMetadataTypeEnum = match b {
                    BasicTypeEnum::ArrayType(t) => t.into(),
                    BasicTypeEnum::FloatType(t) => t.into(),
                    BasicTypeEnum::IntType(t) => t.into(),
                    BasicTypeEnum::PointerType(t) => t.into(),
                    BasicTypeEnum::StructType(t) => t.into(),
                    BasicTypeEnum::VectorType(t) => t.into(),
                    BasicTypeEnum::ScalableVectorType(t) => t.into(),
                };
                ret_metadata.fn_type(&param_metadata, false)
            },
            Type::Function(f) => f,
            Type::Void => self.context.void_type().fn_type(&param_metadata, false),
        };
        
        // Only declare if not already declared
        if self.module.get_function(&function.name).is_none() {
            let function_value = self.module.add_function(&function.name, function_type, None);
            println!("  Declared function: {:?}", function_value);
            
            // Set names for all arguments
            for (i, (arg_name, _)) in function.args.iter().enumerate() {
                if let Some(param) = function_value.get_nth_param(i as u32) {
                    param.set_name(arg_name);
                }
            }
            
            // Add to symbol table
            self.symbols.insert(
                function.name.clone(),
                Symbol::Function(function_value),
            );
        }
        Ok(())
    }

    /// Compiles a single function.
    pub fn compile_function(&mut self, function: &ast::Function) -> Result<(), CompileError> {
        let function_value = self.module.get_function(&function.name)
            .ok_or_else(|| CompileError::InternalError("Function not declared".to_string(), None))?;
            
        let entry_block = self.context.append_basic_block(function_value, "entry");
        self.builder.position_at_end(entry_block);
        self.current_function = Some(function_value);

        // Clear variables from previous function by entering a new scope
        self.symbols.enter_scope();

        // Store function parameters in variables
        for (i, (name, type_)) in function.args.iter().enumerate() {
            let param = function_value.get_nth_param(i as u32).unwrap();
            // Get the LLVM type for this parameter
            let llvm_type = self.to_llvm_type(type_)?;
            let basic_type = self.expect_basic_type(llvm_type)?;
            let alloca = self.builder.build_alloca(basic_type, name)?;
            self.builder.build_store(alloca, param)?;
            
            // Use the symbol table to declare the parameter
            self.declare_variable(name, type_.clone(), alloca)?;
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
    fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompileError> {
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
                
                // Use the symbol table to declare the variable
                self.declare_variable(name, type_.clone(), alloca)?;
                
                // If there's an initializer, store the value
                if let Some(init) = initializer {
                    let value = self.compile_expression(init)?;
                    self.builder.build_store(alloca, value)?;
                }
                
                Ok(())
            }
            Statement::VariableAssignment { name, value } => {
                // Look up the variable in the symbol table
                let alloca = self.get_variable(name)?;
                let value = self.compile_expression(value)?;
                self.builder.build_store(alloca, value)?;
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
                if !cond.is_int_value() || cond.into_int_value().get_type().get_bit_width() != 1 {
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
    fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
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
                // Always create a global string constant and return its pointer
                let ptr = self.builder.build_global_string_ptr(val, "str")?;
                let ptr_val = ptr.as_pointer_value();
                
                // If we're in a function context and the expected type is an integer,
                // convert the pointer to an integer
                if let Some(func) = self.current_function {
                    if let Some(return_type) = func.get_type().get_return_type() {
                        if return_type.is_int_type() {
                            let ptr_int = self.builder.build_ptr_to_int(
                                ptr_val,
                                return_type.into_int_type(),
                                "str_to_int"
                            )?;
                            return Ok(ptr_int.into());
                        }
                    }
                }
                
                // Default: return as pointer
                Ok(ptr_val.into())
            },
            Expression::Identifier(name) => {
                let ptr = self.get_variable(name)?;
                let ptr_type = ptr.get_type();
                // Get the type from the pointer
                let elem_type = ptr.get_type();
                
                // Build load with proper error handling
                match self.builder.build_load(elem_type, ptr, name) {
                    Ok(val) => Ok(val.into()),
                    Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
                }
            },
            Expression::BinaryOp { op, left, right } => {
                let left_val = self.compile_expression(left)?;
                let right_val = self.compile_expression(right)?;

                match op {
                    BinaryOperator::Add => {
                        println!("Left value type: {:?}", left_val.get_type());
                        println!("Right value type: {:?}", right_val.get_type());
                        println!("Left is int: {}, right is int: {}", 
                               left_val.is_int_value(), 
                               right_val.is_int_value());
                        println!("Left is float: {}, right is float: {}", 
                               left_val.is_float_value(), 
                               right_val.is_float_value());

                        if left_val.is_int_value() && right_val.is_int_value() {
                            println!("Performing integer addition");
                            let result = self.builder.build_int_add(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "addtmp"
                            )?;
                            Ok(result.into())
                        } else if left_val.is_float_value() && right_val.is_float_value() {
                            println!("Performing float addition");
                            let result = self.builder.build_float_add(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "addtmp"
                            )?;
                            Ok(result.into())
                        } else {
                            println!("Type mismatch in addition");
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
                        // Ensure both operands are string pointers (i8* in LLVM)
                        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                        
                        let left_ptr = if left_val.is_pointer_value() {
                            left_val.into_pointer_value()
                        } else if left_val.is_int_value() {
                            // Convert from integer to pointer
                            let int_val = left_val.into_int_value();
                            self.builder.build_int_to_ptr(
                                int_val,
                                i8_ptr_type,
                                "str_ptr"
                            )?
                        } else {
                            return Err(CompileError::TypeMismatch {
                                expected: "string or string pointer".to_string(),
                                found: left_val.get_type().to_string(),
                                span: None,
                            });
                        };
                        
                        let right_ptr = if right_val.is_pointer_value() {
                            right_val.into_pointer_value()
                        } else if right_val.is_int_value() {
                            // Convert from integer to pointer
                            let int_val = right_val.into_int_value();
                            self.builder.build_int_to_ptr(
                                int_val,
                                i8_ptr_type,
                                "str_ptr"
                            )?
                        } else {
                            return Err(CompileError::TypeMismatch {
                                expected: "string or string pointer".to_string(),
                                found: right_val.get_type().to_string(),
                                span: None,
                            });
                        };
                        
                        // Declare the strcat function if it doesn't exist
                        let strcat_fn = match self.module.get_function("strcat") {
                            Some(f) => f,
                            None => {
                                // Declare strcat: i8* @strcat(i8*, i8*)
                                let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                                let fn_type = self.context.i8_type().fn_type(
                                    &[i8_ptr_type.into(), i8_ptr_type.into()], 
                                    false
                                );
                                self.module.add_function("strcat", fn_type, None)
                            }
                        };
                        
                        // Declare malloc if it doesn't exist
                        let malloc_fn = match self.module.get_function("malloc") {
                            Some(f) => f,
                            None => {
                                // Declare malloc: i8* @malloc(i64)
                                let i64_type = self.context.i64_type();
                                let fn_type = self.context.ptr_type(AddressSpace::default())
                                    .fn_type(&[i64_type.into()], false);
                                self.module.add_function("malloc", fn_type, None)
                            }
                        };
                        
                        // Declare strlen if it doesn't exist
                        let strlen_fn = match self.module.get_function("strlen") {
                            Some(f) => f,
                            None => {
                                // Declare strlen: i64 @strlen(i8*)
                                let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                                let fn_type = self.context.i64_type().fn_type(
                                    &[i8_ptr_type.into()], 
                                    false
                                );
                                self.module.add_function("strlen", fn_type, None)
                            }
                        };
                        
                        // Get lengths of both strings
                        let left_len = {
                            let call = self.builder.build_call(
                                strlen_fn, 
                                &[left_ptr.into()], 
                                "left_len"
                            )?;
                            call.try_as_basic_value().left().ok_or_else(|| 
                                CompileError::InternalError("strlen did not return a value".to_string(), None)
                            )?.into_int_value()
                        };
                        
                        let right_len = {
                            let call = self.builder.build_call(
                                strlen_fn, 
                                &[right_ptr.into()], 
                                "right_len"
                            )?;
                            call.try_as_basic_value().left().ok_or_else(|| 
                                CompileError::InternalError("strlen did not return a value".to_string(), None)
                            )?.into_int_value()
                        };
                        
                        // Calculate total length needed (left + right + 1 for null terminator)
                        let total_len = self.builder.build_int_add(
                            left_len,
                            right_len,
                            "total_len"
                        )?;
                        
                        let one = self.context.i64_type().const_int(1, false);
                        let total_len = self.builder.build_int_add(total_len, one, "total_len_with_null")?;
                        
                        // Allocate memory for the new string
                        let new_str_ptr = {
                            let call = self.builder.build_call(
                                malloc_fn,
                                &[total_len.into()],
                                "new_str"
                            )?;
                            call.try_as_basic_value().left().ok_or_else(|| 
                                CompileError::InternalError("malloc did not return a value".to_string(), None)
                            )?.into_pointer_value()
                        };
                        
                        // Cast the result to i8*
                        let new_str_ptr = self.builder.build_pointer_cast(
                            new_str_ptr,
                            self.context.i8_type().ptr_type(AddressSpace::default()),
                            "new_str_ptr"
                        )?;
                        
                        // Copy first string
                        self.builder.build_store(new_str_ptr, self.context.i8_type().const_int(0, false))?;
                        let _ = self.builder.build_call(
                            strcat_fn,
                            &[new_str_ptr.into(), left_ptr.into()],
                            "concat1"
                        )?;
                        
                        // Concatenate second string
                        let _ = self.builder.build_call(
                            strcat_fn,
                            &[new_str_ptr.into(), right_ptr.into()],
                            "concat2"
                        )?;
                        
                        Ok(new_str_ptr.into())
                    }
                }
            }
            Expression::FunctionCall { name, args } => {
                let function = self.module.get_function(name)
                    .ok_or_else(|| CompileError::UndeclaredFunction(name.clone(), None))?;

                // Get the function type to check parameter types
                let fn_type = function.get_type();
                let param_types: Vec<BasicMetadataValueEnum> = (0..fn_type.get_param_types().len())
                    .filter_map(|i| fn_type.get_param_types().get(i).cloned())
                    .collect();

                // Compile all argument expressions with type information
                let mut compiled_args = Vec::with_capacity(args.len());
                for (i, arg) in args.iter().enumerate() {
                    let val = self.compile_expression(arg)?;
                    
                    // Check if we need to convert the argument type to match the function signature
                    let expected_type = param_types.get(i);
                    let val = if let Some(expected_type) = expected_type {
                        if val.is_int_value() && expected_type.is_pointer_value() {
                            // Convert integer to pointer if needed (e.g., for string literals)
                            let int_val = val.into_int_value();
                            let ptr_type = self.context.ptr_type(AddressSpace::default());
                            self.builder.build_int_to_ptr(
                                int_val,
                                ptr_type,
                                "str_literal_ptr"
                            )?.into()
                        } else {
                            val
                        }
                    } else {
                        val
                    };
                    
                    compiled_args.push(val);
                }

                // Convert arguments to BasicMetadataValueEnum
                let args_metadata: Vec<BasicMetadataValueEnum> = compiled_args
                    .iter()
                    .map(|arg| BasicMetadataValueEnum::try_from(*arg).map_err(|_| 
                        CompileError::InternalError("Failed to convert argument to metadata".to_string(), None)
                    ))
                    .collect::<Result<Vec<_>, _>>()?;

                // Build the call
                let call = self.builder.build_call(
                    function,
                    &args_metadata,
                    "calltmp",
                )?;

                // Handle the return value
                let fn_type = function.get_type();
                let is_void = match fn_type.get_return_type() {
                    None => true,
                    Some(ty) => match ty {
                        BasicTypeEnum::IntType(_) => false,
                        BasicTypeEnum::FloatType(_) => false,
                        BasicTypeEnum::PointerType(_) => false,
                        BasicTypeEnum::StructType(_) => false,
                        BasicTypeEnum::ArrayType(_) => false,
                        BasicTypeEnum::VectorType(_) => false,
                        _ => true, // Treat any other type as void for safety
                    },
                };
                
                if is_void {
                    // For void functions, return a dummy value
                    Ok(self.context.i64_type().const_int(0, false).into())
                } else {
                    match call.try_as_basic_value() {
                        Either::Left(bv) => Ok(bv),
                        Either::Right(_) => Err(CompileError::InternalError(
                            "Function call did not return a value".to_string(),
                            None,
                        )),
                    }
                }
            }
            Expression::Conditional { scrutinee, arms } => {
                let parent_function = self.current_function
                    .ok_or_else(|| CompileError::InternalError("No current function for conditional".to_string(), None))?;

                let cond_val = self.compile_expression(scrutinee)?;
                if !cond_val.is_int_value() || cond_val.into_int_value().get_type().get_bit_width() != 1 {
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
                let ptr = ptr_val.into_pointer_value();
                Ok(self.builder.build_load(ptr.get_type(), ptr, "load_tmp")?.into())
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
                    let ptr_type = base_val.get_type();
                    let offset = offset_val.into_int_value();
                    let ptr = base_val.into_pointer_value();
                    Ok(self.builder.build_gep(ptr_type, ptr, &[self.context.i32_type().const_int(0, false)], "gep_tmp")?.into())
                }
            }
            Expression::StructLiteral { name, fields } => {
                // Get the struct type info and collect all field information
                let (llvm_type, fields_with_info) = {
                    let struct_info = self.struct_types.get(name)
                        .ok_or_else(|| CompileError::TypeError(
                            format!("Undefined struct type: {}", name), 
                            None
                        ))?;
                    
                    // Collect all field information including expressions
                    let mut fields_with_info = Vec::new();
                    for (field_name, field_expr) in fields {
                        let (field_index, field_type) = struct_info.fields.get(field_name)
                            .ok_or_else(|| CompileError::TypeError(
                                format!("No field '{}' in struct '{}'", field_name, name),
                                None
                            ))?;
                        
                        fields_with_info.push((
                            field_name.clone(),
                            *field_index,
                            field_type.clone(),
                            field_expr.clone()
                        ));
                    }
                    
                    // Sort fields by their index to ensure correct order
                    fields_with_info.sort_by_key(|&(_, idx, _, _)| idx);
                    (struct_info.llvm_type, fields_with_info)
                };
                
                // Allocate space for the struct on the stack
                let alloca = self.builder.build_alloca(
                    llvm_type, 
                    &format!("{}_tmp", name)
                )?;
                
                // Store each field
                for (field_name, field_index, field_type, field_expr) in fields_with_info {
                    // Compile the field value
                    let field_val = self.compile_expression(&field_expr)?;
                    
                    // Get a pointer to the field
                    let field_ptr = unsafe {
                        self.builder.build_struct_gep(
                            llvm_type,
                            alloca,
                            field_index as u32,
                            &format!("{}_ptr", field_name)
                        )
                    }?;
                    
                    // Store the field value
                    self.builder.build_store(field_ptr, field_val)?;
                }
                
                // Load the entire struct
                match self.builder.build_load(
                    llvm_type,
                    alloca,
                    &format!("{}_val", name)
                ) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
                }
            }
            Expression::StructField { struct_, field } => {
                // First compile the struct expression to get its value
                let struct_val = self.compile_expression(struct_)?;
                
                // Get the struct type from the value
                let (struct_type, struct_name) = match struct_val.get_type() {
                    BasicTypeEnum::StructType(ty) => {
                        // Find the struct name from our type info
                        let struct_name = self.struct_types.iter()
                            .find(|(_, info)| info.llvm_type == ty)
                            .map(|(name, _)| name.clone())
                            .ok_or_else(|| CompileError::TypeError(
                                format!("Unknown struct type: {:?}", ty),
                                None
                            ))?;
                        (ty, struct_name)
                    },
                    BasicTypeEnum::PointerType(ptr_type) => {
                        // Handle pointer to struct in LLVM 18.1
                        // In LLVM 18.1, we need to use the context to create a new pointer type
                        // and then use that to access the struct fields
                        // First, we'll assume the pointee is a struct and try to find a matching struct type
                        let ptr_type_ref = ptr_type.as_type_ref();
                        
                        // Find a struct type that matches the pointer's element type
                        let pointee_type = self.struct_types.values()
                            .find(|info| {
                                // Create a pointer type for the struct and compare it to our pointer type
                                let struct_ptr_type = self.context.ptr_type(AddressSpace::default());
                                struct_ptr_type.as_type_ref() == ptr_type_ref
                            })
                            .map(|info| info.llvm_type)
                            .ok_or_else(|| CompileError::TypeError(
                                "Expected pointer to struct".to_string(),
                                None
                            ))?;
                        
                        let struct_name = self.struct_types.iter()
                            .find(|(_, info)| info.llvm_type == pointee_type)
                            .map(|(name, _)| name.clone())
                            .ok_or_else(|| CompileError::TypeError(
                                format!("Unknown struct type: {:?}", pointee_type),
                                None
                            ))?;
                        (pointee_type, struct_name)
                    },
                    _ => {
                        return Err(CompileError::TypeMismatch {
                            expected: "struct or struct pointer".to_string(),
                            found: format!("{:?}", struct_val.get_type()),
                            span: None,
                        });
                    }
                };

                // Get the struct type info
                let struct_info = self.struct_types.get(&struct_name)
                    .ok_or_else(|| CompileError::TypeError(
                        format!("Undefined struct type: {}", struct_name),
                        None
                    ))?;

                // Find the field in the struct
                let (field_index, field_type) = struct_info.fields.get(field)
                    .ok_or_else(|| CompileError::TypeError(
                        format!("No field '{}' in struct '{}'", field, struct_name),
                        None
                    ))?;

                // Compile the struct expression to get a value
                let struct_val = self.compile_expression(struct_)?;
                
                // Get a pointer to the struct
                let struct_ptr = if struct_val.is_pointer_value() {
                    // If it's already a pointer, use it directly
                    struct_val.into_pointer_value()
                } else {
                    // If it's a value, store it in an alloca
                    let alloca = self.builder.build_alloca(
                        struct_val.get_type(),
                        "struct_field_tmp"
                    )?;
                    self.builder.build_store(alloca, struct_val)?;
                    alloca
                };
                
                // Get the field index and type before we need to modify self
                let field_index = *field_index;
                let field_type_clone = field_type.clone();
                
                // Get the LLVM type of the field and convert to BasicTypeEnum
                let field_basic_type = match self.to_llvm_type(&field_type_clone)? {
                    Type::Basic(ty) => ty,
                    _ => return Err(CompileError::TypeError(
                        "Expected basic type for struct field".to_string(),
                        None
                    )),
                };
                
                // Get a pointer to the field using struct_gep
                let field_ptr = unsafe {
                    self.builder.build_struct_gep(
                        struct_type,
                        struct_ptr,
                        field_index as u32,
                        &format!("{}_field_{}_ptr", struct_name, field)
                    )
                }.map_err(|e| CompileError::InternalError(
                    format!("Failed to get field '{}' from struct: {}", field, e),
                    None
                ))?;
                
                // Load the field value using the correct type
                match self.builder.build_load(
                    field_basic_type,
                    field_ptr,
                    &format!("{}_val", field)
                ) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
                }
            },
            Expression::StringLength(expr) => {
                // Evaluate the string expression to get a pointer
                let str_val = self.compile_expression(expr)?;
                
                // Ensure it's a pointer type (to i8)
                if !str_val.is_pointer_value() {
                    return Err(CompileError::TypeMismatch {
                        expected: "string (i8*) for length operation".to_string(),
                        found: format!("{:?}", str_val.get_type()),
                        span: None,
                    });
                }
                
                let str_ptr = str_val.into_pointer_value();
                
                // Declare strlen if it doesn't exist
                let strlen_fn = match self.module.get_function("strlen") {
                    Some(f) => f,
                    None => {
                        // Declare strlen: i64 @strlen(i8*)
                        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                        let fn_type = self.context.i64_type().fn_type(
                            &[i8_ptr_type.into()], 
                            false
                        );
                        self.module.add_function("strlen", fn_type, None)
                    }
                };
                
                // Call strlen to get the string length
                let len = {
                    let call = self.builder.build_call(
                        strlen_fn,
                        &[str_ptr.into()],
                        "strlen"
                    )?;
                    
                    // Extract the return value from the call
                    match call.try_as_basic_value() {
                        Either::Left(bv) => {
                            let int_val = bv.into_int_value();
                            // Convert to the appropriate integer type based on the target architecture
                            let target_int_type = self.context.i64_type();
                            if int_val.get_type().get_bit_width() != target_int_type.get_bit_width() {
                                self.builder.build_int_z_extend(int_val, target_int_type, "strlen_ext")?.into()
                            } else {
                                int_val.into()
                            }
                        },
                        Either::Right(_) => return Err(CompileError::InternalError(
                            "strlen did not return a basic value".to_string(),
                            None,
                        )),
                    }
                };
                
                Ok(len)
            },
        }
    }

    /// Converts a Zen type to an LLVM type.
    fn to_llvm_type(&mut self, type_: &AstType) -> Result<Type<'ctx>, CompileError> {
        match type_ {
            AstType::Int8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::Int32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::Int64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::Float => Ok(Type::Basic(self.context.f64_type().into())),
            AstType::Void => Ok(Type::Void),
            AstType::Pointer(inner_type) => {
                if let AstType::Void = **inner_type {
                    return Err(CompileError::UnsupportedFeature(
                        "pointer to void is not supported".to_string(),
                        None,
                    ));
                }
                let inner_llvm_type = self.to_llvm_type(inner_type)?;
                Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into()))
            }
            AstType::String => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())), // Use Context::ptr_type instead of i8_type().ptr_type()
            AstType::Function { args, return_type } => {
                let param_types: Result<Vec<BasicTypeEnum>, CompileError> = args
                    .iter()
                    .map(|ty| {
                        self.to_llvm_type(ty)
                            .and_then(|t| t.into_basic_type())
                    })
                    .collect();
                
                let return_type = self.to_llvm_type(return_type)?;
                let return_type = return_type.into_basic_type()?;
                
                let param_types = param_types?;
                let param_metadata: Vec<BasicMetadataValueEnum> = param_types
                    .into_iter()
                    .map(|ty| {
                        // Convert BasicTypeEnum to BasicValueEnum and then to BasicMetadataValueEnum
                        match ty {
                            BasicTypeEnum::ArrayType(t) => t.const_zero().into(),
                            BasicTypeEnum::FloatType(t) => t.const_float(0.0).into(),
                            BasicTypeEnum::IntType(t) => t.const_zero().into(),
                            BasicTypeEnum::PointerType(t) => t.const_null().into(),
                            BasicTypeEnum::StructType(t) => t.const_zero().into(),
                            BasicTypeEnum::VectorType(t) => t.const_zero().into(),
                            BasicTypeEnum::ScalableVectorType(t) => t.const_zero().into(),
                        }
                    })
                    .collect();

                let fn_type = return_type.fn_type(&param_metadata, false);
                Ok(Type::Function(fn_type))
            }
            AstType::Struct { name, fields } => {
                // Check if we already have this struct type
                if let Some(info) = self.struct_types.get(name) {
                    return Ok(Type::Basic(info.llvm_type.into()));
                }
                
                // Check if the struct type exists in the module but not in our map
                if let Some(struct_type) = self.module.get_struct_type(name) {
                    // Rebuild the field mapping
                    let field_mapping: HashMap<String, (usize, AstType)> = fields
                        .iter()
                        .enumerate()
                        .map(|(i, (name, ty))| (name.clone(), (i, ty.clone())))
                        .collect();
                    
                    // Add to our struct types map
                    let info = StructTypeInfo {
                        llvm_type: struct_type,
                        fields: field_mapping,
                    };
                    
                    // Clone the name to avoid borrowing issues
                    let name_clone = name.clone();
                    self.struct_types.insert(name_clone, info);
                    
                    return Ok(Type::Basic(struct_type.into()));
                }
                
                // Create a new struct type
                let field_types: Result<Vec<BasicTypeEnum>, CompileError> = fields
                    .iter()
                    .map(|(_, ty)| {
                        self.to_llvm_type(ty)
                            .and_then(|lyn_type| self.expect_basic_type(lyn_type))
                    })
                    .collect();
                
                let field_types = field_types?;
                
                // Create field mapping
                let field_mapping: HashMap<String, (usize, AstType)> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, (name, ty))| (name.clone(), (i, ty.clone())))
                    .collect();
                
                // Create the struct type
                let struct_type = self.context.opaque_struct_type(name);
                struct_type.set_body(&field_types, false); // false for not packed
                
                // Store the struct type info
                let info = StructTypeInfo {
                    llvm_type: struct_type,
                    fields: field_mapping,
                };
                
                // Clone the name to avoid borrowing issues
                let name_clone = name.clone();
                self.struct_types.insert(name_clone, info);
                
                Ok(Type::Basic(struct_type.into()))
            }
        }
    }

    /// Expects a basic type, returning an error if not.
    fn expect_basic_type<'a>(&self, t: Type<'a>) -> Result<BasicTypeEnum<'a>, CompileError> {
        match t {
            Type::Basic(b) => Ok(b),
            _ => Err(CompileError::InternalError("Expected basic type".to_string(), None)),
        }
    }
} 