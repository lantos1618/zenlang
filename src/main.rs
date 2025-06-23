use inkwell::context::Context;
use inkwell::targets::{Target, TargetMachine};
use inkwell::OptimizationLevel;

use std::path::Path;

mod ast;
mod compiler;
mod error;

use crate::compiler::Compiler;
use crate::error::{Result, CompileError};
use crate::ast::{Program, Declaration, Function, Statement, Expression, AstType};

fn main() -> Result<()> {
    // Initialize LLVM
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())?;
    
    // Create LLVM context and module
    let context = Context::create();
    let module = context.create_module("main");
    
    // Create compiler
    let mut compiler = Compiler::new(&context);
    
    // Create a simple test program
    let program = Program {
        declarations: vec![
            Declaration::Function(Function {
                name: "main".to_string(),
                args: vec![], // No arguments for main
                return_type: AstType::Int32,
                body: vec![
                    Statement::Return(Expression::Integer32(42))
                ],
            }),
        ],
    };
    
    // Compile the program
    compiler.compile_program(&program)?;
    
    // Create target machine
    let target = match Target::from_name("x86-64") {
        Some(t) => t,
        None => return Err(CompileError::InternalError("Failed to get x86-64 target".to_string(), None)),
    };
    
    let target_machine = target.create_target_machine(
        &TargetMachine::get_default_triple(),
        "generic",
        "",
        OptimizationLevel::None,
        inkwell::targets::RelocMode::Default,
        inkwell::targets::CodeModel::Default,
    ).ok_or(CompileError::InternalError("Failed to create target machine".to_string(), None))?;
    
    // Emit object file
    let output_path = Path::new("output.o");
    target_machine.write_to_file(&module, inkwell::targets::FileType::Object, output_path)?;
    
    println!("Compilation successful! Output written to output.o");
    Ok(())
} 