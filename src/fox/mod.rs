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

pub fn run(source: &Source) -> FoxResult<()> {
    let mut scanner = Scanner::with_source(source);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(&tokens);
    let expr = parser.parse()?;

    let value = AstPrinter.print(&expr)?;
    println!("AST: {}", value);
    Ok(())
}
