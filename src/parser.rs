use crate::error::Error;
use std::iter::Peekable;
use std::slice::Iter;

use crate::expr::Expr;
use crate::token::{Keyword, Token, TokenKind};

pub struct Parser<'tok> {
    tokens: Peekable<Iter<'tok, Token>>,
}

/// Recursive descent parser
impl<'tok> Parser<'tok> {
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        self.expression()
    }

    /// expression -> equality ;
    fn expression(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        self.equality()
    }

    /// equality -> comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        let mut expr = self.comparison()?;

        while let Some(TokenKind::BangEqual | TokenKind::EqualEqual) =
            self.tokens.peek().map(|tok| tok.kind())
        {
            let operator = self.tokens.next().unwrap();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    /// comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        let mut expr = self.term()?;

        while let Some(
            TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::LessEqual,
        ) = self.tokens.peek().map(|tok| tok.kind())
        {
            let operator = self.tokens.next().unwrap();
            let right = self.term()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    /// term -> factor ( ( "- | "+" ) factor )* ;
    fn term(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        let mut expr = self.factor()?;

        while let Some(TokenKind::Minus | TokenKind::Plus) =
            self.tokens.peek().map(|tok| tok.kind())
        {
            let operator = self.tokens.next().unwrap();
            let right = self.factor()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    /// factor -> unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        let mut expr = self.unary()?;

        while let Some(TokenKind::Slash | TokenKind::Star) =
            self.tokens.peek().map(|tok| tok.kind())
        {
            let operator = self.tokens.next().unwrap();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    /// unary -> ( "!" | "-" ) unary
    ///        | primary ;
    fn unary(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        let expr = match self.tokens.peek().map(|tok| tok.kind()) {
            Some(TokenKind::Bang | TokenKind::Minus) => {
                let operator = self.tokens.next().unwrap();
                let right = self.unary()?;
                Box::new(Expr::Unary { operator, right })
            }
            _ => self.primary()?,
        };
        Ok(expr)
    }

    /// primary -> NUMBER | STRING | "true" | "false" | "nil"
    ///          | "(" expression ")" ;
    fn primary(&mut self) -> Result<Box<Expr<'tok>>, Error> {
        let token = self.tokens.peek();
        match token.map(|tok| tok.kind()) {
            Some(
                TokenKind::Keyword(Keyword::True)
                | TokenKind::Keyword(Keyword::False)
                | TokenKind::Keyword(Keyword::Nil)
                | TokenKind::Number
                | TokenKind::String,
            ) => Ok(Box::new(Expr::Literal {
                value: self.tokens.next().unwrap(),
            })),
            Some(TokenKind::LeftParen) => {
                self.tokens.next(); // consume left parenthesis
                let expression = self.expression()?;
                if let Some(TokenKind::RightParen) = self.tokens.next().map(|tok| tok.kind()) {
                    Ok(Box::new(Expr::Grouping { expression }))
                } else {
                    Err(Error::UnclosedParenthesis {
                        source_line: "FIXME".to_string(),
                        line_number: 1,
                        column_number: 1,
                    })
                }
            }
            Some(TokenKind::UnterminatedString) => Err(Error::UnterminatedString {
                source_line: "FIXME".to_string(),
                line_number: 1,
                column_number: 1,
            }),
            Some(TokenKind::UnterminatedBlockComment) => Err(Error::UnterminatedBlockComment {
                source_line: "FIXME".to_string(),
                line_number: 1,
                column_number: 1,
            }),
            _ => Err(Error::ParseError {
                source_line: "FIXME".to_string(),
                line_number: 1,
                column_number: 1,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::Scanner;

    use super::*;

    #[test]
    fn primaries() {
        let literals = vec!["true", "false", "nil", "123", "\"hello\"", "(1 + 2.0)"];
        let tokens = vec![
            vec![Token::new_keyword(0, "true")],
            vec![Token::new_keyword(0, "false")],
            vec![Token::new_keyword(0, "nil")],
            vec![Token::new_number(0, "123")],
            vec![Token::new_string(0, "\"hello\"")],
            vec![
                Token::new_number(1, "1"),
                Token::new_plus(3),
                Token::new_number(5, "2.0"),
            ],
        ];
        let primaries = vec![
            Expr::Literal {
                value: &tokens[0][0],
            },
            Expr::Literal {
                value: &tokens[1][0],
            },
            Expr::Literal {
                value: &tokens[2][0],
            },
            Expr::Literal {
                value: &tokens[3][0],
            },
            Expr::Literal {
                value: &tokens[4][0],
            },
            Expr::Grouping {
                expression: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: &tokens[5][0],
                    }),
                    operator: &tokens[5][1],
                    right: Box::new(Expr::Literal {
                        value: &tokens[5][2],
                    }),
                }),
            },
        ];
        for (source, expected) in literals.iter().zip(primaries) {
            let scanner = Scanner::new(source);
            let tokens = scanner.tokens();
            let mut parser = Parser::new(&tokens);
            let actual = parser.parse().unwrap();
            assert_eq!(*actual, expected);
        }
    }

    #[test]
    fn primary_unbalanced_parens() {
        let source = "(1 + 2";
        let scanner = Scanner::new(source);
        let tokens = scanner.tokens();
        let mut parser = Parser::new(&tokens);
        // FIXME:
        parser.parse().unwrap_err();
    }
}
