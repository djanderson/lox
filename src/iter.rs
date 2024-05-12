//! Peekable line and column number tracking iterator.
//!
//! Inspired by https://github.com/serde-rs/json/blob/master/src/iter.rs.

#[derive(Debug, Clone)]
pub struct PeekableLineColIterator<I>
where
    I: Iterator<Item = char>,
{
    iter: I,

    /// Index of the current line. Characters in the first line of the input
    /// (before the first newline character) are in line 1.
    line: usize,

    /// Index of the current column. The first character in the input and any
    /// characters immediately following a newline character are in column 1.
    /// The column is 0 immediately after a newline character has been read.
    column: usize,

    /// Byte offset of the start of the current line. This is the sum of lengths
    /// of all previous lines. Keeping track of things this way allows efficient
    /// computation of the current line, column, and byte offset while only
    /// updating one of the counters in `next()` in the common case.
    start_of_line: usize,

    /// Peeked character.
    peeked: Option<char>,
}

impl<I> PeekableLineColIterator<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            line: 1,
            column: 0,
            start_of_line: 0,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        if self.peeked.is_none() {
            // Take next from inner iter to avoid incrementing col
            self.peeked = self.iter.next();
        }
        self.peeked.as_ref()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    /// Offset is the iterator's character index + 1
    pub fn offset(&self) -> usize {
        self.start_of_line + self.column
    }
}

impl<I> Iterator for PeekableLineColIterator<I>
where
    I: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.peeked.take().or_else(|| self.iter.next())? {
            '\n' => {
                self.start_of_line += self.column + 1;
                self.line += 1;
                self.column = 0;
                Some('\n')
            }
            c => {
                self.column += 1;
                Some(c)
            }
        }
    }
}
