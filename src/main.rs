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

    // Look for directive lines
    let mut use_preprocessor = false;
    {
        let iter = buf.lines()
            .filter(|line| !line.is_empty())
            .filter(|line| line.trim().starts_with('#'))
            .map(|line| line.split_at(1).1);
        for line in iter {
            if line == "enable-preprocessor" {
                use_preprocessor = true;
            } else {
                Err(format!("{} is not a known preprocessor command", line))?;
            }
        }
    }

    let removed_commands: String = buf.lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    // println!("{}", removed_commands);
    let processed;
    let iter = match use_preprocessor {
        true => {
            let stripped = first_pass(removed_commands);
            processed = process(stripped);
            processed.chars()
        },
        false => removed_commands.chars()
    };

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
    let preprocess_repl = false;

    let mut ctx = Context::new();
    intrinsics::load_functions(&mut ctx);
    intrinsics::load_macros(&mut ctx);

    // Load file
    let expr = load("stdlib.rlisp").expect("could not read file");
    let _evaluated = expr.eval(&mut ctx);

    loop {
        print!("> ");
        stdout().flush().expect("could not flush");
        let mut line_buf = String::new();
        stdin()
            .read_line(&mut line_buf)
            .expect("could not read line");

        let line = match preprocess_repl {
            true => process(first_pass(line_buf)),
            false => line_buf
        };
        let mut parser = Parser::new(line.chars());
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
