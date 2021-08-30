use anyhow::{Context, Result};

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

struct Lox {
    had_error: bool,
}

impl Lox {
    fn new() -> Self {
        Self { had_error: false }
    }

    fn run(&mut self, code: &str) {
        let scanner = Scanner::new(&code);
        let tokens = scanner.scan();

        for token in tokens {
            println!("{:?}", token);
        }
    }

    fn run_file(&mut self, filename: &str) -> Result<()> {
        let code = std::fs::read_to_string(filename).context("Could not read code from file")?;
        self.run(&code);

        if self.had_error {
            std::process::exit(65);
        }

        Ok(())
    }

    fn run_prompt(&mut self) -> Result<()> {
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

            self.run(&line);
            self.had_error = false;

            line.clear();
        }

        Ok(())
    }

    fn report(&mut self, line: usize, loc: &str, msg: &str) {
        println!("[line {}] Error{}: {}", line, loc, msg);
        self.had_error = true;
    }
}

fn main() -> Result<()> {
    let mut lox = Lox::new();

    let args: Vec<_> = std::env::args().collect();

    match args.len() {
        2 => lox.run_file(&args[1])?,
        n if n > 2 => {
            eprintln!("Usage: {} [script]", args[0]);
            std::process::exit(64);
        }
        _ => lox.run_prompt()?,
    }

    Ok(())
}
