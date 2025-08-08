use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::fox::{FoxError, FoxResult, utils::SharedPtr};

use super::{
    class::{ClassInstance, MetaClass},
    func::{BuiltinFunc, Func},
};

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

    pub fn as_meta_class(&self) -> FoxResult<Rc<MetaClass>> {
        match self {
            Object::Class(meta) => Ok(meta.clone()),
            _ => Err(FoxError::bug(&format!(
                "Expected MetaClass, found {self:?}"
            ))),
        }
    }

    pub fn as_class_instance(&self) -> FoxResult<SharedPtr<ClassInstance>> {
        match self {
            Object::Instance(obj) => Ok(obj.clone()),
            _ => Err(FoxError::bug(&format!(
                "Expected class instance, found {self:?}"
            ))),
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
