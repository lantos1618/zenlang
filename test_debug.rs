use zen::lexer::Lexer;
use zen::parser::Parser;

fn main() {
    let input = r#"
        identity<T> = (x: T) T {
            x
        }
        
        main = () i32 {
            a := identity(42);
            b := identity(3.14);
            a
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    println!("Parsed program:");
    for decl in &program.declarations {
        match decl {
            zen::ast::Declaration::Function(func) => {
                println!("Function: {}, type_params: {:?}", func.name, func.type_params);
            }
            _ => {}
        }
    }
}