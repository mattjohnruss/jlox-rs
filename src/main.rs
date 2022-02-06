#![allow(unused)]

use anyhow::{Context, Result};

use std::io;
use std::io::prelude::*;

#[derive(Debug)]
enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
}

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
    Literal(Literal),
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

// TODO: this is awful - replace with proper error handling
static mut HAD_ERROR: bool = false;

#[derive(Debug)]
struct Scanner<'source> {
    source: &'source str,
    tokens: Vec<Token>,
    line: usize,
    lexeme: String,
}

impl<'source> Scanner<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            tokens: vec![],
            line: 1,
            lexeme: String::new(),
        }
    }

    fn scan(&mut self) -> &[Token] {
        let mut char_iter = self.source.chars().peekable();

        use TokenKind as TK;

        while let Some(c) = char_iter.next() {
            self.lexeme.push(c);

            let p = char_iter.peek();

            match c {
                '(' => self.add_token(TK::LeftParen),
                ')' => self.add_token(TK::RightParen),
                '{' => self.add_token(TK::LeftBrace),
                '}' => self.add_token(TK::RightBrace),
                ',' => self.add_token(TK::Comma),
                '.' => self.add_token(TK::Dot),
                '-' => self.add_token(TK::Minus),
                '+' => self.add_token(TK::Plus),
                ';' => self.add_token(TK::Semicolon),
                '*' => self.add_token(TK::Star),
                '!' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(TK::BangEqual);
                        char_iter.next();
                    } else {
                        self.add_token(TK::Bang)
                    }
                }
                '=' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(TK::EqualEqual);
                        char_iter.next();
                    } else {
                        self.add_token(TK::Equal)
                    }
                }
                '<' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(TK::LessEqual);
                        char_iter.next();
                    } else {
                        self.add_token(TK::Less)
                    }
                }
                '>' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(TK::GreaterEqual);
                        char_iter.next();
                    } else {
                        self.add_token(TK::Greater)
                    }
                }
                '/' => {
                    if let Some('/') = p {
                        // Don't skip over the second slash - let loop below start at the second
                        // second slash so (which we know exists at this point) so we can peek to
                        // the next character.

                        // The rest of the line is a comment so now skip to the end
                        while let Some(c_comment) = char_iter.next() {
                            if let Some('\n') = char_iter.peek() {
                                break;
                            }
                        }

                        // We have to clear the lexeme string here manually because the double
                        // slash isn't treated as a token, but we've already added the first slash
                        // to the string. Would it be better to treat it as a token and simply
                        // ignore it later?
                        self.lexeme.clear();
                    } else {
                        self.add_token(TK::Slash);
                    }
                }
                '"' => {
                    let mut lit = String::new();

                    while let Some(&c_next) = char_iter.peek() {
                        match c_next {
                            '"' => {
                                self.lexeme.push(c_next);
                                self.add_token(TK::Literal(Literal::String(lit)));
                                break;
                            }
                            '\n' => {
                                self.lexeme.push(c_next);
                                lit.push(c_next);
                                self.line += 1;
                            }
                            _ => {
                                self.lexeme.push(c_next);
                                lit.push(c_next);
                            }
                        }
                        char_iter.next();

                        // Check if we reached the end before finding a closing quote
                        if char_iter.peek().is_none() {
                            Lox::report(self.line, "", "Unterminated string.");
                        }
                    }

                    // Skip over the closing quote
                    char_iter.next();
                }
                '0'..='9' => {
                    // Digits in the integer part
                    while let Some(&c_next @ '0'..='9') = char_iter.peek() {
                        self.lexeme.push(c_next);
                        char_iter.next();
                    }

                    if let Some(&c_next @ '.') = char_iter.peek() {
                        // How can we disallow trailing '.' in number literals if we can't do 2
                        // character lookahead? (`Peekable` only allows for 1 char lookahead via
                        // peak() - we could switch to using `...chars().windows(3)` in the main
                        // loop?). For now, assume the '.' is part of the number regardless of what
                        // comes after it.
                        self.lexeme.push(c_next);
                        char_iter.next();

                        // Digits in the fractional part
                        while let Some(&c_next @ '0'..='9') = char_iter.peek() {
                            self.lexeme.push(c_next);
                            char_iter.next();
                        }
                    }
                    let lit: f64 = self
                        .lexeme
                        .parse()
                        .expect(&format!("error parsing number literal: `{}`", self.lexeme));
                    self.add_token(TK::Literal(Literal::Number(lit)));
                }
                ' ' | '\r' | '\t' => {}
                '\n' => {
                    self.line += 1;
                    self.lexeme.clear();
                }
                _ => Lox::report(self.line, "", &format!("unexpected character `{c}`")),
            }
        }

        self.tokens.push(Token::new(TK::Eof, "", self.line));

        &self.tokens
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token::new(kind, &self.lexeme, self.line));
        self.lexeme.clear();
    }
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    lexeme: String,
    line: usize,
}

impl Token {
    fn new(kind: TokenKind, lexeme: impl AsRef<str>, line: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.as_ref().to_owned(),
            line,
        }
    }

    fn to_string(&self) -> String {
        // formatting of the literal
        format!("{:?} {}", self.kind, self.lexeme)
    }
}

struct Lox {}

impl Lox {
    fn new() -> Self {
        Self {}
    }

    fn run(&mut self, code: &str) {
        let mut scanner = Scanner::new(code);
        let tokens = scanner.scan();

        for token in tokens {
            println!("{:?}", token);
        }
    }

    fn run_file(&mut self, filename: &str) -> Result<()> {
        let code = std::fs::read_to_string(filename).context("Could not read code from file")?;
        self.run(&code);

        if unsafe { HAD_ERROR } {
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

            if line.is_empty() {
                break;
            }

            if line == "\n" {
                line.clear();
                continue;
            }

            self.run(&line);
            unsafe { HAD_ERROR = false };

            line.clear();
        }

        Ok(())
    }

    fn report(line: usize, loc: &str, msg: &str) {
        println!("[line {line}] Error{loc}: {msg}");
        unsafe {
            HAD_ERROR = true;
        }
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
