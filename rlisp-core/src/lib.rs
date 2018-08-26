extern crate im;
extern crate termcolor;

pub mod context;
pub mod exception;
pub mod expression;
pub mod intrinsics;
pub mod parser;
pub mod util;

pub mod prelude {
    pub use {
        context::Context, exception::Exception, expression::Expression, intrinsics::load,
        parser::Parser, util::nil,
    };
    // pub use context::Context;
    // pub use environment::Environment;
    // pub use expression::Expression;
    // pub use exception::Exception;
    // pub use parser::Parser;
    // pub use intrinsics::load;
    // pub use util::{
    //     nil
    // };
}
