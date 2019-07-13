pub extern crate rlisp_interpreter as interpreter;
pub extern crate rlisp_intrinsics as intrinsics;
pub extern crate rlisp_parser as parser;

#[cfg(feature = "native")]
pub mod app;

pub mod repl;

pub mod prelude {
    #[cfg(feature = "native")]
    pub use crate::app::*;
    pub use crate::repl::*;
}
