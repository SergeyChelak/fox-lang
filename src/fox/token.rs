use std::{
    cmp::Ordering,
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::fox::ast::FunctionStmt;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Object,
    pub code_location: CodeLocation,
}

impl Token {
    pub fn is_eof(&self) -> bool {
        matches!(self.token_type, TokenType::Eof)
    }
}

#[derive(Debug, Clone)]
pub enum Func {
    Builtin {
        body: fn(&[Object]) -> Object,
        arity: usize,
    },
    Declaration(Box<FunctionStmt>),
}

#[derive(Debug, Clone)]
pub enum Object {
    Nil,
    Double(f32),
    Text(String),
    Bool(bool),
    Callee(Func),
}

impl Object {
    pub fn is_true(&self) -> bool {
        match self {
            Object::Nil => false,
            Object::Bool(value) => *value,
            _ => true,
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        use Object::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Double(l), Double(r)) => l.partial_cmp(r) == Some(Ordering::Equal),
            (Text(l), Text(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            (Callee(_), Callee(_)) => {
                println!("[WARN] Callee comparison isn't implemented");
                false
            }
            _ => false,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Double(value) => write!(f, "{value}"),
            Self::Text(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Callee(value) => write!(f, "{value}"),
        }
    }
}

impl Display for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<")?;
        if self.is_builtin() {
            write!(f, "builtin ")?;
        }
        write!(f, "fun ({} args)", self.arity())?;
        write!(f, ">")
    }
}

impl Func {
    pub fn clock() -> Self {
        let body = |_: &[Object]| -> Object {
            let time = SystemTime::now();
            let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
                println!("[ERROR] failed to calculate system time duration");
                return Object::Nil;
            };
            Object::Double(duration.as_secs() as f32)
        };
        Self::Builtin { body, arity: 0 }
    }

    pub fn arity(&self) -> usize {
        match self {
            Func::Builtin { arity, .. } => *arity,
            Func::Declaration(stmt) => stmt.params.len(),
        }
    }

    fn is_builtin(&self) -> bool {
        matches!(self, Func::Builtin { .. })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
pub struct CodeLocation {
    line: usize,
    abs_position: usize,
}

impl CodeLocation {
    pub fn new(line: usize, abs_position: usize) -> Self {
        Self { line, abs_position }
    }

    pub fn line_number(&self) -> usize {
        self.line
    }

    pub fn absolute_position(&self) -> usize {
        self.abs_position
    }
}

impl Default for CodeLocation {
    fn default() -> Self {
        Self {
            line: 1,
            abs_position: 0,
        }
    }
}
