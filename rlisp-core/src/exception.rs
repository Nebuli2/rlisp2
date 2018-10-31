//! The `exception` module deals with handling exceptions for the rlisp
//! language. As of now, 5 different exception varieties are offered:
//! * `Arity`
//! * `Signature`
//! * `Undefined`
//! * `Syntax`
//! * `Custom`

use std::fmt;
use util::Str;

/// An `ErrorCode` is a numerical representation of each individual type of
/// error.
pub type ErrorCode = u16;

/// The `Exception` type represents all possible exceptions in the rlisp
/// language.
#[derive(Clone)]
pub enum Exception {
    /// An arity mismatch exception, where the number of arguments
    /// provided to a function does not match the number of arguments expected.
    Arity(usize, usize),

    /// A type signature mismatch exception, where the types of the
    /// arguments provided to a function do not match the types expected.
    Signature(Str, Str),

    /// Any exception that does not fit into the other categories.
    Custom(ErrorCode, Str),

    /// RAn exception wherein the specified symbol is undefined.
    Undefined(Str),

    /// An arbitrary syntax exception.
    Syntax(ErrorCode, Str),
}

use self::Exception::*;

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arity(expected, found) => write!(
                f,
                "arity mismatch: expected {}, found {}",
                expected, found
            ),
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
    /// Produces the error code of the `Exception`.
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
