use std::{cmp::min, fmt};

use crate::source::Span;

#[derive(Clone, PartialEq)]
pub struct Token {
    /// The type of the token.
    kind: TokenKind,
    /// The start and end positions of the token in the source code.
    span: Span,
    /// The first 7 bytes of the lexeme. This provides a nice [`Display`] repr
    /// for tokens and expressions.
    view: [u8; 7],
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .field("view", &format_args!("\"{}\"", self))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexView<'a> {
    Partial(&'a str),
    Complete(&'a str),
}

impl Token {
    pub fn new(kind: TokenKind, pos: usize, lexeme: &str) -> Self {
        let mut view = [0; 7];
        let len = lexeme.as_bytes().len();
        let view_len = min(len, 7);
        view[..view_len].copy_from_slice(&lexeme.as_bytes()[..view_len]);
        let span = Span {
            start: pos as u32,
            end: (pos + len) as u32,
        };
        Self { kind, span, view }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Reads the view and determines if it's a partial or complete view.
    pub fn view(&self) -> LexView {
        let len = (self.span.end - self.span.start) as usize;
        let view_len = min(len, 7);
        // SAFETY: view_len is validated to be <= 7
        let view = unsafe { str::from_utf8_unchecked(&self.view[..view_len]) };
        if len <= 7 {
            LexView::Complete(view)
        } else {
            LexView::Partial(view)
        }
    }

    /// Returns the full lexeme for the token.
    ///
    /// Panics if this token is not from the provided source.
    pub fn lexeme<'a>(&'a self, source: &'a str) -> &'a str {
        match self.view() {
            LexView::Complete(view) => view,
            LexView::Partial(_) => &source[(self.span.start as usize)..(self.span.end as usize)],
        }
    }

    pub fn new_line_comment(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::LineComment, pos, lexeme)
    }

    pub fn new_block_comment(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::BlockComment, pos, lexeme)
    }

    pub fn new_left_paren(pos: usize) -> Self {
        Token::new(TokenKind::LeftParen, pos, "(")
    }

    pub fn new_right_paren(pos: usize) -> Self {
        Token::new(TokenKind::RightParen, pos, ")")
    }

    pub fn new_left_brace(pos: usize) -> Self {
        Token::new(TokenKind::LeftBrace, pos, "{")
    }

    pub fn new_right_brace(pos: usize) -> Self {
        Token::new(TokenKind::RightBrace, pos, "}")
    }

    pub fn new_comma(pos: usize) -> Self {
        Token::new(TokenKind::Comma, pos, ",")
    }

    pub fn new_dot(pos: usize) -> Self {
        Token::new(TokenKind::Dot, pos, ".")
    }

    pub fn new_minus(pos: usize) -> Self {
        Token::new(TokenKind::Minus, pos, "-")
    }

    pub fn new_plus(pos: usize) -> Self {
        Token::new(TokenKind::Plus, pos, "+")
    }

    pub fn new_semicolon(pos: usize) -> Self {
        Token::new(TokenKind::Semicolon, pos, ";")
    }

    pub fn new_slash(pos: usize) -> Self {
        Token::new(TokenKind::Slash, pos, "/")
    }

    pub fn new_star(pos: usize) -> Self {
        Token::new(TokenKind::Star, pos, "*")
    }

    pub fn new_bang(pos: usize) -> Self {
        Token::new(TokenKind::Bang, pos, "!")
    }

    pub fn new_bang_equal(pos: usize) -> Self {
        Token::new(TokenKind::BangEqual, pos, "!=")
    }

    pub fn new_equal(pos: usize) -> Self {
        Token::new(TokenKind::Equal, pos, "=")
    }

    pub fn new_equal_equal(pos: usize) -> Self {
        Token::new(TokenKind::EqualEqual, pos, "==")
    }

    pub fn new_greater(pos: usize) -> Self {
        Token::new(TokenKind::Greater, pos, ">")
    }

    pub fn new_greater_equal(pos: usize) -> Self {
        Token::new(TokenKind::GreaterEqual, pos, ">=")
    }

    pub fn new_less(pos: usize) -> Self {
        Token::new(TokenKind::Less, pos, "<")
    }

    pub fn new_less_equal(pos: usize) -> Self {
        Token::new(TokenKind::LessEqual, pos, "<=")
    }

    pub fn new_identifier(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::Identifier, pos, lexeme)
    }

    pub fn new_string(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::String, pos, lexeme)
    }

    pub fn new_number(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::Number, pos, lexeme)
    }

    /// Panics if `lexeme` is not a valid keyword.
    pub fn new_keyword(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::Keyword(Keyword::from(lexeme)), pos, lexeme)
    }

    pub fn new_unterminated_block_comment(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::UnterminatedBlockComment, pos, lexeme)
    }

    pub fn new_unterminated_string(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::UnterminatedString, pos, lexeme)
    }

    pub fn new_invalid_character(pos: usize, lexeme: &str) -> Self {
        Token::new(TokenKind::InvalidCharacter, pos, lexeme)
    }

    /// Return true if the token is invalid.
    pub fn is_invalid(&self) -> bool {
        matches!(self.kind, TokenKind::InvalidCharacter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

impl From<&str> for Keyword {
    fn from(value: &str) -> Self {
        match value {
            "and" => Self::And,
            "class" => Self::Class,
            "else" => Self::Else,
            "false" => Self::False,
            "fun" => Self::Fun,
            "for" => Self::For,
            "if" => Self::If,
            "nil" => Self::Nil,
            "or" => Self::Or,
            "print" => Self::Print,
            "return" => Self::Return,
            "super" => Self::Super,
            "this" => Self::This,
            "true" => Self::True,
            "var" => Self::Var,
            "while" => Self::While,
            _ => unimplemented!("invalid keyword {value}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    /// Line `// comment`.
    LineComment,
    /// Block `/* comment */`.
    BlockComment,
    // Single-character tokens.
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `-`
    Minus,
    /// `+`
    Plus,
    /// `;`
    Semicolon,
    /// `/`
    Slash,
    /// `*`
    Star,
    // One or two character tokens.
    /// `!`
    Bang,
    /// `!=`
    BangEqual,
    /// `=`
    Equal,
    /// `==`
    EqualEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    // Literals.
    /// An identifier, like `idx`.
    Identifier,
    /// A raw UTF-8 string literal in double quotes, like `"Hello, world!"`.
    String,
    /// A number literal, like `123` or `1.5`.
    Number,
    /// A reserved keyword, like `class`.
    Keyword(Keyword),
    // Invalid tokens.
    /// An unterminated block comment.
    UnterminatedBlockComment,
    /// An unterminated string.
    UnterminatedString,
    /// An invalid character.
    InvalidCharacter,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.view() {
            LexView::Complete(view) => f.write_str(view),
            LexView::Partial(view) => write!(f, "{view}…"),
        }
    }
}
