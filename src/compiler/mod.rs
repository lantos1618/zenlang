pub mod symbols;
pub mod lexer;
pub mod parser;

// Re-export the symbols module for easier access
pub use symbols::{Symbol, SymbolTable};

// Rest of the compiler code will go here
// ...
use crate::ast::{self, BinaryOperator, Expression, Statement, AstType as AstType};
use crate::error::CompileError;
use inkwell::{
    context::Context,
    module::Module,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
    builder::Builder,
};
use inkwell::{
    types::{
        AsTypeRef, BasicType, BasicTypeEnum, FunctionType, StructType, BasicMetadataTypeEnum
    },
    IntPredicate, FloatPredicate,
};
use inkwell::values::BasicValue as _;
use inkwell::AddressSpace;
use inkwell::Either;
use std::collections::HashMap;

// Expression codegen modules - now integrated into main Compiler impl
mod expr_codegen;

mod stmt_codegen;
mod types;
mod structs;

#[derive(Debug, Clone)]
pub enum Type<'ctx> {
    Basic(BasicTypeEnum<'ctx>),
    Pointer(Box<Type<'ctx>>),
    Struct(StructType<'ctx>),
    Function(FunctionType<'ctx>),
    Void,
}

impl<'ctx> Type<'ctx> {
    fn into_basic_type(self) -> Result<BasicTypeEnum<'ctx>, CompileError> {
        match self {
            Type::Basic(t) => Ok(t),
            _ => Err(CompileError::TypeMismatch {
                expected: "basic type".to_string(),
                found: format!("{:?}", self),
                span: None,
            }),
        }
    }
}

/// Information about a struct type, including its fields and their indices
#[derive(Debug, Clone)]
pub struct StructTypeInfo<'ctx> {
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
    pub struct_types: HashMap<String, StructTypeInfo<'ctx>>,
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
    fn declare_variable(&mut self, name: &str, _ty: AstType, ptr: PointerValue<'ctx>) -> Result<(), CompileError> {
        let symbol = Symbol::Variable(ptr);
        if self.symbols.exists_in_current_scope(name) {
            return Err(CompileError::UndeclaredVariable(name.to_string(), None));
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }

    /// Looks up a variable and returns its pointer
    fn get_variable(&self, name: &str) -> Result<(PointerValue<'ctx>, AstType), CompileError> {
        if let Some(entry) = self.variables.get(name) {
            return Ok(entry.clone());
        }
        // If not found, check if it's a function in the module
        if let Some(function) = self.module.get_function(name) {
            // Return the function pointer and its type as a function pointer type
            let ptr = function.as_global_value().as_pointer_value();
            // Build the AstType for a function pointer
            // (Assume all functions are of type AstType::Function)
            // You may need to look up the function signature if available
            // For now, use a placeholder type
            let ty = AstType::Pointer(Box::new(AstType::Function {
                args: vec![],
                return_type: Box::new(AstType::Void),
            }));
            return Ok((ptr, ty));
        }
        Err(CompileError::UndeclaredVariable(name.to_string(), None))
    }

    /// Compiles a Zen module.
    /// This is the main entry point for compilation.
    pub fn compile_program(&mut self, program: &ast::Program) -> Result<(), CompileError> {
        println!("Compiling program with {} declarations", program.declarations.len());
        
        // First pass: declare external functions only
        for declaration in &program.declarations {
            match declaration {
                ast::Declaration::ExternalFunction(ext_func) => {
                    println!("Declaring external function: {}", ext_func.name);
                    self.declare_external_function(ext_func)?;
                }
                ast::Declaration::Function(_) => {
                    // Skip function declarations - we'll define them directly
                }
                ast::Declaration::Struct(_) | ast::Declaration::Enum(_) | ast::Declaration::ModuleImport { .. } => {
                    // Skip type definitions and module imports for now
                }
            }
        }
        
        // Second pass: define and compile function bodies
        for declaration in &program.declarations {
            if let ast::Declaration::Function(func) = declaration {
                println!("Defining and compiling function: {}", func.name);
                self.define_and_compile_function(func)?;
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
                Ok(match ty {
                    BasicTypeEnum::ArrayType(t) => t.into(),
                    BasicTypeEnum::FloatType(t) => t.into(),
                    BasicTypeEnum::IntType(t) => t.into(),
                    BasicTypeEnum::PointerType(t) => t.into(),
                    BasicTypeEnum::StructType(t) => t.into(),
                    BasicTypeEnum::VectorType(t) => t.into(),
                    BasicTypeEnum::ScalableVectorType(t) => t.into(),
                })
            })
            .collect();
            
        let param_metadata = param_metadata?;

        // Create the function type with the metadata types
        let function_type = match ret_type {
            Type::Basic(b) => {
                match b {
                    BasicTypeEnum::ArrayType(t) => t.fn_type(&param_metadata, ext_func.is_varargs),
                    BasicTypeEnum::FloatType(t) => t.fn_type(&param_metadata, ext_func.is_varargs),
                    BasicTypeEnum::IntType(t) => t.fn_type(&param_metadata, ext_func.is_varargs),
                    BasicTypeEnum::PointerType(t) => t.fn_type(&param_metadata, ext_func.is_varargs),
                    BasicTypeEnum::StructType(t) => t.fn_type(&param_metadata, ext_func.is_varargs),
                    BasicTypeEnum::VectorType(t) => t.fn_type(&param_metadata, ext_func.is_varargs),
                    BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_metadata, ext_func.is_varargs),
                }
            },
            Type::Function(f) => f,
            Type::Void => self.context.void_type().fn_type(&param_metadata, ext_func.is_varargs),
            Type::Pointer(_) => {
                return Err(CompileError::UnsupportedFeature(
                    "Cannot use pointer type as function return type".to_string(),
                    None,
                ));
            }
            Type::Struct(_) => {
                return Err(CompileError::UnsupportedFeature(
                    "Cannot use struct type as function return type".to_string(),
                    None,
                ));
            }
        };

        // Only declare if not already declared
        if self.module.get_function(&ext_func.name).is_none() {
            self.module.add_function(&ext_func.name, function_type, None);
        }
        Ok(())
    }

    /// Defines and compiles a function in one step
    fn define_and_compile_function(&mut self, function: &ast::Function) -> Result<(), CompileError> {
        println!("DEBUG: Starting to define and compile function: {}", function.name);
        
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
                Ok(match ty {
                    BasicTypeEnum::ArrayType(t) => t.into(),
                    BasicTypeEnum::FloatType(t) => t.into(),
                    BasicTypeEnum::IntType(t) => t.into(),
                    BasicTypeEnum::PointerType(t) => t.into(),
                    BasicTypeEnum::StructType(t) => t.into(),
                    BasicTypeEnum::VectorType(t) => t.into(),
                    BasicTypeEnum::ScalableVectorType(t) => t.into(),
                })
            })
            .collect();
            
        let param_metadata = param_metadata?;
        println!("  Param metadata types: {:?}", param_metadata);
        
        // Create the function type with the metadata types
        let function_type = match return_type {
            Type::Basic(b) => {
                match b {
                    BasicTypeEnum::ArrayType(t) => t.fn_type(&param_metadata, false),
                    BasicTypeEnum::FloatType(t) => t.fn_type(&param_metadata, false),
                    BasicTypeEnum::IntType(t) => t.fn_type(&param_metadata, false),
                    BasicTypeEnum::PointerType(t) => t.fn_type(&param_metadata, false),
                    BasicTypeEnum::StructType(t) => t.fn_type(&param_metadata, false),
                    BasicTypeEnum::VectorType(t) => t.fn_type(&param_metadata, false),
                    BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_metadata, false),
                }
            },
            Type::Function(f) => f,
            Type::Void => self.context.void_type().fn_type(&param_metadata, false),
            Type::Pointer(_) => {
                return Err(CompileError::UnsupportedFeature(
                    "Cannot use pointer type as function return type".to_string(),
                    None,
                ));
            }
            Type::Struct(_) => {
                return Err(CompileError::UnsupportedFeature(
                    "Cannot use struct type as function return type".to_string(),
                    None,
                ));
            }
        };
        
        // Define the function (this creates a definition, not a declaration)
        let function_value = self.module.add_function(&function.name, function_type, None);
        println!("  Defined function: {:?}", function_value);
        
        // Set the function linkage to external so it can be linked
        function_value.set_linkage(inkwell::module::Linkage::External);
        
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
        
        // Now compile the function body
        let entry_block = self.context.append_basic_block(function_value, "entry");
        self.builder.position_at_end(entry_block);
        self.current_function = Some(function_value);
        println!("DEBUG: Created entry block and positioned builder");

        // Clear variables from previous function by entering a new scope
        self.symbols.enter_scope();

        // Store function parameters in variables
        for (i, (name, type_)) in function.args.iter().enumerate() {
            let param = function_value.get_nth_param(i as u32).unwrap();
            // Get the LLVM type for this parameter
            let llvm_type = self.to_llvm_type(type_)?;
            let basic_type = self.expect_basic_type(llvm_type)?;
            let alloca = self.builder.build_alloca(basic_type, name)?;
            eprintln!("[DEBUG] Storing param '{}' of type {:?} into alloca", name, param.get_type());
            self.builder.build_store(alloca, param)?;
            // Register the parameter in the variables map
            self.variables.insert(name.clone(), (alloca, type_.clone()));
        }

        println!("DEBUG: Compiling {} statements in function body", function.body.len());
        for (i, statement) in function.body.iter().enumerate() {
            println!("DEBUG: Compiling statement {}: {:?}", i, statement);
            self.compile_statement(statement)?;
        }

        // Check if we need to add a return statement
        if let Some(block) = self.builder.get_insert_block() {
            if block.get_terminator().is_none() {
                println!("DEBUG: No terminator found, adding return statement");
                // Check if the last statement was an expression that should be returned
                if let Some(last_stmt) = function.body.last() {
                    if let Statement::Expression(expr) = last_stmt {
                        // For non-void functions, treat trailing expressions as return values
                        if !matches!(function.return_type, AstType::Void) {
                            println!("DEBUG: Compiling trailing expression as return value");
                            let value = self.compile_expression(expr)?;
                            self.builder.build_return(Some(&value))?;
                            println!("DEBUG: Added return statement with value");
                        } else {
                            // For void functions, just return void
                            self.builder.build_return(None)?;
                            println!("DEBUG: Added void return statement");
                        }
                    } else {
                        // Not a trailing expression, handle normally
                        if let AstType::Void = function.return_type {
                            self.builder.build_return(None)?;
                            println!("DEBUG: Added void return statement (no trailing expr)");
                        } else {
                            return Err(CompileError::MissingReturnStatement(function.name.clone(), None));
                        }
                    }
                } else {
                    // No statements in function body
                    if let AstType::Void = function.return_type {
                        self.builder.build_return(None)?;
                        println!("DEBUG: Added void return statement (empty function)");
                    } else {
                        return Err(CompileError::MissingReturnStatement(function.name.clone(), None));
                    }
                }
            } else {
                println!("DEBUG: Block already has terminator");
            }
        } else {
            println!("DEBUG: No insert block found");
        }

        self.current_function = None;
        println!("DEBUG: Finished defining and compiling function: {}", function.name);
        Ok(())
    }

    // Expression compilation methods
    pub fn compile_integer_literal(&self, value: i64) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self.context.i64_type().const_int(value as u64, false).into())
    }

    pub fn compile_float_literal(&self, value: f64) -> Result<BasicValueEnum<'ctx>, CompileError> {
        Ok(self.context.f64_type().const_float(value).into())
    }

    pub fn compile_string_literal(&mut self, val: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let ptr = self.builder.build_global_string_ptr(val, "str")?;
        let ptr_val = ptr.as_pointer_value();
        // Always return the pointer value, don't convert to integer
        // This fixes the issue where string literals were being converted to integers
        // when used as function arguments, breaking string operations
        Ok(ptr_val.into())
    }

    pub fn compile_identifier(&mut self, name: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // First check if this is a function name
        if let Some(function) = self.module.get_function(name) {
            // Return the function's address as a pointer value
            Ok(function.as_global_value().as_pointer_value().into())
        } else {
            // It's a variable, load it normally
            let (ptr, ast_type) = self.get_variable(name)?;
            let loaded: BasicValueEnum = match &ast_type {
                AstType::Pointer(inner) if matches!(**inner, AstType::Function { .. }) => {
                    match self.builder.build_load(self.context.ptr_type(AddressSpace::default()), ptr, name) {
                        Ok(val) => val.into(),
                        Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                    }
                }
                AstType::Function { .. } => {
                    match self.builder.build_load(self.context.ptr_type(AddressSpace::default()), ptr, name) {
                        Ok(val) => val.into(),
                        Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                    }
                }
                _ => {
                    let elem_type = self.to_llvm_type(&ast_type)?;
                    let basic_type = self.expect_basic_type(elem_type)?;
                    match self.builder.build_load(basic_type, ptr, name) {
                        Ok(val) => val.into(),
                        Err(e) => return Err(CompileError::InternalError(e.to_string(), None)),
                    }
                }
            };
            Ok(loaded)
        }
    }

    pub fn compile_binary_operation(
        &mut self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let left_val = self.compile_expression(left)?;
        let right_val = self.compile_expression(right)?;

        match op {
            BinaryOperator::Add => self.compile_add(left_val, right_val),
            BinaryOperator::Subtract => self.compile_subtract(left_val, right_val),
            BinaryOperator::Multiply => self.compile_multiply(left_val, right_val),
            BinaryOperator::Divide => self.compile_divide(left_val, right_val),
            BinaryOperator::Equals => self.compile_equals(left_val, right_val),
            BinaryOperator::NotEquals => self.compile_not_equals(left_val, right_val),
            BinaryOperator::LessThan => self.compile_less_than(left_val, right_val),
            BinaryOperator::GreaterThan => self.compile_greater_than(left_val, right_val),
            BinaryOperator::LessThanEquals => self.compile_less_than_equals(left_val, right_val),
            BinaryOperator::GreaterThanEquals => self.compile_greater_than_equals(left_val, right_val),
            BinaryOperator::StringConcat => self.compile_string_concat(left_val, right_val),
            BinaryOperator::Modulo | BinaryOperator::And | BinaryOperator::Or => {
                todo!("Modulo, And, Or operators not implemented yet")
            }
        }
    }

    fn compile_add(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_subtract(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_multiply(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_divide(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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
        } else if left_val.is_pointer_value() && right_val.is_pointer_value() {
            // String comparison: call strcmp and check for zero
            let strcmp_fn = match self.module.get_function("strcmp") {
                Some(f) => f,
                None => {
                    let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                    let fn_type = self.context.i32_type().fn_type(
                        &[i8_ptr_type.into(), i8_ptr_type.into()],
                        false
                    );
                    self.module.add_function("strcmp", fn_type, None)
                }
            };
            let left_ptr = left_val.into_pointer_value();
            let right_ptr = right_val.into_pointer_value();
            let call = self.builder.build_call(
                strcmp_fn,
                &[left_ptr.into(), right_ptr.into()],
                "strcmp_call"
            )?;
            let cmp_result = call.try_as_basic_value().left().ok_or_else(||
                CompileError::InternalError("strcmp did not return a value".to_string(), None)
            )?.into_int_value();
            let zero = self.context.i32_type().const_int(0, false);
            let result = self.builder.build_int_compare(
                IntPredicate::EQ,
                cmp_result,
                zero,
                "strcmp_eq"
            )?;
            // Zero-extend i1 to i64 for test compatibility
            let zext = self.builder.build_int_z_extend(result, self.context.i64_type(), "zext_strcmp_eq")?;
            Ok(zext.into())
        } else {
            Err(CompileError::TypeMismatch {
                expected: "int or float or string".to_string(),
                found: "mixed types".to_string(),
                span: None,
            })
        }
    }

    fn compile_not_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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
        } else if left_val.is_pointer_value() && right_val.is_pointer_value() {
            // String comparison: call strcmp and check for nonzero
            let strcmp_fn = match self.module.get_function("strcmp") {
                Some(f) => f,
                None => {
                    let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                    let fn_type = self.context.i32_type().fn_type(
                        &[i8_ptr_type.into(), i8_ptr_type.into()],
                        false
                    );
                    self.module.add_function("strcmp", fn_type, None)
                }
            };
            let left_ptr = left_val.into_pointer_value();
            let right_ptr = right_val.into_pointer_value();
            let call = self.builder.build_call(
                strcmp_fn,
                &[left_ptr.into(), right_ptr.into()],
                "strcmp_call"
            )?;
            let cmp_result = call.try_as_basic_value().left().ok_or_else(||
                CompileError::InternalError("strcmp did not return a value".to_string(), None)
            )?.into_int_value();
            let zero = self.context.i32_type().const_int(0, false);
            let result = self.builder.build_int_compare(
                IntPredicate::NE,
                cmp_result,
                zero,
                "strcmp_ne"
            )?;
            // Zero-extend i1 to i64 for test compatibility
            let zext = self.builder.build_int_z_extend(result, self.context.i64_type(), "zext_strcmp_ne")?;
            Ok(zext.into())
        } else {
            Err(CompileError::TypeMismatch {
                expected: "int or float or string".to_string(),
                found: "mixed types".to_string(),
                span: None,
            })
        }
    }

    fn compile_less_than(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_greater_than(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_less_than_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_greater_than_equals(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    fn compile_string_concat(
        &mut self,
        left_val: BasicValueEnum<'ctx>,
        right_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
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
            self.context.ptr_type(AddressSpace::default()),
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

    pub fn compile_function_call(&mut self, name: &str, args: &[Expression]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // First check if this is a direct function call
        if let Some(function) = self.module.get_function(name) {
            // Direct function call
            let mut compiled_args = Vec::with_capacity(args.len());
            for arg in args {
                let val = self.compile_expression(arg)?;
                compiled_args.push(val);
            }
            let args_metadata: Vec<BasicMetadataValueEnum> = compiled_args.iter()
                .map(|arg| BasicMetadataValueEnum::try_from(*arg).map_err(|_| 
                    CompileError::InternalError("Failed to convert argument to metadata".to_string(), None)
                ))
                .collect::<Result<Vec<_>, _>>()?;
            let call = self.builder.build_call(function, &args_metadata, "calltmp")?;
            Ok(call.try_as_basic_value().left().ok_or_else(||
                CompileError::InternalError("Function call did not return a value".to_string(), None)
            )?)
        } else if let Ok((alloca, var_type)) = self.get_variable(name) {
            // Function pointer call - load the function pointer from variable
            let function_ptr = self.builder.build_load(
                alloca.get_type(),
                alloca,
                "func_ptr"
            )?;
            
            // Get function type from the variable type
            let function_type = match &var_type {
                AstType::Function { args, return_type } => {
                    let param_types: Result<Vec<BasicTypeEnum>, CompileError> = args
                        .iter()
                        .map(|ty| {
                            let llvm_ty = self.to_llvm_type(ty)?;
                            match llvm_ty {
                                Type::Basic(b) => Ok(b),
                                _ => Err(CompileError::InternalError("Function argument type must be a basic type".to_string(), None)),
                            }
                        })
                        .collect();
                    let param_types = param_types?;
                    let param_metadata: Vec<BasicMetadataTypeEnum> = param_types.iter().map(|ty| (*ty).into()).collect();
                    let ret_type = self.to_llvm_type(return_type)?;
                    match ret_type {
                        Type::Basic(b) => b.fn_type(&param_metadata, false),
                        Type::Void => self.context.void_type().fn_type(&param_metadata, false),
                        _ => return Err(CompileError::InternalError("Function return type must be a basic type or void".to_string(), None)),
                    }
                },
                AstType::Pointer(inner) if matches!(**inner, AstType::Function { .. }) => {
                    let inner_llvm_type = self.to_llvm_type(inner)?;
                    match inner_llvm_type {
                        Type::Basic(BasicTypeEnum::PointerType(_ptr_type)) => {
                            // For function pointers, we need to get the function type
                            // Since we can't get it directly from the pointer type in newer LLVM,
                            // we'll create a function type based on the AST type
                            if let AstType::Function { args, return_type } = &**inner {
                                let param_types: Result<Vec<BasicTypeEnum>, CompileError> = args
                                    .iter()
                                    .map(|ty| {
                                        let llvm_ty = self.to_llvm_type(ty)?;
                                        match llvm_ty {
                                            Type::Basic(b) => Ok(b),
                                            _ => Err(CompileError::InternalError("Function argument type must be a basic type".to_string(), None)),
                                        }
                                    })
                                    .collect();
                                let param_types = param_types?;
                                let param_metadata: Vec<BasicMetadataTypeEnum> = param_types.iter().map(|ty| (*ty).into()).collect();
                                let ret_type = self.to_llvm_type(return_type)?;
                                match ret_type {
                                    Type::Basic(b) => b.fn_type(&param_metadata, false),
                                    Type::Void => self.context.void_type().fn_type(&param_metadata, false),
                                    _ => return Err(CompileError::InternalError("Function return type must be a basic type or void".to_string(), None)),
                                }
                            } else {
                                return Err(CompileError::InternalError("Expected function type in pointer".to_string(), None));
                            }
                        },
                        _ => return Err(CompileError::TypeMismatch {
                            expected: "function pointer".to_string(),
                            found: format!("{:?}", inner_llvm_type),
                            span: None,
                        }),
                    }
                },
                _ => return Err(CompileError::TypeMismatch {
                    expected: "function pointer".to_string(),
                    found: format!("{:?}", var_type),
                    span: None,
                }),
            };
            
            // Compile arguments
            let mut compiled_args = Vec::with_capacity(args.len());
            for arg in args {
                let val = self.compile_expression(arg)?;
                compiled_args.push(val);
            }
            let args_metadata: Vec<BasicMetadataValueEnum> = compiled_args.iter()
                .map(|arg| BasicMetadataValueEnum::try_from(*arg).map_err(|_| 
                    CompileError::InternalError("Failed to convert argument to metadata".to_string(), None)
                ))
                .collect::<Result<Vec<_>, _>>()?;
            
            // Cast the loaded pointer to the correct function type
            let casted_function_ptr = self.builder.build_pointer_cast(
                function_ptr.into_pointer_value(),
                self.context.ptr_type(AddressSpace::default()),
                "casted_func_ptr"
            )?;
            
            // Make indirect call using build_indirect_call for function pointers
            let call = self.builder.build_indirect_call(
                function_type,
                casted_function_ptr,
                &args_metadata,
                "indirect_call"
            )?;
            Ok(call.try_as_basic_value().left().ok_or_else(||
                CompileError::InternalError("Function call did not return a value".to_string(), None)
            )?)
        } else {
            // Function not found
            Err(CompileError::UndeclaredFunction(name.to_string(), None))
        }
    }

    pub fn compile_conditional(&mut self, scrutinee: &Expression, arms: &[(Expression, Expression)]) -> Result<BasicValueEnum<'ctx>, CompileError> {
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

    pub fn compile_address_of(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
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

    pub fn compile_dereference(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let ptr_val = self.compile_expression(expr)?;
        if !ptr_val.is_pointer_value() {
            return Err(CompileError::TypeMismatch {
                expected: "pointer".to_string(),
                found: format!("{:?}", ptr_val.get_type()),
                span: None,
            });
        }
        let ptr = ptr_val.into_pointer_value();
        
        // Try to determine the element type from the pointer
        // For struct pointers, we need to find the struct type
        let element_type: BasicTypeEnum = if let BasicTypeEnum::PointerType(_) = ptr_val.get_type() {
            // Check if this is a pointer to a struct
            let struct_name = self.struct_types.iter()
                .find(|(_, _info)| {
                    let struct_ptr_type = self.context.ptr_type(AddressSpace::default());
                    struct_ptr_type.as_type_ref() == ptr_val.get_type().as_type_ref()
                })
                .map(|(name, _)| name.clone());
            
            if let Some(name) = struct_name {
                let struct_info = self.struct_types.get(&name)
                    .ok_or_else(|| CompileError::TypeError(
                        format!("Undefined struct type: {}", name),
                        None
                    ))?;
                struct_info.llvm_type.as_basic_type_enum()
            } else {
                self.context.i64_type().as_basic_type_enum()
            }
        } else {
            self.context.i64_type().as_basic_type_enum()
        };
        
        Ok(self.builder.build_load(element_type, ptr, "load_tmp")?.into())
    }

    pub fn compile_pointer_offset(&mut self, pointer: &Expression, offset: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
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
            let _offset = offset_val.into_int_value();
            let ptr = base_val.into_pointer_value();
            Ok(self.builder.build_gep(ptr_type, ptr, &[self.context.i32_type().const_int(0, false)], "gep_tmp")?.into())
        }
    }

    pub fn compile_struct_literal(&mut self, name: &str, fields: &[(String, Expression)]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let (llvm_type, fields_with_info) = {
            let struct_info = self.struct_types.get(name)
                .ok_or_else(|| CompileError::TypeError(
                    format!("Undefined struct type: {}", name), 
                    None
                ))?;
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
            fields_with_info.sort_by_key(|&(_, idx, _, _)| idx);
            (struct_info.llvm_type, fields_with_info)
        };
        let alloca = self.builder.build_alloca(
            llvm_type, 
            &format!("{}_tmp", name)
        )?;
        for (field_name, field_index, _field_type, field_expr) in fields_with_info {
            let field_val = self.compile_expression(&field_expr)?;
            let field_ptr = self.builder.build_struct_gep(
                llvm_type,
                alloca,
                field_index as u32,
                &format!("{}_ptr", field_name)
            )?;
            self.builder.build_store(field_ptr, field_val)?;
        }
        match self.builder.build_load(
            llvm_type,
            alloca,
            &format!("{}_val", name)
        ) {
            Ok(val) => Ok(val),
            Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
        }
    }

    pub fn compile_struct_field(&mut self, struct_: &Expression, field: &str) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let struct_val = self.compile_expression(struct_)?;
        let (struct_type, struct_name) = match struct_val.get_type() {
            BasicTypeEnum::StructType(ty) => {
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
                // This is a pointer to a struct, we need to find the struct type
                // First, try to find a struct type that matches this pointer type
                let struct_name = self.struct_types.iter()
                    .find(|(_, _info)| {
                        // Check if this struct's pointer type matches our pointer type
                        let struct_ptr_type = self.context.ptr_type(AddressSpace::default());
                        struct_ptr_type.as_type_ref() == ptr_type.as_type_ref()
                    })
                    .map(|(name, _)| name.clone());
                
                if let Some(name) = struct_name {
                    let struct_info = self.struct_types.get(&name)
                        .ok_or_else(|| CompileError::TypeError(
                            format!("Undefined struct type: {}", name),
                            None
                        ))?;
                    (struct_info.llvm_type, name)
                } else {
                    // If we can't find a direct match, assume it's a pointer to a struct
                    // and try to find any struct type (this is a fallback)
                    let first_struct = self.struct_types.iter().next()
                        .ok_or_else(|| CompileError::TypeError(
                            "No struct types defined".to_string(),
                            None
                        ))?;
                    (first_struct.1.llvm_type, first_struct.0.clone())
                }
            },
            _ => {
                return Err(CompileError::TypeMismatch {
                    expected: "struct or struct pointer".to_string(),
                    found: format!("{:?}", struct_val.get_type()),
                    span: None,
                });
            }
        };
        
        // Get field information
        let (field_index, field_type) = {
            let struct_info = self.struct_types.get(&struct_name)
                .ok_or_else(|| CompileError::TypeError(
                    format!("Undefined struct type: {}", struct_name),
                    None
                ))?;
            let (field_index, field_type) = struct_info.fields.get(field)
                .ok_or_else(|| CompileError::TypeError(
                    format!("No field '{}' in struct '{}'", field, struct_name),
                    None
                ))?;
            (*field_index, field_type.clone())
        };
        
        // Handle the struct value - if it's a pointer, use it directly; otherwise, create a temporary
        let struct_ptr = if struct_val.is_pointer_value() {
            struct_val.into_pointer_value()
        } else {
            let alloca = self.builder.build_alloca(
                struct_val.get_type(),
                "struct_field_tmp"
            )?;
            self.builder.build_store(alloca, struct_val)?;
            alloca
        };
        
        let field_basic_type = match self.to_llvm_type(&field_type)? {
            Type::Basic(ty) => ty,
            _ => return Err(CompileError::TypeError(
                "Expected basic type for struct field".to_string(),
                None
            )),
        };
        
        let field_ptr = self.builder.build_struct_gep(
            struct_type,
            struct_ptr,
            field_index as u32,
            &format!("{}_field_{}_ptr", struct_name, field)
        )?;
        
        match self.builder.build_load(
            field_basic_type,
            field_ptr,
            &format!("{}_val", field)
        ) {
            Ok(val) => Ok(val),
            Err(e) => Err(CompileError::InternalError(e.to_string(), None)),
        }
    }

    pub fn compile_string_length(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let str_val = self.compile_expression(expr)?;
        if !str_val.is_pointer_value() {
            return Err(CompileError::TypeMismatch {
                expected: "string (i8*) for length operation".to_string(),
                found: format!("{:?}", str_val.get_type()),
                span: None,
            });
        }
        let str_ptr = str_val.into_pointer_value();
        let strlen_fn = match self.module.get_function("strlen") {
            Some(f) => f,
            None => {
                let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = self.context.i64_type().fn_type(
                    &[i8_ptr_type.into()], 
                    false
                );
                self.module.add_function("strlen", fn_type, None)
            }
        };
        let len = {
            let call = self.builder.build_call(
                strlen_fn,
                &[str_ptr.into()],
                "strlen"
            )?;
            match call.try_as_basic_value() {
                Either::Left(bv) => {
                    let int_val = bv.into_int_value();
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
    }
} 