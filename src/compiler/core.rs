use crate::ast::{self, AstType};
use crate::error::CompileError;
use inkwell::{
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue},
    builder::Builder,
};
use inkwell::{
    types::{
        AsTypeRef, BasicType, BasicTypeEnum, FunctionType, StructType, BasicMetadataTypeEnum
    },
};
use inkwell::AddressSpace;
use std::collections::HashMap;
use super::symbols::{Symbol, SymbolTable};

#[derive(Debug, Clone)]
pub enum Type<'ctx> {
    Basic(BasicTypeEnum<'ctx>),
    Pointer(Box<Type<'ctx>>),
    Struct(StructType<'ctx>),
    Function(FunctionType<'ctx>),
    Void,
}

impl<'ctx> Type<'ctx> {
    pub fn into_basic_type(self) -> Result<BasicTypeEnum<'ctx>, CompileError> {
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
    pub llvm_type: StructType<'ctx>,
    /// Mapping from field name to (index, type)
    pub fields: HashMap<String, (usize, AstType)>,
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
    pub fn get_type(&self, name: &str) -> Result<BasicTypeEnum<'ctx>, CompileError> {
        self.symbols.lookup(name)
            .and_then(|sym| match sym {
                Symbol::Type(ty) => Some(*ty),
                _ => None,
            })
            .ok_or_else(|| CompileError::UndeclaredVariable(name.to_string(), None))
    }

    /// Declares a variable in the current scope
    pub fn declare_variable(&mut self, name: &str, _ty: AstType, ptr: PointerValue<'ctx>) -> Result<(), CompileError> {
        let symbol = Symbol::Variable(ptr);
        if self.symbols.exists_in_current_scope(name) {
            return Err(CompileError::UndeclaredVariable(name.to_string(), None));
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }

    /// Looks up a variable and returns its pointer
    pub fn get_variable(&self, name: &str) -> Result<(PointerValue<'ctx>, AstType), CompileError> {
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
} 