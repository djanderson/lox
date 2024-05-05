#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(
        "invalid character, line {line_number}\n{source_line}\n{:->column_number$}",
        "^"
    )]
    InvalidCharacter {
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
    #[error(
        "unterminated comment, line {line_number}\n{source_line}\n{:->column_number$}",
        "^"
    )]
    UnterminatedComment {
        source_line: String,
        line_number: usize,
        column_number: usize,
    },
    #[error(
        "parse error, line {line_number}\n{source_line}\n{:->column_number$}",
        "^"
    )]
    ParseError {
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
    fn invalid_character() {
        let source_line = String::from("class @bad");
        let e = Error::InvalidCharacter {
            source_line,
            line_number: 1,
            column_number: 7,
        };
        let actual = format!("{e}");
        let expected = indoc! {r#"
            invalid character, line 1
            class @bad
            ------^"#
        };
        assert_eq!(actual, expected)
    }
}
