use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::fox::{FoxError, FoxResult, object::*, token::Token};

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
                &format!("Undefined property '{lexeme}'"),
            );
            return Err(err);
        };
        Ok(obj)
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        self.fields.insert(name.lexeme.clone(), value);
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
