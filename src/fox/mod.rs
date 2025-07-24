mod ast;
mod error;
mod interpreter;
mod parser;
mod scanner;
mod token;

pub use error::*;
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

        // for token in tokens.iter() {
        //     println!("{token:?}");
        // }

        let mut parser = Parser::new(&tokens);
        let statements = parser.parse()?;

        Interpreter.interpret(&statements)
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
