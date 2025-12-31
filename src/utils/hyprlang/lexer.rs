use super::token::{Token, TokenType};
use regex::Regex;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    text: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: text.chars().peekable(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.text[self.pos..].chars().nth(n)
    }

    fn advance(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => {
                self.pos += c.len_utf8();
                if c == '\n' {
                    self.line += 1;
                    self.col = 1;
                } else {
                    self.col += 1;
                }
                Some(c)
            }
            None => None,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_until(&mut self, stop_chars: &str, include_newline: bool) -> String {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if stop_chars.contains(c) {
                break;
            }
            if c == '\n' && !include_newline {
                break;
            }
            result.push(self.advance().unwrap());
        }
        result
    }

    fn read_quoted_string(&mut self) -> String {
        let quote = self.advance().unwrap();
        let mut result = String::new();
        result.push(quote);

        while let Some(c) = self.peek() {
            if c == quote {
                break;
            }
            if c == '\\' {
                if let Some(next) = self.peek_n(1) {
                    if next == quote || next == '\\' {
                        result.push(self.advance().unwrap());
                    }
                }
            }
            result.push(self.advance().unwrap());
        }
        if self.peek() == Some(quote) {
            result.push(self.advance().unwrap());
        }
        result
    }

    fn read_arithmetic(&mut self) -> String {
        self.advance();
        self.advance();
        let mut result = String::new();
        let mut depth = 1;

        while let Some(c) = self.peek() {
            if depth == 0 {
                break;
            }
            if c == '{' && self.peek_n(1) == Some('{') {
                depth += 1;
                result.push(self.advance().unwrap());
                result.push(self.advance().unwrap());
            } else if c == '}' && self.peek_n(1) == Some('}') {
                depth -= 1;
                if depth > 0 {
                    result.push(self.advance().unwrap());
                    result.push(self.advance().unwrap());
                } else {
                    self.advance();
                    self.advance();
                    break;
                }
            } else {
                result.push(self.advance().unwrap());
            }
        }
        result
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(_c) = self.peek() {
            self.skip_whitespace();
            let c_opt = self.peek();
            if c_opt.is_none() {
                break;
            }
            let c = c_opt.unwrap();

            let start_pos = self.pos;
            let start_line = self.line;
            let start_col = self.col;

            if c == '\n' {
                tokens.push(Token::new(
                    TokenType::Newline,
                    "\n",
                    start_line,
                    start_col,
                    start_pos,
                ));
                self.advance();
                continue;
            }

            if c == '#' {
                if self.peek_n(1) == Some('#') {
                    self.advance();
                    self.advance();
                    tokens.push(Token::new(
                        TokenType::Ident,
                        "#",
                        start_line,
                        start_col,
                        start_pos,
                    ));
                    continue;
                }

                self.advance();
                let rest = self.read_until("\n", false);
                let trimmed = rest.trim();
                let token_type = if trimmed.starts_with("hyprlang ") {
                    TokenType::Directive
                } else {
                    TokenType::Comment
                };

                let value = if token_type == TokenType::Directive {
                    rest.trim_start()
                        .strip_prefix("hyprlang ")
                        .unwrap_or("")
                        .trim()
                        .to_string()
                } else {
                    rest
                };

                tokens.push(Token::new(
                    token_type, value, start_line, start_col, start_pos,
                ));
                continue;
            }

            if c == '$' {
                self.advance();
                let name = self.read_until(" \t\n=:{}[]#,", false);
                tokens.push(Token::new(
                    TokenType::Variable,
                    name,
                    start_line,
                    start_col,
                    start_pos,
                ));
                continue;
            }

            if c == '{' && self.peek_n(1) == Some('{') {
                let expr = self.read_arithmetic();
                tokens.push(Token::new(
                    TokenType::Arithmetic,
                    expr,
                    start_line,
                    start_col,
                    start_pos,
                ));
                continue;
            }

            let token_type = match c {
                '=' => Some(TokenType::Equals),
                ':' => Some(TokenType::Colon),
                '{' => Some(TokenType::LBrace),
                '}' => Some(TokenType::RBrace),
                '[' => Some(TokenType::LBracket),
                ']' => Some(TokenType::RBracket),
                ',' => Some(TokenType::Comma),
                _ => None,
            };

            if let Some(tt) = token_type {
                self.advance();
                tokens.push(Token::new(
                    tt,
                    c.to_string(),
                    start_line,
                    start_col,
                    start_pos,
                ));
                continue;
            }

            if c == '"' || c == '\'' {
                let s = self.read_quoted_string();
                tokens.push(Token::new(
                    TokenType::String,
                    s,
                    start_line,
                    start_col,
                    start_pos,
                ));
                continue;
            }

            let ident = self.read_until(" \t\n=:{}[]#,", false);
            if !ident.is_empty() {
                let kind = if is_number(&ident) {
                    TokenType::Number
                } else {
                    TokenType::Ident
                };
                tokens.push(Token::new(kind, ident, start_line, start_col, start_pos));
            } else {
                self.advance();
            }
        }

        tokens.push(Token::new(
            TokenType::Eof,
            "",
            self.line,
            self.col,
            self.pos,
        ));
        tokens
    }
}

use lazy_static::lazy_static;

fn is_number(s: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^-?\d+(\.\d+)?$").unwrap();
    }
    RE.is_match(s)
}
