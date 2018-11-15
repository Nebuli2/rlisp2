//! This crate provides an interpreter for the rlisp language.
//! # Examples
//! ```rust
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<Error>> {
//! # use crate::prelude::*;
//!
//! // Create an expression to run
//! let code = "(+ 1 2 3)";
//! let mut parser = Parser::new(code.chars());
//! let expr = parser.parse_expr()?;
//!
//! // Create evaluation context for expression
//! let mut ctx = init_ctx();
//!
//! // Evaluate the expression
//! let res = expr.eval(&mut ctx);
//! println!("{}", res);
//! # }
//! ```

#![forbid(unsafe_code)]
// #![warn(missing_docs)]

pub extern crate im;
extern crate termcolor;

#[macro_use]
extern crate lazy_static;
extern crate regex;

#[macro_use]
pub mod util;

pub mod context;
pub mod exception;
pub mod expression;
pub mod intrinsics;

#[macro_use]
pub mod parser;

pub mod math;
pub mod quat;

/// The prelude module re-exports commonly used portions of the `rlisp_core`
/// crate for easier access.
pub mod prelude {
    pub use {
        context::Context, exception::Exception, expression::Expression,
        intrinsics::init_context, parser::Parser, util::nil,
    };
}
