//! This module provides the `Context` struct for storing data during the
//! evaluation of expressions.

use expression::Expression;
use std::collections::HashMap;

/// The ID of an rlisp struct type.
type StructId = usize;

/// An individual scope in the evaluation context.
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

/// Represents the evaluation context for use during the evaluation of rlisp
/// expressions. It provides a means of accessing stored variables and
/// information about custom struct types.
pub struct Context {
    scopes: Vec<Scope>,
    struct_count: usize,
}

impl Default for Context {
    fn default() -> Context {
        Context::new()
    }
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
        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.bindings.get(key.as_ref()))
            .next()
    }

    /// Attempts to retrieve a mutable reference to the value stored at the
    /// specified key in the `Context`.
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut Expression> {
        self.scopes
            .iter_mut()
            .rev()
            .filter_map(|scope| scope.bindings.get_mut(key.as_ref()))
            .next()
    }

    /// Inserts the specified value into the `Context` at the current scope.
    pub fn insert(&mut self, ident: impl ToString, value: Expression) {
        let ident = ident.to_string();
        self.scopes
            .last_mut()
            .map(|scope| scope.bindings.insert(ident, value));
    }

    /// Defines a struct with the specified name in the `Context`. If the
    /// scopes of the `Context` are empty, `None` is returned. Otherwise, a
    /// `StructId` is returned.
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

    /// Looks up the `StructId` of the struct with the specified name in the
    /// `Context`.
    pub fn get_struct_id(&self, name: impl AsRef<str>) -> Option<StructId> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.structs.get(name.as_ref()))
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
