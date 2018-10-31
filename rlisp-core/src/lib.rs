//! Foo

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub extern crate im;
extern crate termcolor;

#[macro_use]
pub mod util;

pub mod context;
pub mod exception;
pub mod expression;
pub mod intrinsics;

#[macro_use]
pub mod parser;

/// The prelude module re-exports commonly used portions of the `rlisp_core`
/// crate for easier access.
pub mod prelude {
    pub use {
        context::Context, exception::Exception, expression::Expression,
        intrinsics::init_context, parser::Parser, util::nil,
    };
}
