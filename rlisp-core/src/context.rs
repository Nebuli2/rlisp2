//! This module provides the `Context` struct for storing data during the
//! evaluation of expressions.

use crate::expression::Expression;
use crate::util::Str;
use std::collections::{HashMap, HashSet};

/// The ID of an rlisp struct type.
type StructId = usize;

/// An individual scope in the evaluation context.
#[derive(Debug)]
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

use rand::prelude::*;

/// Represents the evaluation context for use during the evaluation of rlisp
/// expressions. It provides a means of accessing stored variables and
/// information about custom struct types.
#[derive(Debug)]
pub struct Context {
    scopes: Vec<Scope>,
    struct_count: usize,
    rng: ThreadRng,
    read_files: HashSet<Str>,
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
            rng: thread_rng(),
            read_files: HashSet::new(),
        }
    }

    pub fn rng(&mut self) -> &mut impl Rng {
        &mut self.rng
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
    pub fn insert(
        &mut self,
        ident: impl ToString,
        value: impl Into<Expression>,
    ) {
        let ident = ident.to_string();
        self.scopes
            .last_mut()
            .map(|scope| scope.bindings.insert(ident, value.into()));
    }

    pub fn remove(&mut self, ident: impl AsRef<str>) {
        let ident = ident.as_ref();
        self.scopes
            .last_mut()
            .map(|scope| scope.bindings.remove(ident));
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

    pub fn get_cur_file(&self) -> Option<Str> {
        self.get("__FILE__").and_then(|ex| match ex {
            Expression::Str(s) => Some(s.clone()),
            _ => None,
        })
    }

    pub fn add_file(&mut self, file_name: Str) {
        self.read_files.insert(file_name);
    }

    pub fn remove_file(&mut self, file_name: &Str) {
        self.read_files.remove(file_name);
    }

    pub fn has_read_file(&self, file_name: &Str) -> bool {
        self.read_files.contains(file_name)
    }
}
