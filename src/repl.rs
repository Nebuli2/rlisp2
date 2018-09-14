use rlisp_core::prelude::*;
use rlisp_core::util::{print_err, print_prompt};
use std::io::prelude::*;
use std::io::{stdin, stdout};

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
        print_prompt(prompt);
        stdout().flush().expect("failed to flush stdout");
        stdin().read_line(&mut line).expect("failed to read line");
        {
            let mut parser = Parser::new(line.chars());
            let expr = parser.parse_all();
            let result = expr.eval(ctx);
            match &result {
                Expression::Exception(ex) => {
                    print_err(&ex);
                }
                ref res if !res.is_nil() => {
                    println!("= {}", res);
                }
                _ => {}
            }
            ctx.insert("_", result);
        }
        line.clear();
    }
}
