mod fox;

use std::process::exit;

use crate::fox::Fox;

type ExitCode = i32;
const EXIT_CODE_OK: ExitCode = 0;
const EXIT_CODE_IO_ERROR: ExitCode = 1;
const EXIT_CODE_PROCESSING_ERROR: ExitCode = 2;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        2 => run(&args[1]),
        _ => show_usage(),
    }
    exit(EXIT_CODE_OK);
}

fn run<T: AsRef<str>>(path: T) {
    let Ok(data) = std::fs::read_to_string(path.as_ref()) else {
        exit(EXIT_CODE_IO_ERROR);
    };
    let code = data.chars().collect::<Vec<_>>();
    let fox = Fox::with(code);
    let result = fox.run();
    if let Err(err) = result {
        println!("{}", fox.error_description(&err));
        exit(EXIT_CODE_PROCESSING_ERROR);
    }
}

fn show_usage() {
    println!("Usage: fox-lang <script.fox>");
}
