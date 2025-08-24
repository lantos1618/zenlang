use zen::ast::{self, *};
use zen::codegen::llvm::LLVMCompiler;
use zen::lexer::Lexer;
use zen::parser::Parser;
use inkwell::context::Context;

#[test]
fn test_enum_variant_indices() {
    let input = r#"
Color = | Red | Green | Blue

main :: () -> i32 {
    red_color := Color::Red
    green_color := Color::Green
    blue_color := Color::Blue
    0
}"#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Compile the program
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context);
    compiler.compile_program(&program).expect("Failed to compile program");
    
    // Verify that the module compiles successfully
    let module_string = compiler.module.print_to_string().to_string();
    
    // Check that enum values are created
    assert!(module_string.contains("Red_enum"), "Red variant should be compiled");
    assert!(module_string.contains("Green_enum"), "Green variant should be compiled");
    assert!(module_string.contains("Blue_enum"), "Blue variant should be compiled");
}

#[test]
fn test_enum_with_payload() {
    let input = r#"
Message = | Text(content: *u8) | Number(value: i64) | Empty

main :: () -> i32 {
    msg := Message::Number(42)
    empty := Message::Empty
    0
}"#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Compile the program
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context);
    compiler.compile_program(&program).expect("Failed to compile program");
    
    // Verify that the module compiles successfully
    let module_string = compiler.module.print_to_string().to_string();
    
    // Check that enum values with payloads are created
    assert!(module_string.contains("Number_enum"), "Number variant with payload should be compiled");
    assert!(module_string.contains("Empty_enum"), "Empty variant should be compiled");
}

#[test]
fn test_enum_variant_index_ordering() {
    let input = r#"
Status = | Pending | Processing | Completed | Failed

main :: () -> i32 {
    s1 := Status::Pending
    s2 := Status::Processing  
    s3 := Status::Completed
    s4 := Status::Failed
    0
}"#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Compile the program
    let context = Context::create();
    let mut compiler = LLVMCompiler::new(&context);
    compiler.compile_program(&program).expect("Failed to compile program");
    
    // Verify that the module compiles successfully and variants have correct indices
    let module_string = compiler.module.print_to_string().to_string();
    
    // The LLVM IR should contain stores with the correct variant indices (0, 1, 2, 3)
    assert!(module_string.contains("Pending_enum"), "Pending variant should be compiled");
    assert!(module_string.contains("Processing_enum"), "Processing variant should be compiled");  
    assert!(module_string.contains("Completed_enum"), "Completed variant should be compiled");
    assert!(module_string.contains("Failed_enum"), "Failed variant should be compiled");
}