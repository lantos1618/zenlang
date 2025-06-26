// This file is intentionally empty to satisfy the Cargo.toml target requirement.
// You can add actual test utilities here as needed. 

use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::types::BasicType;
use zen::ast::{self, AstType, Expression, Statement, BinaryOperator};
use zen::compiler::Compiler;
use zen::compiler::symbols::{SymbolTable, Symbol};
use zen::error::CompileError;
use std::ops::{Deref, DerefMut};

/// A test context that manages the LLVM context, module, and compiler state
/// for running tests. This ensures all compilation and execution use the same
/// module and context.
pub struct TestContext<'ctx> {
    context: &'ctx Context,
    compiler: Compiler<'ctx>,
}

impl<'ctx> TestContext<'ctx> {
    /// Creates a new test context with a fresh module and compiler.
    pub fn new(context: &'ctx Context) -> Self {
        let compiler = Compiler::new(context);
        Self {
            context,
            compiler,
        }
    }

    /// Resets the test context to a clean state.
    /// This clears all compiled code and resets the compiler state.
    pub fn reset(&mut self) {
        // Clear the module by creating a new one with the same context
        // This is safer than creating a new compiler instance which can cause
        // LLVM object lifetime issues
        self.compiler.module = self.context.create_module("main");
        
        // Reset the builder
        self.compiler.builder = self.context.create_builder();
        
        // Clear all state
        self.compiler.variables.clear();
        self.compiler.functions.clear();
        self.compiler.current_function = None;
        
        // Reset symbol table but keep basic types
        self.compiler.symbols = SymbolTable::new();
        
        // Re-add basic types to the symbol table
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let float_type = self.context.f64_type();
        let bool_type = self.context.bool_type();
        
        self.compiler.symbols.insert("i64", Symbol::Type(i64_type.as_basic_type_enum()));
        self.compiler.symbols.insert("i32", Symbol::Type(i32_type.as_basic_type_enum()));
        self.compiler.symbols.insert("f64", Symbol::Type(float_type.as_basic_type_enum()));
        self.compiler.symbols.insert("bool", Symbol::Type(bool_type.as_basic_type_enum()));
    }

    /// Compiles a program into the test context's module.
    pub fn compile(&mut self, program: &ast::Program) -> Result<(), CompileError> {
        // Reset to a clean state before each compilation
        self.reset();
        
        // Compile the program
        self.compiler.compile_program(program)?;
        
        // Verify functions were added to the module
        let func_count = self.compiler.module.get_functions().count();
        println!("After compilation, module has {} functions", func_count);
        
        if func_count == 0 {
            return Err(CompileError::InternalError(
                "No functions were added to the module".to_string(), 
                None
            ));
        }
        
        // Print the module's IR for debugging
        println!(
            "Module IR after compilation:\n{}", 
            self.compiler.module.print_to_string().to_string()
        );
        
        // Verify the module has the expected functions
        let func_names: Vec<_> = self.compiler.module.get_functions()
            .map(|f| f.get_name().to_str().unwrap_or("<invalid>").to_string())
            .collect();
        println!("Functions in module after compilation: {:?}", func_names);
        
        Ok(())
    }

    /// Runs a compiled program and returns its result.
    /// The program must have a 'main' function that returns an i64.
    pub fn run(&self) -> Result<i64, String> {
        // Debug: Print all functions in the module
        println!("Functions in module before execution (count: {}):", self.compiler.module.get_functions().count());
        for func in self.compiler.module.get_functions() {
            println!("  - {}", func.get_name().to_str().unwrap_or("<invalid>"));
        }
        
        // Create a new execution engine with our module
        let execution_engine = self.compiler.module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| format!("Failed to create JIT engine: {}", e))?;
            
        // Verify the module contains the main function
        if self.compiler.module.get_function("main").is_none() {
            return Err(format!("Module does not contain a 'main' function. Available functions: {:?}", 
                self.compiler.module.get_functions()
                    .map(|f| f.get_name().to_str().unwrap_or("<invalid>").to_string())
                    .collect::<Vec<_>>()
            ));
        }

        let jit_function: JitFunction<unsafe extern "C" fn() -> i64> = unsafe {
            execution_engine
                .get_function("main")
                .map_err(|e| format!("Failed to get main function: {}", e))?
        };

        Ok(unsafe { jit_function.call() })
    }

    /// Gets the IR string for the current module.
    pub fn get_ir(&self) -> String {
        self.compiler.module.print_to_string().to_string()
    }

    /// Creates a simple test program that returns a constant value.
    pub fn create_simple_program(value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::i64,
            body: vec![Statement::Return(Expression::Integer64(value))],
        }])
    }

    /// Creates a test program that performs a binary operation.
    pub fn create_binary_op_program(
        left: i64,
        op: BinaryOperator,
        right: i64,
    ) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::i64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Integer64(left)),
                op,
                right: Box::new(Expression::Integer64(right)),
            })],
        }])
    }

    /// Creates a test program that declares and returns a variable.
    pub fn create_variable_program(name: &str, value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::i64,
            body: vec![
                Statement::VariableDeclaration {
                    name: name.to_string(),
                    type_: AstType::i64,
                    initializer: Some(Expression::Integer64(value)),
                },
                Statement::Return(Expression::Identifier(name.to_string())),
            ],
        }])
    }

    /// Creates a test program that calls a function.
    pub fn create_function_call_program(
        func_name: &str,
        args: Vec<Expression>,
        return_type: AstType,
    ) -> ast::Program {
        ast::Program::from_functions(vec![
            ast::Function {
                name: func_name.to_string(),
                args: vec![("arg".to_string(), return_type.clone())],
                return_type: return_type.clone(),
                body: vec![Statement::Return(Expression::Identifier("arg".to_string()))],
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::i64,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: func_name.to_string(),
                    args,
                })],
            },
        ])
    }

    /// Creates a test program that declares and returns a function.
    pub fn create_function_program(name: &str, return_type: AstType) -> ast::Program {
        ast::Program::from_functions(vec![
            ast::Function {
                name: name.to_string(),
                args: vec![("arg".to_string(), return_type.clone())],
                return_type: return_type.clone(),
                body: vec![Statement::Return(Expression::Identifier("arg".to_string()))],
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::i64,
                body: vec![Statement::Return(Expression::Integer64(0))],
            },
        ])
    }
}

impl<'ctx> Deref for TestContext<'ctx> {
    type Target = Compiler<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.compiler
    }
}

impl<'ctx> DerefMut for TestContext<'ctx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.compiler
    }
}

/// Macro to create a test context and run a test
#[macro_export]
macro_rules! test_context {
    (|$test_context:ident: &mut TestContext| $body:block) => {
        let context = Context::create();
        let mut $test_context = TestContext::new(&context);
        $body
    };
} 