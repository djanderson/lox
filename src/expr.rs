use std::fmt;

use crate::token::Token;

#[derive(Debug)]
pub enum Expr<'a> {
    Binary {
        left: Box<Expr<'a>>,
        operator: &'a Token,
        right: Box<Expr<'a>>,
    },
    Grouping {
        expression: Box<Expr<'a>>,
    },
    Literal {
        value: &'a Token,
    },
    Unary {
        operator: &'a Token,
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
            Expr::Literal { value } => write!(f, "{value}"),
            Expr::Unary { operator, right } => write!(f, "({operator} {right})"),
        }
    }
}

impl PartialEq for Expr<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Expr::Binary {
                    left,
                    operator,
                    right,
                },
                Expr::Binary {
                    left: other_left,
                    operator: other_operator,
                    right: other_right,
                },
            ) => left == other_left && operator == other_operator && right == other_right,
            (
                Expr::Grouping { expression },
                Expr::Grouping {
                    expression: other_expression,
                },
            ) => expression == other_expression,
            (Expr::Literal { value }, Expr::Literal { value: other_value }) => value == other_value,
            (
                Expr::Unary { operator, right },
                Expr::Unary {
                    operator: other_operator,
                    right: other_right,
                },
            ) => operator == other_operator && right == other_right,
            _ => false,
        }
    }
}
