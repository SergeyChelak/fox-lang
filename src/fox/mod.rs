mod error;
mod scanner;
pub use error::*;

type Source = [u8];

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Object,
    code_location: CodeLocation,
}

impl Token {
    pub fn is_eof(&self) -> bool {
        matches!(self.token_type, TokenType::Eof)
    }
}

pub enum Object {
    Empty,
    Double(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum TokenType {
    // single-character tokens
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // 1 or 2 chars tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // literals
    Identifier,
    String,
    Number,
    // Keywords
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
    //
    Eof,
}

pub struct CodeLocation {
    line: usize,
    abs_position: usize,
}

impl CodeLocation {
    pub fn new(line: usize, abs_position: usize) -> Self {
        Self { line, abs_position }
    }
}
