use crate::error::Error;
use crate::iter::PeekableLineColIterator;
use crate::token::Token;

#[derive(Debug)]
pub struct Scanner<'a> {
    pub source: &'a str,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    /// Walk the source and tokenize.
    pub fn tokens(&self) -> Result<Vec<Token<'_>>, Error> {
        let mut tokens = Vec::new();
        let mut scanner = PeekableLineColIterator::new(self.source.chars());

        while let Some(c) = scanner.next() {
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
                    if let Some('=') = scanner.peek() {
                        scanner.next();
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                }
                '=' => {
                    if let Some('=') = scanner.peek() {
                        scanner.next();
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '<' => {
                    if let Some('=') = scanner.peek() {
                        scanner.next();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if let Some('=') = scanner.peek() {
                        scanner.next();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '/' => {
                    let mut lookahead = scanner.clone();
                    if let Some('/') = lookahead.peek() {
                        // C++ style comment, consume the rest of the line.
                        for c in scanner.by_ref() {
                            if c == '\n' {
                                break;
                            }
                        }
                        continue;
                    } else if let Some('*') = lookahead.peek() {
                        /* C-style comment, consume until its end. */
                        // Track where the comment starts.
                        lookahead.next();
                        let start_line = scanner.line();
                        let start_column = scanner.column();
                        let mut length = 1;
                        loop {
                            let Some(pos) = lookahead.position(|c| c == '*') else {
                                return Err(Error::UnterminatedMultilineComment {
                                    source_line: self
                                        .source
                                        .lines()
                                        .nth(start_line - 1)
                                        .expect("current line must be in source")
                                        .to_string(),
                                    line_number: start_line,
                                    column_number: start_column,
                                });
                            };
                            length += pos + 1;
                            if lookahead.peek().is_some_and(|c| *c == '/') {
                                break;
                            }
                        }
                        scanner.nth(length);
                        continue;
                    } else {
                        Token::Slash
                    }
                }
                ' ' | '\n' | '\t' | '\r' => {
                    continue;
                }
                '"' => {
                    // String literal.
                    // Track where the string starts.
                    let start_line = scanner.line();
                    let start_column = scanner.column();
                    let string = &self.source[scanner.offset()..];
                    let mut lookahead = scanner.clone();
                    let Some(pos) = lookahead.position(|c| c == '"') else {
                        return Err(Error::UnterminatedString {
                            source_line: self
                                .source
                                .lines()
                                .nth(start_line - 1)
                                .expect("current line must be in source")
                                .to_string(),
                            line_number: start_line,
                            column_number: start_column,
                        });
                    };
                    let length = pos;
                    scanner.nth(length); // advance source past closing quote
                    Token::String(&string[..length])
                }
                '0'..='9' => {
                    // Number literal.
                    let number = &self.source[scanner.offset() - 1..];
                    let mut length = 1;
                    let mut lookahead = scanner.clone();
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
                        scanner.nth(length - 2); // advance source past number
                    }
                    Token::Number(&number[..length])
                }
                c if c == '_' || c.is_ascii_alphabetic() => {
                    // Reserved words and identifiers.
                    let symbol = &self.source[scanner.offset() - 1..];
                    let mut length = 1;
                    let lookahead = scanner.clone();
                    length += lookahead
                        .take_while(|c| *c == '_' || c.is_ascii_alphanumeric())
                        .count();
                    if length > 1 {
                        scanner.nth(length - 2); // advance source past symbol
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
                    return Err(Error::InvalidCharacter {
                        source_line: self
                            .source
                            .lines()
                            .nth(scanner.line() - 1)
                            .expect("current line must be in source")
                            .to_string(),
                        line_number: scanner.line(),
                        column_number: scanner.column(),
                    });
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
            Token::Number("1"),
            Token::EqualEqual,
            Token::Number("1.0"),
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
        let expected = Err(Error::InvalidCharacter {
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
        let expected = Err(Error::InvalidCharacter {
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
            Token::Number("1"),
            Token::Number("2.0"),
            Token::Number("0.3"),
            Token::Number("000.3"),
            Token::Number("0.0003"),
            Token::Number("123"),
            Token::Number("123.123"),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn number_literals_invalid() {
        let scanner = Scanner::new(".123 123. -123");
        let actual = scanner.tokens().unwrap();
        let expected = vec![
            Token::Dot,
            Token::Number("123"),
            Token::Number("123"),
            Token::Dot,
            Token::Minus,
            Token::Number("123"),
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
