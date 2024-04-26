use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum LoxError {
    #[error(
        "invalid syntax, line {line_number}\n{source_line}\n{:->column_number$}",
        "^"
    )]
    InvalidSyntax {
        source_line: String,
        line_number: usize,
        column_number: usize,
    },
    #[error(
        "unterminated string, line {line_number}\n{source_line}\n{:->column_number$}",
        "^"
    )]
    UnterminatedString {
        source_line: String,
        line_number: usize,
        column_number: usize,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn invalid_syntax() {
        let source_line = String::from("class @bad");
        let e = LoxError::InvalidSyntax {
            source_line,
            line_number: 1,
            column_number: 7,
        };
        let actual = format!("{e}");
        let expected = indoc! {r#"
            invalid syntax, line 1
            class @bad
            ------^"#
        };
        assert_eq!(actual, expected)
    }
}
