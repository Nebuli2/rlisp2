use expression::Expression;
use std::collections::HashMap;

type Scope = HashMap<String, Expression>;

pub struct Context {
    scopes: Vec<Scope>,
}

impl Context {
    /// Produces an empty `Context`.
    pub fn new() -> Context {
        Context {
            scopes: vec![HashMap::new()],
        }
    }

    /// Attempts to retrieve the value stored at the specified key in the
    /// `Context`.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&Expression> {
        let key = key.as_ref();

        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.get(key))
            .next()
    }

    /// Inserts the specified value into the `Context` at the current scope.
    pub fn insert(&mut self, ident: impl ToString, value: Expression) {
        let ident = ident.to_string();
        self.scopes
            .last_mut()
            .map(|scope| scope.insert(ident, value));
    }

    /// Ascends one level of scope.
    pub fn ascend_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Descends one level of scope, dropping all values in the dropped scopes.
    pub fn descend_scope(&mut self) {
        self.scopes.pop();
    }
}
