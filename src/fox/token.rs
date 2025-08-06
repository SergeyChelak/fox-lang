use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::fox::{
    class::{ClassInstance, MetaClass},
    func::{BuiltinFunc, Func},
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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

#[derive(Clone, Debug)]
pub enum Object {
    Nil,
    Double(f32),
    Text(String),
    Bool(bool),
    BuiltinCallee(BuiltinFunc),
    Callee(Func),
    Class(Rc<MetaClass>),
    Instance(Rc<RefCell<ClassInstance>>),
}

impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Object::*;
        match self {
            Nil => 0.hash(state),
            Double(val) => {
                1.hash(state);
                val.to_bits().hash(state);
            }
            Text(val) => {
                2.hash(state);
                val.hash(state);
            }
            Bool(val) => {
                3.hash(state);
                val.hash(state);
            }
            Callee(val) => {
                4.hash(state);
                val.hash(state);
            }
            BuiltinCallee(val) => {
                5.hash(state);
                val.hash(state);
            }
            Class(val) => {
                6.hash(state);
                val.hash(state);
            }
            Instance(val) => {
                7.hash(state);
                val.borrow().hash(state);
            }
        }
    }
}

impl std::cmp::Eq for Object {}

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
            (Double(l), Double(r)) => l == r,
            (Text(l), Text(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            (Callee(l), Callee(r)) => l == r,
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
            Self::BuiltinCallee(value) => write!(f, "{value}"),
            Self::Callee(value) => write!(f, "{value}"),
            Self::Class(value) => write!(f, "class {value}"),
            Self::Instance(value) => write!(f, "instance of {}", value.borrow()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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
