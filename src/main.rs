#![allow(unknown_lints)]
#![warn(clippy)]

extern crate clap;
extern crate rlisp_core;

mod app;
mod repl;

fn main() {
    let size = std::mem::size_of::<rlisp_core::expression::Callable>();
    println!("size = {} bytes", size);
    app::run();
    // let mut buf = String::new();
    // let stdin = std::io::stdin();
    // loop {
    //     println!("Enter a number:");
    //     stdin.read_line(&mut buf).expect("failed to read stdin");
    //     let res = buf.parse::<rlisp_core::quat::Quat>();
    //     println!("res: {:?}", res);
    //     buf.clear();
    // }
}
