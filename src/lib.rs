pub extern crate rlisp_interpreter as interpreter;
pub extern crate rlisp_intrinsics as intrinsics;
pub extern crate rlisp_parser as parser;

#[cfg(not(feature = "wasm"))]
pub mod app;

pub mod repl;

pub mod prelude {
    #[cfg(not(feature = "wasm"))]
    pub use crate::app::*;
    pub use crate::repl::*;
}
