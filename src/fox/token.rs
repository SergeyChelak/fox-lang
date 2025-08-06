use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::fox::{FoxError, FoxResult, ast::FunctionStmt, environment::SharedEnvironmentPtr};

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

pub type BuiltInFunction = dyn Fn(&[Object]) -> Object;

#[derive(Clone)]
pub enum Func {
    Builtin {
        body: Rc<BuiltInFunction>,
        arity: usize,
    },
    Declaration {
        decl: Box<FunctionStmt>,
        closure: SharedEnvironmentPtr,
    },
}

impl Debug for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin { arity, .. } => f
                .debug_struct("Builtin func")
                .field("arity", arity)
                .finish(),
            Self::Declaration { .. } => f.debug_struct("Decl func").finish(),
        }
    }
}

impl std::hash::Hash for Func {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Func::*;
        match self {
            Builtin { body, arity } => {
                0.hash(state);
                Rc::as_ptr(body).hash(state);
                arity.hash(state);
            }
            Declaration { decl, closure } => {
                1.hash(state);
                decl.hash(state);
                closure.as_ptr().hash(state);
            }
        }
    }
}

impl Eq for Func {}

impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Builtin {
                    body: l_body,
                    arity: l_arity,
                },
                Self::Builtin {
                    body: r_body,
                    arity: r_arity,
                },
            ) => Rc::ptr_eq(l_body, r_body) && l_arity == r_arity,
            (
                Self::Declaration {
                    decl: l_decl,
                    closure: l_closure,
                },
                Self::Declaration {
                    decl: r_decl,
                    closure: r_closure,
                },
            ) => l_decl == r_decl && Rc::ptr_eq(l_closure, r_closure),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct MetaClass {
    name: String,
}

impl MetaClass {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Display for MetaClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "meta class {}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct ClassInstance {
    meta_class_ref: Rc<MetaClass>,
    fields: HashMap<String, Object>,
}

impl ClassInstance {
    pub fn new(meta_class_ref: Rc<MetaClass>) -> Self {
        Self {
            meta_class_ref,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> FoxResult<Object> {
        let lexeme = &name.lexeme;
        let Some(obj) = self.fields.get(lexeme).cloned() else {
            let err = FoxError::runtime(
                Some(name.clone()),
                &format!("Undefined property '{}'", lexeme),
            );
            return Err(err);
        };
        Ok(obj)
    }
}

impl std::hash::Hash for ClassInstance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.meta_class_ref.hash(state);
        let mut keys: Vec<_> = self.fields.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(state);
            self.fields[key].hash(state);
        }
    }
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "class '{}'", self.meta_class_ref.name)
    }
}

#[derive(Clone, Debug)]
pub enum Object {
    Nil,
    Double(f32),
    Text(String),
    Bool(bool),
    Callee(Func),
    Class(Rc<MetaClass>),
    Instance(ClassInstance),
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
            Class(val) => {
                5.hash(state);
                val.hash(state);
            }
            Instance(val) => {
                6.hash(state);
                val.hash(state);
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
            Self::Callee(value) => write!(f, "{value}"),
            Self::Class(value) => write!(f, "class {value}"),
            Self::Instance(value) => write!(f, "instance of {value}"),
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
        Self::Builtin {
            body: Rc::new(body),
            arity: 0,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Func::Builtin { arity, .. } => *arity,
            Func::Declaration { decl, .. } => decl.params.len(),
        }
    }

    fn is_builtin(&self) -> bool {
        matches!(self, Func::Builtin { .. })
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
