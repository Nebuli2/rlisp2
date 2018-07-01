use expression::Expression;
use std::collections::HashMap;
use std::borrow::Borrow;
use std::mem::replace;

type Scope = HashMap<String, Expression>;

pub struct Context {
    scopes: Vec<Scope>
}

impl Context {
    pub fn new() -> Context {
        Context {
            scopes: vec![HashMap::new()]
        }
    }

    /// TODO Don't clone the value after getting it
    pub fn get(&self, key: impl AsRef<str>) -> Option<&Expression> {
        let key = key.as_ref();
        self.scopes.iter()
            .filter_map(|scope| scope.get(key))
            .next()
    }

    pub fn insert(&mut self, ident: impl ToString, value: Expression) {
        let ident = ident.to_string();
        self.scopes.last_mut().map(|scope| scope.insert(ident, value));
    }

    pub fn ascend_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Consumes the context, producing the previous context if present, or an 
    /// empty context.
    pub fn descend_scope(&mut self) {
        self.scopes.pop();
    }
}
