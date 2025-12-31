use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Ident,
    Variable,
    Number,
    String,

    Equals,
    Colon,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,

    Newline,
    Comment,
    Directive,
    Arithmetic,
    Eof,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenType,
    pub value: String,
    pub line: usize,
    pub col: usize,
    pub start_pos: usize,
}

impl Token {
    pub fn new(
        kind: TokenType,
        value: impl Into<String>,
        line: usize,
        col: usize,
        start_pos: usize,
    ) -> Self {
        Self {
            kind,
            value: value.into(),
            line,
            col,
            start_pos,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token({:?}, {:?}, L{})",
            self.kind, self.value, self.line
        )
    }
}
