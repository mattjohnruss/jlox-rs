use anyhow::{Context, Result};
use std::process::exit;

use std::io;
use std::io::prelude::*;

#[derive(Debug)]
struct Scanner;

impl Scanner {
    fn new(_code: &str) -> Self {
        Self
    }

    fn scan(&self) -> Vec<Token> {
        vec![]
    }
}

#[derive(Debug)]
struct Token;

fn run(code: &str) {
    let scanner = Scanner::new(&code);
    let tokens = scanner.scan();

    for token in tokens {
        println!("{:?}", token);
    }
}

fn run_file(filename: &str) -> Result<()> {
    let code = std::fs::read_to_string(filename).context("Could read code from file")?;
    run(&code);

    Ok(())
}

fn run_prompt() -> Result<()> {
    let mut line = String::new();
    let input = io::stdin();

    loop {
        print!("> ");
        // annoying but apparently we have to manually flush stdout to get the prompt to reliably
        // appear here
        io::stdout().flush()?;

        input.read_line(&mut line)?;

        if line.len() == 0 {
            break;
        }
        run(&line);
        line.clear();
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();

    match args.len() {
        2 => run_file(&args[1])?,
        n if n > 2 => {
            eprintln!("Usage: {} [script]", args[0]);
            exit(64);
        }
        _ => run_prompt()?,
    }

    Ok(())
}
