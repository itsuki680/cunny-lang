use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use cunny_lang::{Machine, brat_correction, build_site, compile};

pub fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args().collect();
    let program = arguments.first().map(String::as_str).unwrap_or("cunny");

    match arguments.as_slice() {
        [_, flag] if flag == "--version" || flag == "-V" => {
            println!("cunny {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        [_, flag] if flag == "--help" || flag == "-h" => {
            println!("{}", usage(program));
            Ok(())
        }
        [_, first, second, root] if first == "brat" && second == "correction" => {
            brat_correction(Path::new(root))
        }
        [_, command, root] if command == "build" => build(root),
        [_, flag, path] if flag == "--imouto" => run_file(path, true),
        [_, path] if path != "build" && path != "brat" && path != "--imouto" => {
            run_file(path, false)
        }
        _ => Err(usage(program)),
    }
}

fn run_file(path: &str, imouto: bool) -> Result<(), String> {
    if !path.ends_with(".cunny") {
        return Err("programs must use the .cunny extension".to_owned());
    }

    let source =
        fs::read_to_string(path).map_err(|error| format!("could not read '{path}': {error}"))?;
    let instructions = compile(&source).map_err(|error| {
        if imouto {
            format!("imouto found a problem: {error}")
        } else {
            error
        }
    })?;

    let mut machine = Machine::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();

    if imouto {
        let stderr = io::stderr();
        let mut trace = stderr.lock();
        machine
            .run_with_imouto(&instructions, &mut input, &mut output, &mut trace)
            .map_err(|error| format!("imouto stopped: {error}"))?;
    } else {
        machine.run(&instructions, &mut input, &mut output)?;
    }

    output
        .flush()
        .map_err(|error| format!("could not flush output: {error}"))
}

fn build(root: &str) -> Result<(), String> {
    let report = build_site(Path::new(root))?;
    println!(
        "Built {} page(s) in {}",
        report.pages,
        report.output.display()
    );
    Ok(())
}

fn usage(program: &str) -> String {
    format!(
        "usage:\n  {program} <file.cunny>\n  {program} --imouto <file.cunny>\n  {program} build <site-directory>\n  {program} brat correction <site-directory>\n  {program} --help\n  {program} --version"
    )
}
