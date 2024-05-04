use std::fmt;

use crate::token::Token;

#[derive(Debug)]
pub enum Expr<'a> {
    Binary {
        left: Box<Expr<'a>>,
        operator: &'a Token<'a>,
        right: Box<Expr<'a>>,
    },
    Grouping {
        expression: Box<Expr<'a>>,
    },
    Literal {
        value: &'a Token<'a>,
    },
    Unary {
        operator: &'a Token<'a>,
        right: Box<Expr<'a>>,
    },
}

/// Display Expr in Polish notation.
///
/// E.g., "1 + 2" -> "(+ 1 2)"
impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({operator} {left} {right})"),
            Expr::Grouping { expression } => write!(f, "(group {expression})"),
            Expr::Literal { value } => write!(f, "{}", value),
            Expr::Unary { operator, right } => write!(f, "({} {})", operator, right),
        }
    }
}
