extern crate rlisp_core;
use rlisp_core::expression::Expression;
use rlisp_core::prelude::*;
use rlisp_core::util::{clear_color, set_red};
use rlisp_core::intrinsics::functions::_import;
use std::io::prelude::*;
use std::io::{stdin, stdout};

fn main() {
    let mut ctx = Context::new();
    load(&mut ctx);

    // Load stdlib
    let res = _import(&[Expression::Str("stdlib.rlisp".into())], &mut ctx);
    if let err @ Expression::Exception(..) = res {
        set_red();
        println!("{}", err);
        clear_color();
        return;
    }

    let mut line = String::new();
    loop {
        print!("> ");
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
