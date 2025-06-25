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
    
    // Debug: Check if the module is working
    println!("Module name: {}", compiler.module.get_name().to_str().unwrap_or("<invalid>"));
    println!("Module functions before compilation: {}", compiler.module.get_functions().count());
    
    // Zen source code
    let zen_source = "main = () int32 { x := 42; y := 10; x + y }";
    println!("Compiling Zen source: {}", zen_source);
    
    // The parser bug has been fixed! This should now correctly parse all 3 statements:
    // 1. x := 42 (variable declaration)
    // 2. y := 10 (variable declaration) 
    // 3. x + y (trailing expression as return value)
    
    // Lex the source
    let lexer = Lexer::new(zen_source);
    
    // Parse the source
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    println!("Parsed program with {} declarations", program.declarations.len());
    
    // Compile the program
    println!("Compiling program with {} declarations", program.declarations.len());
    compiler.compile_program(&program)?;
    
    // Print the generated LLVM IR for debugging
    println!("Generated LLVM IR:");
    let ir_string = module.print_to_string().to_string();
    println!("IR string length: {}", ir_string.len());
    println!("IR content: '{}'", ir_string);
    
    // Write IR to a file to see if it's different
    std::fs::write("output.ll", &ir_string).unwrap();
    println!("IR written to output.ll");
    
    // Try printing each function individually
    println!("Individual function IR:");
    for func in module.get_functions() {
        println!("Function: {}", func.get_name().to_str().unwrap_or("<invalid>"));
        // Print the function's LLVM representation
        println!("LLVM value: {}", func.get_name().to_str().unwrap_or("<invalid>"));
    }
    
    // Verify the LLVM IR
    if let Err(errors) = module.verify() {
        println!("LLVM IR verification failed:");
        println!("{}", errors.to_string());
    } else {
        println!("LLVM IR verification passed");
    }
    
    // Also check what functions are in the module
    println!("Functions in module:");
    for func in module.get_functions() {
        println!("  Function: {} (defined: {})", 
            func.get_name().to_str().unwrap_or("<invalid>"),
            func.get_first_basic_block().is_some());
        
        // Print function details
        if let Some(block) = func.get_first_basic_block() {
            println!("    First block: {}", block.get_name().to_str().unwrap_or("<invalid>"));
            println!("    Block terminator: {:?}", block.get_terminator());
        }
    }
    
    // Write object file
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
    
    // Note: The object file can be linked into an executable using:
    // gcc output.o -o zen_program
    // ./zen_program
    
    Ok(())
} 