use std::fmt;
use util::Str;

pub type ErrorCode = u16;

#[derive(Clone)]
pub enum Exception {
    Arity(usize, usize),
    Signature(Str, Str),
    Custom(ErrorCode, Str),
    Undefined(Str),
    Syntax(ErrorCode, Str),
}

use self::Exception::*;

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arity(expected, found) => {
                write!(f, "arity mismatch: expected {}, found {}", expected, found)
            }
            Signature(expected, found) => write!(
                f,
                "signature mismatch: expected {}, found {}",
                expected, found
            ),
            Custom(_, err) => write!(f, "{}", err),
            Undefined(symbol) => write!(f, "undefined symbol: `{}`", symbol),
            Syntax(_, desc) => write!(f, "syntax error: {}", desc),
        }
    }
}

impl Exception {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            Arity(..) => 4,
            Signature(..) => 9,
            Custom(code, ..) => *code,
            Undefined(..) => 1,
            Syntax(code, ..) => *code,
        }
    }
}
