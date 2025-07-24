use std::collections::HashMap;

use crate::fox::{FoxError, FoxResult, token::Token};

use super::Object;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: Default::default(),
        }
    }

    pub fn define(&mut self, name: &str, object: Object) {
        self.values.insert(name.to_string(), object);
    }

    pub fn get(&self, token: &Token) -> FoxResult<Object> {
        let Some(obj) = self.values.get(&token.lexeme).cloned() else {
            let err = FoxError::token(
                crate::fox::ErrorKind::UndefinedVariable(token.lexeme.clone()),
                Some(token.clone()),
            );
            return Err(err);
        };
        Ok(obj)
    }
}
