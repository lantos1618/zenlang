pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub mod ast;
pub mod codegen;
pub mod compiler;
pub mod error;
pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
