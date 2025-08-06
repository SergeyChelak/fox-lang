use std::cell::RefCell;
use std::rc::Rc;
use std::{
    fmt::{Debug, Display},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::fox::ast::FunctionStmt;
use crate::fox::class::ClassInstance;
use crate::fox::environment::{Environment, SharedEnvironmentPtr};
use crate::fox::{KEYWORD_THIS, Object};

/// Builtin function definition
///
pub type BuiltinFnBody = dyn Fn(&[Object]) -> Object;

#[derive(Clone)]
pub struct BuiltinFunc {
    pub body: Rc<BuiltinFnBody>,
    arity: usize,
}

impl Debug for BuiltinFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builtin func")
            .field("arity", &self.arity)
            .finish()
    }
}

impl std::hash::Hash for BuiltinFunc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.body).hash(state);
        self.arity.hash(state);
    }
}

impl Eq for BuiltinFunc {}

impl PartialEq for BuiltinFunc {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.body, &other.body) && self.arity == other.arity
    }
}

impl Display for BuiltinFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<builtin fun ({} args)>", self.arity())
    }
}

impl BuiltinFunc {
    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn clock() -> Self {
        let body = |_: &[Object]| -> Object {
            let time = SystemTime::now();
            let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
                println!("[ERROR] failed to calculate system time duration");
                return Object::Nil;
            };
            Object::Double(duration.as_secs() as f32)
        };
        Self {
            body: Rc::new(body),
            arity: 0,
        }
    }
}

/// Usual (language) function definition
///
#[derive(Clone)]
pub struct Func {
    pub decl: Rc<FunctionStmt>,
    pub closure: SharedEnvironmentPtr,
    pub is_initializer: bool,
}

impl Debug for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decl func")
            // .field("decl", &self.decl)
            // .field("closure", &self.closure)
            .field("is_initializer", &self.is_initializer)
            .finish()
    }
}

impl std::hash::Hash for Func {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.decl.hash(state);
        self.closure.as_ptr().hash(state);
        self.is_initializer.hash(state);
    }
}

impl Eq for Func {}

impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        self.decl == other.decl
            && Rc::ptr_eq(&self.closure, &other.closure)
            && self.is_initializer == other.is_initializer
    }
}

impl Display for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = if self.is_initializer { "init" } else { "fun" };
        write!(f, "<{id} ({} args)>", self.arity())
    }
}

impl Func {
    pub fn new(
        decl: Rc<FunctionStmt>,
        closure: SharedEnvironmentPtr,
        is_initializer: bool,
    ) -> Self {
        Self {
            decl,
            closure,
            is_initializer,
        }
    }

    pub fn arity(&self) -> usize {
        self.decl.params.len()
    }

    pub fn bind(&self, instance: Rc<RefCell<ClassInstance>>) -> Func {
        let mut env = Environment::with(Some(self.closure.clone()));
        env.define(KEYWORD_THIS, Object::Instance(instance));
        Func {
            decl: self.decl.clone(),
            closure: env.shared_ptr(),
            is_initializer: self.is_initializer,
        }
    }
}
