use std::str::Chars;

use crate::source::PeekableLineColIterator;
use crate::token::Token;

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    chars: PeekableLineColIterator<Chars<'a>>,
}

/// The maximum number of scan errors to allow before giving up.
const MAX_SCAN_ERRORS: u32 = 100;

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: PeekableLineColIterator::new(source.chars()),
        }
    }

    /// Walk the source and tokenize.
    pub fn tokens(self) -> Vec<Token> {
        self.scan(0, |n_errors, token| {
            if token.is_invalid() {
                *n_errors += 1
            }

            if *n_errors <= MAX_SCAN_ERRORS {
                Some(token)
            } else {
                None
            }
        })
        .collect()
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.chars.by_ref().find(|c| !c.is_whitespace())?;
        let pos = self.chars.offset() - 1;
        let src = &self.source[pos..];

        let token = match c {
            '(' => Token::new_left_paren(pos),
            ')' => Token::new_right_paren(pos),
            '{' => Token::new_left_brace(pos),
            '}' => Token::new_right_brace(pos),
            ',' => Token::new_comma(pos),
            '.' => Token::new_dot(pos),
            '-' => Token::new_minus(pos),
            '+' => Token::new_plus(pos),
            ';' => Token::new_semicolon(pos),
            '*' => Token::new_star(pos),
            '!' => {
                if let Some('=') = self.chars.peek() {
                    self.chars.next();
                    Token::new_bang_equal(pos)
                } else {
                    Token::new_bang(pos)
                }
            }
            '=' => {
                if let Some('=') = self.chars.peek() {
                    self.chars.next();
                    Token::new_equal_equal(pos)
                } else {
                    Token::new_equal(pos)
                }
            }
            '<' => {
                if let Some('=') = self.chars.peek() {
                    self.chars.next();
                    Token::new_less_equal(pos)
                } else {
                    Token::new_less(pos)
                }
            }
            '>' => {
                if let Some('=') = self.chars.peek() {
                    self.chars.next();
                    Token::new_greater_equal(pos)
                } else {
                    Token::new_equal(pos)
                }
            }
            '/' => {
                match self.chars.peek() {
                    Some('/') => {
                        // Line comment, consume to the end of the line.

                        let len = if let Some(line_len) = src.find('\n') {
                            self.chars.nth(line_len);
                            line_len
                        } else {
                            // Line must end the file. Count remaining chars,
                            // accounting for leading `/`.
                            self.chars.by_ref().count() + 1
                        };

                        Token::new_line_comment(pos, &src[..len])
                    }
                    Some('*') => {
                        // Block comment, consume until its end.

                        let mut len = 2;

                        // C-style comments can be nested, like `/* /* comment */ */`
                        let mut depth = 1;

                        loop {
                            let next_pos = src[len..].find(['/', '*']);

                            // Ensure some block comment character was found and
                            // enough source remains to look for the second.
                            if next_pos.is_none_or(|pos| src[(len + pos)..].len() < 2) {
                                self.chars.by_ref().count(); // drain scanner
                                return Some(Token::new_unterminated_block_comment(pos, src));
                            };

                            len += next_pos.unwrap() + 1;

                            match &src[(len - 1)..(len + 1)] {
                                "/*" => {
                                    len += 1;
                                    depth += 1;
                                }
                                "*/" => {
                                    len += 1;
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                _ => continue,
                            }
                        }

                        // Move scanner past comment. Account for start len 2.
                        self.chars.nth(len - 2);

                        Token::new_block_comment(pos, &src[..len])
                    }
                    _ => Token::new_slash(pos),
                }
            }
            '"' => {
                // String literal.

                let mut len = 1;

                loop {
                    let Some(quote_pos) = src[len..].find('"') else {
                        self.chars.by_ref().count(); // drain scanner
                        return Some(Token::new_unterminated_string(pos, src));
                    };

                    len += quote_pos + 1;
                    self.chars.nth(quote_pos + 1); // move scanner past this quote

                    // If quote was escaped, keep parsing, otherwise done. The
                    // position of the final quote is `len - 1`. Look for
                    // escaping `\` just before that.
                    if src.as_bytes()[len - 2] != b'\\' {
                        break;
                    }
                }

                Token::new_string(pos, &src[..len])
            }
            '0'..='9' => {
                // Number literal.

                let mut len = 1;
                let mut lookahead = self.chars.clone();

                while lookahead.peek().is_some_and(|c| c.is_ascii_digit()) {
                    lookahead.next();
                    len += 1;
                }

                if lookahead.next().is_some_and(|c| c == '.')
                    && lookahead.peek().is_some_and(|c| c.is_ascii_digit())
                {
                    len += lookahead.take_while(|c| c.is_ascii_digit()).count() + 1;
                }

                if len > 1 {
                    self.chars.nth(len - 2); // advance scanner past number
                }

                Token::new_number(pos, &src[..len])
            }
            c if c == '_' || c.is_ascii_alphabetic() => {
                // Reserved words and identifiers.
                let mut len = 1;
                let lookahead = self.chars.clone();

                len += lookahead
                    .take_while(|c| *c == '_' || c.is_ascii_alphanumeric())
                    .count();
                if len > 1 {
                    self.chars.nth(len - 2); // advance source past symbol
                }

                let lexeme = &src[..len];
                match lexeme {
                    "and" | "class" | "else" | "false" | "fun" | "for" | "if" | "nil" | "or"
                    | "print" | "return" | "super" | "this" | "true" | "var" | "while" => {
                        Token::new_keyword(pos, lexeme)
                    }
                    _ => Token::new_identifier(pos, lexeme),
                }
            }
            _ => {
                let token = Token::new_invalid_character(pos, &src[..1]);

                // Consume to the end of the line and keep lexin'.
                self.chars.by_ref().take_while(|c| *c != '\n').count();

                token
            }
        };

        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use crate::token::TokenKind;

    use super::*;
    use indoc::indoc;

    #[test]
    fn single_and_double_lexemes() {
        let source = "!=<=>===({,.-=;*})";
        let scanner = Scanner::new(source);
        let actual: Vec<_> = scanner.tokens().iter().map(|tok| tok.kind()).collect();
        let expected = vec![
            TokenKind::BangEqual,
            TokenKind::LessEqual,
            TokenKind::GreaterEqual,
            TokenKind::EqualEqual,
            TokenKind::LeftParen,
            TokenKind::LeftBrace,
            TokenKind::Comma,
            TokenKind::Dot,
            TokenKind::Minus,
            TokenKind::Equal,
            TokenKind::Semicolon,
            TokenKind::Star,
            TokenKind::RightBrace,
            TokenKind::RightParen,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn line_comment_eof() {
        let source = "// this is a comment line";
        let mut scanner = Scanner::new(source);
        let actual = scanner.next().expect("comment should be a token");
        let expected = Token::new_line_comment(0, source);
        assert_eq!(actual, expected);
    }

    #[test]
    fn line_comment_eol() {
        let source = "var a = 1; // eol comment";
        let scanner = Scanner::new(source);
        let actual = scanner.last().unwrap();
        let expected = Token::new_line_comment(11, &source[11..]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn line_comment_ends_at_eol() {
        let source = indoc! {
            r#"// eol comment
            "#
        };
        let scanner = Scanner::new(source);
        let actual = scanner.tokens();
        let expected = vec![Token::new_line_comment(0, "// eol comment")];
        assert_eq!(actual, expected);
    }

    #[test]
    fn block_comment() {
        let source = "(/* a *block* comment */)";
        let scanner = Scanner::new(source);
        let actual = scanner.tokens();
        let expected = vec![
            Token::new_left_paren(0),
            Token::new_block_comment(1, "/* a *block* comment */"),
            Token::new_right_paren(24),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn unterminated_block_comment() {
        let source = "/* comment *";
        let mut scanner = Scanner::new(source);
        let actual = scanner.next().unwrap();
        let expected = Token::new_unterminated_block_comment(0, source);
        assert_eq!(actual, expected);
    }

    #[test]
    fn nested_block_comment() {
        let source = "/* a /* nested /**block**/*/ /**/** comment */";
        let mut scanner = Scanner::new(source);
        let actual = scanner.next().unwrap();
        let expected = Token::new_block_comment(0, source);
        assert_eq!(actual, expected);
    }

    #[test]
    fn multiline_block_comment() {
        let source = indoc! {r#"
            var t1; /* multiline
            block
            comment */ var t2;"#
        };
        let scanner = Scanner::new(source);
        let tokens = scanner.tokens();
        assert_eq!(tokens.len(), 7);
        let actual = &tokens[3];
        let expected = &Token::new_block_comment(
            8,
            indoc! {r#"/* multiline
            block
            comment */"#
            },
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_literals() {
        let source = r#""string (123) // check""#;
        let mut scanner = Scanner::new(source);
        let actual = scanner.next().unwrap();
        let expected = Token::new_string(0, source);
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_literals_escaped() {
        let source = r#""string \"123\" // check""#;
        let mut scanner = Scanner::new(source);
        let actual = scanner.next().unwrap();
        let expected = Token::new_string(0, source);
        assert_eq!(actual, expected);
    }

    #[test]
    fn multiline_string_literals() {
        let source = indoc! {r#""this is a
            multi line
            string.""#
        };
        let mut scanner = Scanner::new(source);
        let actual = scanner.next().unwrap();
        let expected = Token::new_string(0, source);
        assert_eq!(actual, expected);
    }

    #[test]
    fn number_literals_valid() {
        let source = "1 2.0 0.3 000.3 0.0003 123 123.123";
        let scanner = Scanner::new(source);
        let tokens = scanner.tokens();
        let actual: Vec<_> = tokens.iter().map(|tok| tok.lexeme(source)).collect();
        let expected = vec!["1", "2.0", "0.3", "000.3", "0.0003", "123", "123.123"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn number_literals_invalid() {
        let scanner = Scanner::new(".123 456. -789");
        let actual = scanner.tokens();
        let expected = vec![
            Token::new_dot(0),
            Token::new_number(1, "123"),
            Token::new_number(5, "456"),
            Token::new_dot(8),
            Token::new_minus(10),
            Token::new_number(11, "789"),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn reserved_keywords() {
        let source =
            "and class else false fun for if nil or print return super this true var while";
        let words = source.split_whitespace();
        let scanner = Scanner::new(source);
        for (token, word) in scanner.zip(words) {
            assert!(matches!(token.kind(), TokenKind::Keyword(..)));
            let actual = token.lexeme(&source);
            let expected = word;
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn identifiers() {
        let source = "_ _test test_T1 test test_TEST a1 _42";
        let scanner = Scanner::new(source);
        let words = source.split_whitespace();
        for (token, word) in scanner.zip(words) {
            assert!(matches!(token.kind(), TokenKind::Identifier));
            let actual = token.lexeme(source);
            let expected = word;
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn maximal_munch() {
        let source = "andor _and or_ Or";
        let scanner = Scanner::new(source);
        let words = source.split_whitespace();
        for (token, word) in scanner.zip(words) {
            assert!(matches!(token.kind(), TokenKind::Identifier));
            let actual = token.lexeme(source);
            let expected = word;
            assert_eq!(actual, expected);
        }
    }
}
