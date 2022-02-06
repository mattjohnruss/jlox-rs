#![allow(unused)]

use anyhow::{Context, Result};

use std::io;
use std::io::prelude::*;

#[derive(Debug)]
enum Literal {
    Identifier,
    String,
    Number,
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

    fn scan<'scanner>(&'scanner mut self) -> &'scanner [Token] {
        let mut char_iter = self.source.chars().peekable();

        use TokenKind::*;

        while let Some(c) = char_iter.next() {
            self.lexeme.push(c);

            let p = char_iter.peek();

            match c {
                '(' => self.add_token(LeftParen),
                ')' => self.add_token(RightParen),
                '{' => self.add_token(LeftBrace),
                '}' => self.add_token(RightBrace),
                ',' => self.add_token(Comma),
                '.' => self.add_token(Dot),
                '-' => self.add_token(Minus),
                '+' => self.add_token(Plus),
                ';' => self.add_token(Semicolon),
                '*' => self.add_token(Star),
                '!' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(BangEqual);
                        char_iter.next();
                    } else {
                        self.add_token(Bang)
                    }
                }
                '=' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(EqualEqual);
                        char_iter.next();
                    } else {
                        self.add_token(Equal)
                    }
                }
                '<' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(LessEqual);
                        char_iter.next();
                    } else {
                        self.add_token(Less)
                    }
                }
                '>' => {
                    if let Some(&c_next @ '=') = p {
                        self.lexeme.push(c_next);
                        self.add_token(GreaterEqual);
                        char_iter.next();
                    } else {
                        self.add_token(Greater)
                    }
                }
                '/' => {
                    if let Some('/') = p {
                        // Don't skip over the second slash - let loop below start at the second
                        // second slash so (which we know exists at this point) so we can peek to
                        // the next character.

                        // The rest of the line is a comment so now skip to the end
                        while let Some(c_comment) = char_iter.next() {
                            dbg!(c_comment);
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
                        self.add_token(Slash);
                    }
                }
                ' ' | '\r' | '\t' => {}
                '\n' => {
                    self.line += 1;
                    self.lexeme.clear();
                }
                _ => Lox::report(self.line, "", &format!("unexpected character `{c}`")),
            }
        }

        self.tokens
            .push(Token::new(TokenKind::Eof, "", None, self.line));

        &self.tokens
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens
            .push(Token::new(kind, &self.lexeme, None, self.line));
        self.lexeme.clear();
    }
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
}

impl Token {
    fn new(
        kind: TokenKind,
        lexeme: impl AsRef<str>,
        literal: Option<Literal>,
        line: usize,
    ) -> Self {
        Self {
            kind,
            lexeme: lexeme.as_ref().to_owned(),
            literal,
            line,
        }
    }

    fn to_string(&self) -> String {
        // formatting of the literal
        format!("{:?} {} {:?}", self.kind, self.lexeme, self.literal)
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

            if line.len() == 0 {
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
