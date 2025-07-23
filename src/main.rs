mod fox;

use crate::fox::{FoxError, Scanner, Source};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        2 => {
            _ = run_file(&args[1]);
        }
        _ => show_usage(),
    }
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
    println!("Scanned {} tokens", tokens.len());
    Ok(())
}

fn show_usage() {
    println!("Usage: fox-lang <script.fl>");
    // exit(0);
}
