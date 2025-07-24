mod error;
mod expression;
mod interpreter;
mod parser;
mod scanner;
mod token;

pub use error::*;
use expression::*;
use parser::*;
use scanner::*;
use token::*;

use crate::fox::interpreter::Interpreter;

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
        println!("AST: {value}");

        let object = Interpreter.evaluate(&expr)?;
        println!("Result: {}", object);

        Ok(())
    }

    pub fn error_description(&self, error: &FoxError) -> String {
        let mut text = format!("{}", error.kind());

        let location = match error.info() {
            ErrorInfo::Empty => None,
            ErrorInfo::Code(location) => Some(location),
            ErrorInfo::Token(token) => Some(&token.code_location),
        };

        if let Some(location) = location {
            let el = ErrorLine::with(&self.code, location);
            text = el.formatted(&text);
        }

        text
    }
}
