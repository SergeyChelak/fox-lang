mod error;
mod expression;
mod parser;
mod scanner;
mod token;

pub use error::*;
pub use expression::*;
pub use parser::*;
pub use scanner::*;
pub use token::*;

pub type Source = [char];

pub struct Fox {
    code: Vec<char>,
}

impl Fox {
    pub fn with(code: Vec<char>) -> Self {
        Self { code }
    }

    pub fn run(&self) -> FoxResult<()> {
        let mut scanner = Scanner::with_source(&self.code);
        let tokens = scanner.scan_tokens()?;

        let mut parser = Parser::new(&tokens);
        let expr = parser.parse()?;

        let value = AstPrinter.print(&expr)?;
        println!("AST: {}", value);
        Ok(())
    }

    pub fn error_description(&self, _error: &FoxError) -> String {
        "Some error occurred".to_string()
    }
}
