use std::fs;
use std::io;
use std::io::prelude::*;

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{error::ErrorKind::ValueValidation, CommandFactory, Parser as ArgParser};
use lox::parser::Parser;
use lox::scanner::Scanner;

/// Lox interpreter from Crafting Interpreters
#[derive(ArgParser, Debug)]
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
    run(input)
}

fn run_repl() -> Result<()> {
    loop {
        let Some(line) = readline()? else {
            break;
        };
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        let result = run(line);
        if let Err(e) = result {
            eprintln!("{e}");
        }
        // match result {
        //     Ok(_) => println!(" => TODO"),
        //     Err(e) => ,
        // }
    }
    Ok(())
}

fn run(input: String) -> Result<()> {
    let scanner = Scanner::new(&input);
    let tokens = scanner.tokens()?;
    let mut parser = Parser::new(&tokens);
    let ast = parser.parse()?;
    println!("{}", ast);
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
