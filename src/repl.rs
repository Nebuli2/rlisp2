use rlisp_core::expression::Expression::*;
use rlisp_core::prelude::*;
use rlisp_core::util::print_stack_trace;

const REPL: &str = r#"
    (interactive-start)
"#;

pub fn run_repl(ctx: &mut Context) {
    Parser::new(REPL.chars())
        .parse_expr()
        .map(|expr| expr.eval(ctx))
        .map(|res| {
            if let Error(ex) = res {
                print_stack_trace(&ex);
            }
        })
        .unwrap_or_else(|| {
            println!("unknown error occurred");
        });
}
