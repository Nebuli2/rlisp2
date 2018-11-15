use clap::{App, Arg, ArgMatches};
use repl::run_repl;
use rlisp_core::{
    expression::Expression::*, intrinsics::functions::import, prelude::*,
    util::print_err,
};

const RLISP_HOME: &str = env!("RLISP_HOME");

fn create_app<'a>() -> ArgMatches<'a> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("lib-loc")
            .short("L")
            .long("lib")
            .value_name("LIB_LOC")
            .help("Sets the location to load the standard library from. The loader file must specify a (interactive-start) function that is called if interactive mode is enabled.")
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
        .value_of("lib-loc")
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{}/loader.rl", RLISP_HOME));

    let mut ctx = init_context();
    let res = import(&[Str(lib_loc.into())], &mut ctx);

    if let Exception(ex) = res {
        print_err(&ex);
        return;
    }

    match matches.value_of("INPUT") {
        Some(input) => {
            // Load input file
            let res = import(&[Str(input.into())], &mut ctx);
            if let Exception(ex) = res {
                print_err(&ex);
                return;
            }

            if matches.is_present("interactive") {
                run_repl(&mut ctx);
            }
        }
        None => {
            run_repl(&mut ctx);
        }
    }
}
