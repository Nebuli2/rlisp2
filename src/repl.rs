use rlisp_core::prelude::*;
use std::io::prelude::*;
use std::io::{stdout, stdin};
use rlisp_core::util::{set_green, clear_color, print_err};

pub fn run_repl(ctx: &mut Context) {
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
                let result = expr.eval(ctx);
                match result {
                    Expression::Exception(ex) => {
                        print_err(&ex);
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