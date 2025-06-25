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
            eprintln!("[DEBUG] Storing param '{}' of type {:?} into alloca", name, param.get_type());
            self.builder.build_store(alloca, param)?;
            // Register the parameter in the variables map
            self.variables.insert(name.clone(), (alloca, type_.clone()));
        }

        for statement in &function.body {
            self.compile_statement(statement)?;
        }

        // Check if we need to add a return statement
        if let Some(block) = self.builder.get_insert_block() {
            if block.get_terminator().is_none() {
                // Check if the last statement was an expression that should be returned
                if let Some(last_stmt) = function.body.last() {
                    if let Statement::Expression(expr) = last_stmt {
                        // For non-void functions, treat trailing expressions as return values
                        if !matches!(function.return_type, AstType::Void) {
                            let value = self.compile_expression(expr)?;
                            self.builder.build_return(Some(&value))?;
                        } else {
                            // For void functions, just return void
                            self.builder.build_return(None)?;
                        }
                    } else {
                        // Not a trailing expression, handle normally
                        if let AstType::Void = function.return_type {
                            self.builder.build_return(None)?;
                        } else {
                            return Err(CompileError::MissingReturnStatement(function.name.clone(), None));
                        }
                    }
                } else {
                    // No statements in function body
                    if let AstType::Void = function.return_type {
                        self.builder.build_return(None)?;
                    } else {
                        return Err(CompileError::MissingReturnStatement(function.name.clone(), None));
                    }
                }
            }
        }

        self.current_function = None;
        Ok(())
    }
} 