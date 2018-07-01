use expression::Expression;
use im::HashMap;
use std::borrow::Borrow;

pub struct Context {
    bindings: HashMap<String, Expression>,
    prev: Option<Box<Context>>,
}

impl Context {
    fn new() -> Context {
        Context {
            bindings: HashMap::new(),
            prev: None,
        }
    }

    /// TODO Don't clone the value after getting it
    fn get(&self, key: impl AsRef<str>) -> Option<Expression> {
        self.bindings.get(key.as_ref()).map(|expr| (*expr).clone())
    }

    fn insert(self, ident: impl ToString, value: Expression) -> Context {
        Context {
            bindings: self.bindings.insert(ident.to_string(), value),
            prev: self.prev,
        }
    }

    fn new_scope(self) -> Context {
        Context {
            bindings: HashMap::new(),
            prev: Some(Box::new(self)),
        }
    }
}
