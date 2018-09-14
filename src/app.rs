use clap::{App, Arg, ArgMatches};
use repl::run_repl;
use rlisp_core::{intrinsics::functions::_import, prelude::*, util::print_err};

fn create_app<'a>() -> ArgMatches<'a> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("lib-loc")
            .short("L")
            .long("lib")
            .value_name("LIB_LOC")
            .help("Sets the location to load the standard library from")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to interpret")
            .required(false)
            .index(1))
        .arg(Arg::with_name("interactive")
            .short("i")
            .long("interactive")
            .takes_value(false)
            .help("Determines whether or not to start an interactive REPL session after loading the specified input")
            .required(false))
        .get_matches()
}

pub fn run() {
    let matches = create_app();
    let lib_loc = matches
        .value_of("lib")
        .unwrap_or_else(|| "rlisp-lib/loader.rl");

    let mut ctx = init_context();
    let res = _import(&[Expression::Str(lib_loc.into())], &mut ctx);

    if let Expression::Exception(ex) = res {
        print_err(&ex);
        return;
    }

    match matches.value_of("INPUT") {
        Some(input) => {
            // Load input file
            let res = _import(&[Expression::Str(input.into())], &mut ctx);
            if let Expression::Exception(ex) = res {
                print_err(&ex);
                return;
            }

            if matches.is_present("interactive") {
                let s = "(start-repl)";
                let mut parser = Parser::new(s.chars());
                parser.parse_expr().map(|expr| expr.eval(&mut ctx));
                // run_repl(&mut ctx);
            }
        }
        None => {
            let s = "(start-repl)";
            let mut parser = Parser::new(s.chars());
            parser.parse_expr().map(|expr| expr.eval(&mut ctx));
            // run_repl(&mut ctx);
        }
    }
}