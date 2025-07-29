use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::fox::{FoxError, FoxResult, token::Token};

use super::Object;

pub type SharedReference<T> = Rc<RefCell<T>>;
pub type SharedEnvironmentPtr = SharedReference<Environment>;

pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<SharedEnvironmentPtr>,
}

impl Environment {
    pub fn new() -> Self {
        Self::with(None)
    }

    pub fn with(enclosing: Option<SharedEnvironmentPtr>) -> Self {
        Self {
            values: Default::default(),
            enclosing,
        }
    }

    pub fn shared_ptr(self) -> SharedEnvironmentPtr {
        Rc::new(RefCell::new(self))
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> FoxResult<()> {
        let key = &name.lexeme;
        if self.values.contains_key(key) {
            self.define(key, value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(FoxError::token(
            super::ErrorKind::UndefinedVariable(key.clone()),
            Some(name.clone()),
        ))
    }

    pub fn define(&mut self, name: &str, object: Object) {
        self.values.insert(name.to_string(), object);
    }

    pub fn get(&self, token: &Token) -> FoxResult<Object> {
        let mut obj = self.values.get(&token.lexeme).cloned();

        if let Some(enclosing) = &self.enclosing
            && obj.is_none()
        {
            let value = enclosing.borrow().get(token)?;
            obj = Some(value);
        }

        let Some(obj) = obj else {
            let err = FoxError::token(
                crate::fox::ErrorKind::UndefinedVariable(token.lexeme.clone()),
                Some(token.clone()),
            );
            return Err(err);
        };
        Ok(obj)
    }
}
