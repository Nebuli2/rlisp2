use environment::Environment;
use expression::Expression;
use std::collections::HashMap;

type Scope = HashMap<String, Expression>;

pub struct Context {
    scopes: Vec<Scope>,
    indents: usize,
}

impl Context {
    pub fn new() -> Context {
        Context {
            scopes: vec![HashMap::new()],
            indents: 0,
        }
    }

    /// TODO Don't clone the value after getting it

    pub fn insert(&mut self, ident: impl ToString, value: Expression) {
        let ident = ident.to_string();
        self.scopes
            .last_mut()
            .map(|scope| scope.insert(ident, value));
    }

    pub fn indents(&self) -> usize {
        self.indents
    }

    pub fn indent(&mut self) {
        self.indents += 1;
    }

    pub fn unindent(&mut self) {
        self.indents -= 1;
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

impl Environment for Context {
    fn get(&self, key: impl AsRef<str>) -> Option<&Expression> {
        let key = key.as_ref();

        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.get(key))
            .next()
    }
}
