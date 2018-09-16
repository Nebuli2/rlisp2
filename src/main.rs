#![allow(unknown_lints)]
#![warn(clippy)]

extern crate rlisp_core;
extern crate clap;


mod repl;
mod app;

fn main() {
    app::run();
}
