use std::{collections::HashMap, fmt::Display, rc::Rc};

pub const INITIALIZER_NAME: &str = "init";

use crate::fox::{
    FoxError, FoxResult,
    func::Func,
    object::*,
    token::Token,
    utils::{SharedPtr, fill_hash, mutable_cell},
};

/// MetaClass (functions)
///
#[derive(Debug, Clone)]
pub struct MetaClass {
    name: String,
    methods: HashMap<String, Func>,
}

pub struct Constructor {
    pub initializer: Option<Func>,
    pub instance: SharedPtr<ClassInstance>,
}

impl MetaClass {
    pub fn constructor(meta: Rc<Self>) -> Constructor {
        let instance = ClassInstance::new(meta.clone());
        let instance = mutable_cell(instance);
        let initializer = meta
            .find_method(INITIALIZER_NAME)
            .map(|init| init.bind(instance.clone()));
        Constructor {
            initializer,
            instance,
        }
    }

    pub fn new(name: &str, methods: HashMap<String, Func>) -> Self {
        Self {
            name: name.to_string(),
            methods,
        }
    }

    pub fn arity(&self) -> usize {
        let Some(method) = self.find_method(INITIALIZER_NAME) else {
            return 0;
        };
        method.arity()
    }

    fn find_method(&self, name: &str) -> Option<Func> {
        self.methods.get(name).cloned()
    }
}

impl Display for MetaClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "meta class {}", self.name)
    }
}

impl std::hash::Hash for MetaClass {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        fill_hash(&self.methods, state);
    }
}

/// Class instance (data only)
///
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

    pub fn get(instance_ref: SharedPtr<Self>, name: &Token) -> FoxResult<Object> {
        let lexeme = &name.lexeme;
        if let Some(obj) = instance_ref.borrow().fields.get(lexeme).cloned() {
            return Ok(obj);
        };

        if let Some(method) = instance_ref
            .borrow()
            .meta_class_ref
            .find_method(&name.lexeme)
        {
            return Ok(Object::Callee(method.bind(instance_ref.clone())));
        }

        let err = FoxError::runtime(
            Some(name.clone()),
            &format!("Undefined property '{lexeme}'"),
        );
        Err(err)
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}

impl std::hash::Hash for ClassInstance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.meta_class_ref.hash(state);
        fill_hash(&self.fields, state);
    }
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "class '{}'", self.meta_class_ref.name)
    }
}
