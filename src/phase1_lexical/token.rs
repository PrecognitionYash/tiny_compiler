use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Integer,
    Def,
    Return,
    If,
    Else,
    For,
    In,
    Colon,
    Comma,
    LParen,
    RParen,
    Newline,
    Indent,
    Dedent,
    Range,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Amp,
    Pipe,
    Caret,
    ShiftLeft,
    ShiftRight,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Identifier => "Identifier",
            Self::Integer => "Integer",
            Self::Def => "def",
            Self::Return => "return",
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::In => "in",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::Newline => "NEWLINE",
            Self::Indent => "INDENT",
            Self::Dedent => "DEDENT",
            Self::Range => "..",
            Self::Assign => "=",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Amp => "&",
            Self::Pipe => "|",
            Self::Caret => "^",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::EqualEqual => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Eof => "EOF",
        };
        write!(f, "{}", label)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            line,
            column,
        }
    }
}
