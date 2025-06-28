use inkwell::execution_engine::JitFunction;
use inkwell::OptimizationLevel;
use zen::{
    ast::{self, AstType, Expression, Statement, VariableDeclarationType},
    compiler::Compiler,
    error::CompileError,
};
use std::ops::{Deref, DerefMut};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::execution_engine::{ExecutionEngine};

/// A test context that manages the LLVM context and compiler state
/// for running tests. This ensures all compilation and execution use the same
/// context.
pub struct TestContext<'ctx> {
    context: &'ctx Context,
    compiler: Compiler<'ctx>,
    compiled_ir: Option<String>,
}

impl<'ctx> TestContext<'ctx> {
    /// Creates a new test context with a fresh compiler.
    pub fn new(context: &'ctx Context) -> Self {
        let compiler = Compiler::new(context);
        Self {
            context,
            compiler,
            compiled_ir: None,
        }
    }

    /// Resets the test context to a clean state.
    /// This clears all compiled code and resets the compiler state.
    pub fn reset(&mut self) {
        // Create a new compiler instance to ensure clean state
        // This is safer than trying to clear individual fields, especially private ones
        self.compiler = Compiler::new(self.context);
        self.compiled_ir = None;
    }

    /// Compiles a program into the test context's module.
    pub fn compile(&mut self, program: &ast::Program) -> Result<(), CompileError> {
        // Reset to a clean state before each compilation
        self.reset();
        
        // Compile the program using the new API
        let ir = self.compiler.compile_llvm(program)?;
        self.compiled_ir = Some(ir);
        
        // Print the module's IR for debugging
        println!(
            "Module IR after compilation:\n{}", 
            self.compiled_ir.as_ref().unwrap()
        );
        
        Ok(())
    }

    /// Runs a compiled program and returns its result.
    /// The program must have a 'main' function that returns an i64.
    pub fn run(&self) -> Result<i64, String> {
        // For now, we'll need to recompile to get access to the module for execution
        // This is a limitation of the new architecture - we need to store the LLVM compiler
        // or provide a way to execute the compiled IR
        Err("Execution not yet implemented in new architecture".to_string())
    }

    /// Gets the IR string for the current module.
    pub fn get_ir(&self) -> String {
        self.compiled_ir.clone().unwrap_or_else(|| "No IR available".to_string())
    }

    /// Creates a simple test program that returns a constant value.
    pub fn create_simple_program(value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Integer64(value))],
            is_async: false,
        }])
    }

    /// Creates a test program that performs a binary operation.
    pub fn create_binary_op_program(
        left: i64,
        op: ast::BinaryOperator,
        right: i64,
    ) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Integer64(left)),
                op,
                right: Box::new(Expression::Integer64(right)),
            })],
            is_async: false,
        }])
    }

    /// Creates a test program that declares and returns a variable.
    pub fn create_variable_program(name: &str, value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration {
                    name: name.to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(value)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Identifier(name.to_string())),
            ],
            is_async: false,
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
                is_async: false,
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: func_name.to_string(),
                    args,
                })],
                is_async: false,
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
                is_async: false,
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::Integer64(0))],
                is_async: false,
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

/// Helper macro to create a test context and run a test.
#[macro_export]
macro_rules! test_context {
    ($body:expr) => {{
        let context = inkwell::context::Context::create();
        let mut test_context = $crate::test_utils::TestContext::new(&context);
        $body(&mut test_context)
    }};
} 