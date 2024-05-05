use crate::error::Error;
use std::iter::Peekable;
use std::slice::Iter;

use crate::expr::Expr;
use crate::token::Token;

pub struct Parser<'tok> {
    tokens: Peekable<Iter<'tok, Token<'tok>>>,
}

/// Recursive descent parser
impl<'tok> Parser<'tok> {
    pub fn new(tokens: &'tok [Token<'tok>]) -> Self {
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

        while let Some(Token::BangEqual | Token::EqualEqual) = self.tokens.peek() {
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

        while let Some(Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual) =
            self.tokens.peek()
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

        while let Some(Token::Minus | Token::Plus) = self.tokens.peek() {
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

        while let Some(Token::Slash | Token::Star) = self.tokens.peek() {
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
        let expr = match self.tokens.peek() {
            Some(Token::Bang | Token::Minus) => {
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
        match self.tokens.peek() {
            Some(Token::True | Token::False | Token::Nil | Token::Number(_) | Token::String(_)) => {
                Ok(Box::new(Expr::Literal {
                    value: self.tokens.next().unwrap(),
                }))
            }
            Some(Token::LeftParen) => {
                self.tokens.next(); // consume left parenthesis
                let expression = self.expression()?;
                if let Some(Token::RightParen) = self.tokens.next() {
                    Ok(Box::new(Expr::Grouping { expression }))
                } else {
                    Err(Error::ParseError {
                        source_line: "FIXME".to_string(),
                        line_number: 1,
                        column_number: 1,
                    })
                }
            }
            _ => Err(Error::ParseError {
                source_line: "FIXME 2".to_string(),
                line_number: 1,
                column_number: 1,
            }),
        }
    }
}
