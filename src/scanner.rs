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
        let mut source = self.source.chars().enumerate().peekable();

        while let Some((index, c)) = source.next() {
            column = (index - last_newline_position) + 1;
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
                    if let Some('=') = source.peek().map(|t| t.1) {
                        source.next();
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                }
                '=' => {
                    if let Some('=') = source.peek().map(|t| t.1) {
                        source.next();
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '<' => {
                    if let Some('=') = source.peek().map(|t| t.1) {
                        source.next();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if let Some('=') = source.peek().map(|t| t.1) {
                        source.next();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '/' => {
                    let mut lookahead = source.clone().map(|t| t.1).peekable();
                    if let Some('/') = lookahead.peek() {
                        // C++ style comment, consume the rest of the line.
                        for (i, c) in source.by_ref() {
                            if c == '\n' {
                                line += 1;
                                last_newline_position = i + 1;
                                break;
                            }
                        }
                        continue;
                    } else if let Some('*') = lookahead.peek() {
                        /* C-style comment, consume until its end. */
                        // Track where the comment starts.
                        lookahead.next();
                        let start_line = line;
                        let start_column = column;
                        let mut length = 1;
                        loop {
                            let Some(p) = lookahead.position(|c| c == '*') else {
                                return Err(LoxError::UnterminatedComment {
                                    source_line: self
                                        .source
                                        .lines()
                                        .nth(start_line)
                                        .expect("currrent line must be in source")
                                        .to_string(),
                                    line_number: start_line + 1,
                                    column_number: start_column,
                                });
                            };
                            length += p + 1;
                            if lookahead.peek().is_some_and(|c| *c == '/') {
                                length += 1;
                                break;
                            }
                        }
                        // Advance source past the comment and record last newline.
                        for (i, c) in source.by_ref().take(length) {
                            if c == '\n' {
                                line += 1;
                                last_newline_position = i + 1;
                            }
                        }
                        continue;
                    } else {
                        Token::Slash
                    }
                }
                ' ' | '\r' | '\t' => {
                    continue;
                }
                '\n' => {
                    line += 1;
                    last_newline_position = index + 1;
                    continue;
                }
                '"' => {
                    // String literal.
                    // Track where the string starts.
                    let start_line = line;
                    let start_column = column;
                    let string = &self.source[index + 1..];
                    let mut length = 0;
                    let mut lookahead = source.clone().map(|t| t.1).peekable();
                    loop {
                        let Some(p) = lookahead.position(|c| c == '\n' || c == '"') else {
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
                        };
                        length += p + 1;
                        if &string[length - 1..length] == "\n" {
                            line += 1;
                            last_newline_position = index + 1 + length;
                        } else {
                            // At closing quote, string ends one character back.
                            length -= 1;
                            break;
                        }
                    }
                    source.nth(length); // advance source past closing quote
                    Token::String(&string[..length])
                }
                '0'..='9' => {
                    // Number literal.
                    let number = &self.source[index..];
                    let mut length = 1;
                    let mut lookahead = source.clone().map(|t| t.1).peekable();
                    while lookahead.peek().is_some_and(|c| c.is_ascii_digit()) {
                        lookahead.next();
                        length += 1;
                    }
                    if lookahead.next().is_some_and(|c| c == '.')
                        && lookahead.peek().is_some_and(|c| c.is_ascii_digit())
                    {
                        length += lookahead.take_while(|c| c.is_ascii_digit()).count() + 1;
                    }
                    if length > 1 {
                        source.nth(length - 2); // advance source past number
                    }
                    Token::Number(number[..length].parse().expect("valid f32"))
                }
                c if c == '_' || c.is_ascii_alphabetic() => {
                    // Reserved words and identifiers.
                    let symbol = &self.source[index..];
                    let mut length = 1;
                    let lookahead = source.clone().map(|t| t.1).peekable();
                    length += lookahead
                        .take_while(|c| *c == '_' || c.is_ascii_alphanumeric())
                        .count();
                    if length > 1 {
                        source.nth(length - 2); // advance source past symbol
                    }
                    match &symbol[..length] {
                        "and" => Token::And,
                        "class" => Token::Class,
                        "else" => Token::Else,
                        "false" => Token::False,
                        "fun" => Token::Fun,
                        "for" => Token::For,
                        "if" => Token::If,
                        "nil" => Token::Nil,
                        "or" => Token::Or,
                        "print" => Token::Print,
                        "return" => Token::Return,
                        "super" => Token::Super,
                        "this" => Token::This,
                        "true" => Token::True,
                        "var" => Token::Var,
                        "while" => Token::While,
                        identifier => Token::Identifier(identifier),
                    }
                }
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
    use indoc::indoc;

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
    fn ignore_c_style_comments() {
        let source = "(/* this *should* be ignored */)";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![Token::LeftParen, Token::RightParen];
        assert_eq!(actual, expected);
    }

    #[test]
    fn ignore_c_style_comments_multiline() {
        let source = indoc! {r#"
            var t1; /* this
            should be
            ignored */ var t2;"#
        };
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::Var,
            Token::Identifier("t1"),
            Token::Semicolon,
            Token::Var,
            Token::Identifier("t2"),
            Token::Semicolon,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn line_counting_multiline_comments() {
        let source = indoc! {r#"
            var t1; /* this
            should be
            ignored */ var t2;@"#
        };
        let scanner = Scanner::new(source);
        let actual = scanner.tokens();
        let expected = Err(LoxError::InvalidSyntax {
            source_line: "ignored */ var t2;@".to_string(),
            line_number: 3,
            column_number: 19,
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn line_counting_multiline_string() {
        let source = indoc! {r#"
            var t1 = "this is a
            multi line
            string"; var t2;@"#
        };
        let scanner = Scanner::new(source);
        let actual = scanner.tokens();
        let expected = Err(LoxError::InvalidSyntax {
            source_line: r#"string"; var t2;@"#.to_string(),
            line_number: 3,
            column_number: 17,
        });
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
    fn multiline_string_literals() {
        let source = indoc! {r#""this is a
            multi line
            string.""#
        };
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
            Token::Else,
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
        let source = "_ _test test_T1 test test_TEST a1 _42";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::Identifier("_"),
            Token::Identifier("_test"),
            Token::Identifier("test_T1"),
            Token::Identifier("test"),
            Token::Identifier("test_TEST"),
            Token::Identifier("a1"),
            Token::Identifier("_42"),
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
            Token::And,
            Token::Or,
            Token::Identifier("_and"),
            Token::Identifier("or_"),
            Token::Identifier("Or"),
        ];
        assert_eq!(actual, expected);
    }
}
