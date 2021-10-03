#![allow(unused)]

use anyhow::{Context, Result};

use std::any::Any;
use std::io;
use std::io::prelude::*;

#[derive(Debug)]
enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier,
    String,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(Debug)]
struct Scanner<'source> {
    source: &'source str,
    //tokens: Vec<Token>,
}

impl<'source> Scanner<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            //tokens: vec![],
        }
    }

    fn scan(&self) -> Vec<Token> {
        let tokens = vec![];

        let mut char_iter = self.source.chars().peekable();

        while let Some(c) = char_iter.next() {
            match char_iter.peek() {
                Some(c_next) => {
                    println!("{}, {}", c, c_next);
                }
                None => println!("{}", c),
            }
        }

        tokens
    }
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    lexeme: String,
    literal: Box<dyn Any>,
    line: usize,
}

impl Token {
    fn new(kind: TokenKind, lexeme: impl AsRef<str>, literal: Box<dyn Any>, line: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.as_ref().to_owned(),
            literal,
            line,
        }
    }

    fn to_string(&self) -> String {
        // formatting of the literal: Any probably needs to be changed to something useful
        format!("{:?} {} {:?}", self.kind, self.lexeme, self.literal)
    }
}

struct Lox {
    had_error: bool,
}

impl Lox {
    fn new() -> Self {
        Self { had_error: false }
    }

    fn run(&mut self, code: &str) {
        let scanner = Scanner::new(code);
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
