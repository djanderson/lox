use std::str::Chars;

use crate::errors::LoxError;
use crate::token::Token;

#[derive(Debug)]
pub struct Scanner {
    pub source: String,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    pub fn tokens(&self) -> Result<Vec<Token<'_>>, LoxError> {
        let mut tokens = Vec::new();
        let mut line = 0;
        let mut column = 0;
        let mut chars = self.source.as_str().chars();

        while let Some(c) = chars.next() {
            column += 1;
            let token = match c {
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                ',' => Token::Comma,
                '.' => Token::Dot,
                '-' => Token::Minus,
                '+' => Token::Plus,
                ';' => Token::Semicolon,
                '*' => Token::Star,
                '!' => {
                    if chars.clone().next() == Some('=') {
                        chars.next();
                        column += 1;
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                }
                '=' => {
                    if chars.clone().next() == Some('=') {
                        chars.next();
                        column += 1;
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '<' => {
                    if chars.clone().next() == Some('=') {
                        chars.next();
                        column += 1;
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if chars.clone().next() == Some('=') {
                        chars.next();
                        column += 1;
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '/' => {
                    if chars.clone().next() == Some('/') {
                        // Comment, consume the rest of the line.
                        for c in chars.by_ref() {
                            if c == '\n' {
                                break;
                            }
                        }
                        column = 0;
                        line += 1;
                        continue;
                    } else {
                        Token::Slash
                    }
                }
                ' ' | '\r' | '\t' => {
                    column += 1;
                    continue;
                }
                '\n' => {
                    line += 1;
                    column = 0;
                    continue;
                }
                '"' => {
                    // String literal.
                    // Track where the string starts.
                    let start_line = line;
                    let start_column = column;
                    let string_start = chars.as_str();
                    let mut last_newline_position = 0;
                    let mut length = 0;
                    loop {
                        if let Some(p) = chars.position(|c| c == '\n' || c == '"') {
                            length += p;
                            if &string_start[length..length + 1] == "\n" {
                                // Support multiline strings.
                                line += 1;
                                last_newline_position = length;
                            } else {
                                column = length - last_newline_position;
                                break;
                            }
                        } else {
                            return Err(LoxError::UnterminatedString {
                                source_line: self
                                    .source
                                    .lines()
                                    .nth(start_line)
                                    .expect("currrent line is in source")
                                    .to_string(),
                                line_number: start_line + 1,
                                column_number: start_column,
                            });
                        }
                    }
                    Token::String(&string_start[..length])
                }
                _ => {
                    return Err(LoxError::InvalidSyntax {
                        source_line: self
                            .source
                            .lines()
                            .nth(line)
                            .expect("currrent line is in source")
                            .to_string(),
                        line_number: line + 1,
                        column_number: column,
                    })
                }
            };
            tokens.push(token);
        }

        Ok(tokens)
    }
}
