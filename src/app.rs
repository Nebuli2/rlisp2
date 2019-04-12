use crate::repl::run_repl;
use clap::{App, Arg, ArgMatches};
use std::env;
use std::path::Path;

use rlisp_interpreter::{expression::Expression::*, util::print_stack_trace};
use rlisp_intrinsics::{functions::import, init_context};

fn rlisp_home() -> String {
    env::var("RLISP_HOME").expect("RLISP_HOME not defined")
}

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
        .arg(Arg::with_name("program_args")
            .multiple(true)
            .help("Additional arguments passed to the script")
            .required(false))
        .get_matches()
}

pub fn run() {
    let matches = create_app();

    let lib_loc = matches
        .value_of("lib-loc")
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            let home_path = rlisp_home();
            let mut home_path = Path::new(&home_path).to_path_buf();
            home_path.push("loader.rl");
            home_path.into_os_string().into_string().unwrap()
        });

    let mut ctx = init_context(env!("CARGO_PKG_VERSION"));
    let res = import(&[Str(lib_loc.into())], &mut ctx);

    if let Error(ex) = res {
        print_stack_trace(&ex);
        return;
    }

    match matches.value_of("INPUT") {
        Some(input) => {
            // Load input file
            ctx.remove("__FILE__");
            let res = import(&[Str(input.into())], &mut ctx);
            if let Error(ex) = res {
                print_stack_trace(&ex);
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
