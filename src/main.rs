use inkwell::context::Context;
use inkwell::targets::{Target, TargetMachine};
use inkwell::OptimizationLevel;

use std::path::Path;

mod ast;
mod compiler;
mod error;

use crate::compiler::{Compiler, lexer::Lexer, parser::Parser};
use crate::error::{Result, CompileError};

fn main() -> Result<()> {
    // Initialize LLVM
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())?;
    
    // Create LLVM context and module
    let context = Context::create();
    let module = context.create_module("main");
    
    // Create compiler
    let mut compiler = Compiler::new(&context);
    
    // Zen source code
    let zen_source = "main = () int32 { 42 }";
    println!("Compiling Zen source: {}", zen_source);
    
    // Lex the source
    let lexer = Lexer::new(zen_source);
    
    // Parse the source
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    println!("Parsed program with {} declarations", program.declarations.len());
    
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