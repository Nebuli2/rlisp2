use expression::Expression;
use std::collections::HashMap;

// type Scope = HashMap<String, Expression>;
type StructId = usize;

struct Scope {
    bindings: HashMap<String, Expression>,
    structs: HashMap<String, StructId>,
}

impl Default for Scope {
    fn default() -> Scope {
        Scope {
            bindings: HashMap::new(),
            structs: HashMap::new(),
        }
    }
}

pub struct Context {
    scopes: Vec<Scope>,
    struct_count: usize,
}

impl Context {
    /// Produces an empty `Context`.
    pub fn new() -> Context {
        Context {
            scopes: vec![Scope::default()],
            struct_count: 0,
        }
    }

    /// Attempts to retrieve the value stored at the specified key in the
    /// `Context`.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&Expression> {
        let key = key.as_ref();

        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.bindings.get(key))
            .next()
    }

    /// Inserts the specified value into the `Context` at the current scope.
    pub fn insert(&mut self, ident: impl ToString, value: Expression) {
        let ident = ident.to_string();
        self.scopes
            .last_mut()
            .map(|scope| scope.bindings.insert(ident, value));
    }

    pub fn define_struct(&mut self, name: impl ToString) -> Option<StructId> {
        if let Some(scope) = self.scopes.last_mut() {
            self.struct_count += 1;
            let id = self.struct_count;
            scope.structs.insert(name.to_string(), id);
            Some(id)
        } else {
            None
        }
    }

    pub fn get_struct_id(&self, name: impl ToString) -> Option<StructId> {
        let key = name.to_string();

        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.structs.get(&key))
            .next()
            .map(Clone::clone)
    }

    /// Ascends one level of scope.
    pub fn ascend_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    /// Descends one level of scope, dropping all values in the dropped scopes.
    pub fn descend_scope(&mut self) {
        self.scopes.pop();
    }
}
