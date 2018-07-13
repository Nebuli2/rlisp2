extern crate im;
use im::ConsList;

mod context;
mod environment;
mod exception;
mod expression;
mod intrinsics;
mod parser;
mod util;

use context::Context;
use expression::Expression;
use parser::preprocessor::*;
use parser::Parser;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdin;
use std::io::stdout;
use std::io::BufReader;
use util::wrap_begin;

fn load(file: &str) -> Result<expression::Expression, Box<Error>> {
    let file = File::open(file)?;
    let mut reader = BufReader::new(file);

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let stripped = strip_comments(buf);
    let processed = process(stripped);

    let iter = processed.chars();

    let mut parser = Parser::new(iter);

    let mut exprs = Vec::new();
    while let Some(expr) = parser.parse_expr() {
        exprs.push(expr);
    }

    let expr = wrap_begin(exprs.into());
    Ok(expr)
    // Ok(expr)
}

fn main() {
    let mut ctx = Context::new();
    intrinsics::load_functions(&mut ctx);
    intrinsics::load_macros(&mut ctx);

    // Load file
    let expr = load("test2.rlisp").expect("could not read file");
    let _evaluated = expr.eval(&mut ctx);

    loop {
        print!("> ");
        stdout().flush().expect("could not flush");
        let mut line_buf = String::new();
        stdin()
            .read_line(&mut line_buf)
            .expect("could not read line");
        let processed = process(strip_comments(line_buf));
        let mut parser = Parser::new(processed.chars());
        let expr = parser.parse_expr();
        if let Some(expr) = expr {
            // println!("parsed: {:?}", expr);
            let evaluated = expr.eval(&mut ctx);
            if !evaluated.is_nil() {
                println!("{}", evaluated);
            }
        }
    }
}
