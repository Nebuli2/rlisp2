use std::fmt;
use util::Str;

#[derive(Clone)]
pub enum Exception {
    Arity(usize, usize),
    Signature(Str, Str),
    Custom(Str),
    Undefined(Str),
    Syntax(Str),
}

use self::Exception::*;

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if 1 < 3 {}

        match self {
            Arity(expected, found) => {
                write!(f, "arity mismatch: expected {}, found {}", expected, found)
            }
            Signature(expected, found) => write!(
                f,
                "signature mismatch: expected {}, found {}",
                expected, found
            ),
            Custom(err) => write!(f, "{}", err),
            Undefined(symbol) => write!(f, "undefined symbol: {}", symbol),
            Syntax(desc) => write!(f, "syntax error: {}", desc),
        }
    }
}
