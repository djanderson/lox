use std::fs;
use std::io;
use std::io::prelude::*;

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{error::ErrorKind::ValueValidation, CommandFactory, Parser};

/// Lox interpreter from Crafting Interpreters
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Lox file to interpret
    file: Option<Utf8PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if let Some(file) = args.file {
        if !file.exists() {
            Args::command()
                .error(ValueValidation, format!("file {file} does not exist"))
                .exit();
        }
        run_file(&file)
    } else {
        run_repl()
    }
}

fn run_file(file: &Utf8PathBuf) -> Result<()> {
    let input = fs::read_to_string(file)?;
    run(&input)
}

fn run_repl() -> Result<()> {
    loop {
        let Some(line) = readline()? else {
            break;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        run(line)?;
    }
    Ok(())
}

fn run(input: &str) -> Result<()> {
    println!("Running input {:#?}", input);
    Ok(())
}

/// Returns a result with Some(text) or None indicating EOF.
fn readline() -> Result<Option<String>> {
    write!(io::stdout(), "> ")?;
    io::stdout().flush()?;
    let mut buffer = String::new();
    if let Ok(0) = io::stdin().read_line(&mut buffer) {
        return Ok(None);
    }
    Ok(Some(buffer))
}
