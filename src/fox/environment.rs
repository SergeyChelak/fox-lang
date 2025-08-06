use std::collections::HashMap;

use crate::fox::{FoxError, FoxResult, mutable_cell, token::Token, utils::SharedPtr};

use super::Object;

pub type SharedEnvironmentPtr = SharedPtr<Environment>;

#[derive(Debug)]
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
        mutable_cell(self)
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

    pub fn get_at(&self, distance: usize, name: &str) -> FoxResult<Object> {
        let value = if distance == 0 {
            self.values.get(name).cloned()
        } else {
            let enclosing = self.traverse_enclosing(distance)?;
            enclosing.borrow().values.get(name).cloned()
        };
        let Some(obj) = value else {
            let err = FoxError::resolver(None, "Object not found");
            return Err(err);
        };
        Ok(obj)
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Object) -> FoxResult<()> {
        let insert_data =
            |map: &mut HashMap<String, Object>| map.insert(name.lexeme.clone(), value);

        if distance == 0 {
            insert_data(&mut self.values);
        } else {
            let enclosing = self.traverse_enclosing(distance)?;
            let map = &mut enclosing.borrow_mut().values;
            insert_data(map);
        }
        Ok(())
    }

    fn traverse_enclosing(&self, depth: usize) -> FoxResult<SharedEnvironmentPtr> {
        if depth == 0 {
            let err = FoxError::resolver(
                None,
                "Zero depth isn't applicable for ancestor environments",
            );
            return Err(err);
        }
        let mut ptr = self.enclosing.clone();
        for _ in 1..depth {
            let Some(current) = ptr else {
                let err = FoxError::resolver(None, "Invalid depth: Ancestor environment not found");
                return Err(err);
            };
            ptr = current.borrow().enclosing.clone();
        }

        let Some(env) = ptr else {
            let err = FoxError::resolver(
                None,
                "Invalid depth: Environment at specified distance not found",
            );
            return Err(err);
        };
        Ok(env)
    }
}
