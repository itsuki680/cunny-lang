mod cli;

use std::process;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}
