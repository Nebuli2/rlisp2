extern crate im;
extern crate termcolor;

#[macro_use]
pub mod util;

pub mod context;
pub mod exception;
pub mod expression;
pub mod intrinsics;
pub mod parser;

pub mod prelude {
    pub use {
        context::Context, exception::Exception, expression::Expression, intrinsics::load,
        parser::Parser, util::nil,
    };
}
