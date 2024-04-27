use crate::errors::LoxError;
use crate::token::Token;

#[derive(Debug)]
pub struct Scanner {
    pub source: String,
}

impl Scanner {
    pub fn new(source: impl ToString) -> Self {
        Self {
            source: source.to_string(),
        }
    }

    /// Walk the source and tokenize.
    pub fn tokens(&self) -> Result<Vec<Token<'_>>, LoxError> {
        let mut tokens = Vec::new();
        let mut line = 0;
        let mut last_newline_position = 0;
        let mut column;
        let mut chars = self.source.chars().enumerate().peekable();

        while let Some((idx, c)) = chars.next() {
            column = (idx - last_newline_position) + 1;
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
                    if let Some((_, '=')) = chars.peek() {
                        chars.next();
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                }
                '=' => {
                    if let Some((_, '=')) = chars.peek() {
                        chars.next();
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '<' => {
                    if let Some((_, '=')) = chars.peek() {
                        chars.next();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if let Some((_, '=')) = chars.peek() {
                        chars.next();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '/' => {
                    if let Some((_, '/')) = chars.peek() {
                        // Comment, consume the rest of the line.
                        for (i, c) in chars.by_ref() {
                            if c == '\n' {
                                last_newline_position = i;
                                break;
                            }
                        }
                        line += 1;
                        continue;
                    } else {
                        Token::Slash
                    }
                }
                ' ' | '\r' | '\t' => {
                    continue;
                }
                '\n' => {
                    last_newline_position = idx;
                    line += 1;
                    continue;
                }
                '"' => {
                    // String literal.
                    // Track where the string starts.
                    let start_line = line;
                    let start_column = column;
                    let string_start = &self.source[idx + 1..];
                    let mut length = 0;
                    loop {
                        if let Some(p) = chars.position(|(_, c)| c == '\n' || c == '"') {
                            length += p;
                            if &string_start[length..length + 1] == "\n" {
                                // Support multiline strings.
                                line += 1;
                                last_newline_position = length;
                            } else {
                                break;
                            }
                        } else {
                            return Err(LoxError::UnterminatedString {
                                source_line: self
                                    .source
                                    .lines()
                                    .nth(start_line)
                                    .expect("currrent line must be in source")
                                    .to_string(),
                                line_number: start_line + 1,
                                column_number: start_column,
                            });
                        }
                    }
                    Token::String(&string_start[..length])
                }
                c @ '0'..='9' => {
                    let mut number = String::with_capacity(1);
                    number.push(c);
                    column += 1;
                    let mut lookahead = chars.clone();
                    while lookahead
                        .by_ref()
                        .peek()
                        .is_some_and(|(_, c)| c.is_ascii_digit())
                    {
                        lookahead.next();
                        number.push(chars.next().map(|t| t.1).expect("next char is number"));
                        column += 1;
                    }
                    if lookahead.next().is_some_and(|t| t.1 == '.')
                        && lookahead.peek().is_some_and(|t| t.1.is_ascii_digit())
                    {
                        number.push(
                            chars
                                .next()
                                .map(|t| t.1)
                                .expect("next char is decimal point"),
                        );
                        column += 1;
                        for (_, c) in lookahead.by_ref() {
                            if c.is_ascii_digit() {
                                number
                                    .push(chars.next().map(|t| t.1).expect("next char is number"));
                                column += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    Token::Number(number.parse().expect("number should parse as f32"))
                }
                // c if c == '_' || c.is_ascii_alphabetic() => {

                // }
                _ => {
                    return Err(LoxError::InvalidSyntax {
                        source_line: self
                            .source
                            .lines()
                            .nth(line)
                            .expect("currrent line must be in source")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_and_double_lexemes() {
        let source = "!=<=>===({,.-=;*})";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::BangEqual,
            Token::LessEqual,
            Token::GreaterEqual,
            Token::EqualEqual,
            Token::LeftParen,
            Token::LeftBrace,
            Token::Comma,
            Token::Dot,
            Token::Minus,
            Token::Equal,
            Token::Semicolon,
            Token::Star,
            Token::RightBrace,
            Token::RightParen,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn ignore_comment_line() {
        let source = "// this is a comment line";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = Vec::new();
        assert_eq!(actual, expected);
    }

    #[test]
    fn ignore_comment_eol() {
        let source = "(1 == 1.0) // this is an end-of-line comment";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::LeftParen,
            Token::Number(1.0),
            Token::EqualEqual,
            Token::Number(1.0),
            Token::RightParen,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_literals() {
        let source = r#""string (123) // check""#;
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        // Expect the source without surrounding double quotes
        let expected = vec![Token::String(&source[1..source.len() - 1])];
        assert_eq!(actual, expected);
    }

    #[test]
    fn number_literals_valid() {
        let scanner = Scanner::new("1 2.0 0.3 000.3 0.0003 123 123.123");
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::Number(1.0),
            Token::Number(2.0),
            Token::Number(0.3),
            Token::Number(0.3),
            Token::Number(0.0003),
            Token::Number(123.0),
            Token::Number(123.123),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn number_literals_invalid() {
        let scanner = Scanner::new(".123 123. -123");
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::Dot,
            Token::Number(123.0),
            Token::Number(123.0),
            Token::Dot,
            Token::Minus,
            Token::Number(123.0),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn reserved_keywords() {
        let source =
            "and class else false fun for if nil or print return super this true var while";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::And,
            Token::Class,
            Token::False,
            Token::Fun,
            Token::For,
            Token::If,
            Token::Nil,
            Token::Or,
            Token::Print,
            Token::Return,
            Token::Super,
            Token::This,
            Token::True,
            Token::Var,
            Token::While,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn identifiers() {
        let source = "_ _test test_t test test_test";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::Identifier("_"),
            Token::Identifier("_test"),
            Token::Identifier("test_t"),
            Token::Identifier("test"),
            Token::Identifier("test_test"),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn maximal_munch() {
        let source = "andor and or _and or_ Or";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::Identifier("andor"),
            Token::Or,
            Token::And,
            Token::Identifier("_and"),
            Token::Identifier("or_"),
            Token::Identifier("Or"),
        ];
        assert_eq!(actual, expected);
    }
}
