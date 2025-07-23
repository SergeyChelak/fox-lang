mod fox;

use crate::fox::{AstPrinter, FoxError, Parser, Scanner, Source};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        2 => {
            let result = run_file(&args[1]);
            if let Err(err) = result {
                handle_error(&err);
            }
        }
        _ => show_usage(),
    }
}

fn handle_error(error: &FoxError) {
    println!("Error occurred {:?}", error);
}

fn run_file<T: AsRef<str>>(path: T) -> Result<(), FoxError> {
    let data = std::fs::read_to_string(path.as_ref())
        .map_err(|_| FoxError::error(fox::ErrorKind::InputOutput))?
        .chars()
        .collect::<Vec<_>>();
    run(&data)
}

fn run(source: &Source) -> Result<(), FoxError> {
    let mut scanner = Scanner::with_source(source);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(&tokens);
    let expr = parser.parse()?;

    let value = AstPrinter.print(&expr)?;
    println!("AST: {}", value);
    Ok(())
}

fn show_usage() {
    println!("Usage: fox-lang <script.fl>");
    // exit(0);
}
