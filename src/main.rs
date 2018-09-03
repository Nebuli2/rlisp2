extern crate rlisp_core;

use rlisp_core::{
    expression::Expression,
    intrinsics::functions::_import,
    prelude::*,
    util::{clear_color, set_red, set_green},
};
use std::io::{prelude::*, stdin, stdout};

fn main() {
    let mut ctx = Context::new();
    load(&mut ctx);

    // Load stdlib
    let res = _import(&[Expression::Str("rlisp-lib/stdlib.rlisp".into())], &mut ctx);
    if let err @ Expression::Exception(..) = res {
        set_red();
        println!("{}", err);
        clear_color();
        return;
    }

    let mut line = String::new();
    loop {
        let prompt = ctx
            .get("PROMPT")
            .and_then(|p| match p {
                Expression::Str(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "rlisp> ".to_string());
        set_green();
        print!("{}", prompt);
        clear_color();
        stdout().flush().expect("failed to flush stdout");
        stdin().read_line(&mut line).expect("failed to read line");
        {
            let mut parser = Parser::new(line.chars());
            parser.parse_expr().map(|expr| {
                let result = expr.eval(&mut ctx);
                match result {
                    err @ Expression::Exception(..) => {
                        set_red();
                        println!("{}", err);
                        clear_color();
                    }
                    ref res if !res.is_nil() => {
                        println!("{}", res);
                    }
                    _ => {}
                }
            });
        }
        line.clear();
    }
}
