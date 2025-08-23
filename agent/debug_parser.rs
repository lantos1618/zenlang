use zen::lexer::Lexer;
use zen::parser::Parser;

fn main() {
    let input = "loop counter > 0 { counter = counter - 1 }";
    println!("Input: {}", input);
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    // Try to parse just the loop statement
    parser.next_token(); // Move to first token
    
    println!("Current token: {:?}", parser.current_token);
    
    match parser.parse_statement() {
        Ok(stmt) => println!("Parsed statement: {:#?}", stmt),
        Err(e) => println!("Parse error: {:?}", e),
    }
}