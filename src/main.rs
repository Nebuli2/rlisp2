#![allow(unknown_lints)]
#![warn(clippy)]

extern crate clap;

#[macro_use]
extern crate rlisp_core;

mod app;
mod repl;

fn main() {
    app::run();
}
