use zen::lexer::Lexer;
use zen::parser::Parser;

fn main() {
    let input = "Point = { x: i32, y: i32 }";
    println!("Input: {}", input);
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    match parser.parse_program() {
        Ok(program) => println!("Parsed program: {:#?}", program),
        Err(e) => println!("Parse error: {:?}", e),
    }
}