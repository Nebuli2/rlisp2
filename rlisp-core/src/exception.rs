//! The `exception` module deals with handling exceptions for the rlisp
//! language. As of now, 5 different exception varieties are offered:
//! * `Arity`
//! * `Signature`
//! * `Undefined`
//! * `Syntax`
//! * `Custom`

use crate::{expression::Expression, util::Str};
use im::ConsList;
use std::fmt;

/// An `ErrorCode` is a numerical representation of each individual type of
/// error.
pub type ErrorCode = u16;

#[derive(Clone, Debug)]
pub struct Exception {
    pub stack: ConsList<Expression>,
    pub data: ExceptionData,
}

impl Exception {
    pub fn arity(expected: usize, found: usize) -> Exception {
        Exception {
            stack: ConsList::new(),
            data: Arity(expected, found),
        }
    }

    pub fn signature(
        expected: impl Into<Str>,
        found: impl Into<Str>,
    ) -> Exception {
        Exception {
            stack: ConsList::new(),
            data: Signature(expected.into(), found.into()),
        }
    }

    pub fn custom(code: ErrorCode, description: impl Into<Str>) -> Exception {
        Exception {
            stack: ConsList::new(),
            data: Custom(code, description.into()),
        }
    }

    pub fn undefined(symbol: impl Into<Str>) -> Exception {
        Exception {
            stack: ConsList::new(),
            data: Undefined(symbol.into()),
        }
    }

    pub fn syntax(code: ErrorCode, description: impl Into<Str>) -> Exception {
        Exception {
            stack: ConsList::new(),
            data: Syntax(code, description.into()),
        }
    }

    pub fn extend(&self, expr: &Expression) -> Exception {
        let Exception { stack, data } = self.clone();
        Exception {
            stack: stack.cons(expr.clone()),
            data,
        }
    }

    pub fn error_code(&self) -> ErrorCode {
        self.data.error_code()
    }

    pub fn stack(&self) -> ConsList<Expression> {
        self.stack.clone()
    }

    pub fn print_stack_trace(&self) {}
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

/// The `Exception` type represents all possible exceptions in the rlisp
/// language.
#[derive(Clone, Debug)]
pub enum ExceptionData {
    /// An arity mismatch exception, where the number of arguments
    /// provided to a function does not match the number of arguments expected.
    Arity(usize, usize),

    /// A type signature mismatch exception, where the types of the
    /// arguments provided to a function do not match the types expected.
    Signature(Str, Str),

    /// Any exception that does not fit into the other categories.
    Custom(ErrorCode, Str),

    /// An exception wherein the specified symbol is undefined.
    Undefined(Str),

    /// An arbitrary syntax exception.
    Syntax(ErrorCode, Str),
}

use self::ExceptionData::*;

impl fmt::Display for ExceptionData {
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

impl ExceptionData {
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
