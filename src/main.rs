mod fox;

use std::io::{self, Write};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        1 => {
            _ = repl_mode();
        }
        2 => {
            _ = run_file(&args[1]);
        }
        _ => show_usage(),
    }
}

fn repl_mode() -> std::io::Result<()> {
    println!("Fox language REPL. Ctrl+C to terminate");
    let stdin = std::io::stdin();
    loop {
        print!(">");
        std::io::stdout().flush()?;
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        run(&line.as_bytes())
    }
    // Ok(())
}

fn run_file<T: AsRef<str>>(path: T) -> io::Result<()> {
    let data = read_file_as_bytes(path)?;
    run(&data);
    Ok(())
}

fn run(code: &[u8]) {
    todo!()
}

fn show_usage() {
    println!("Usage: fox-lang <script.fl>");
    // exit(0);
}

fn read_file_as_bytes<T: AsRef<str>>(path: T) -> io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path.as_ref())?;
    let mut buffer = Vec::new();
    io::Read::read_to_end(&mut file, &mut buffer)?;
    Ok(buffer)
}
