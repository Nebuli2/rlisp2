//! This module provides access to intrinsic functions of the interpreter.

use context::Context;
use expression::{Callable, Expression};
use im::ConsList;
use std::rc::Rc;

pub mod functions;
pub mod macros;

/// Creates a context and loads all intrinsic functions and macros into it.
pub fn init_context() -> Context {
    let mut ctx = Context::new();
    load_functions(&mut ctx);
    load_macros(&mut ctx);
    ctx.insert("pi", Expression::Num(std::f64::consts::PI));
    ctx
}

fn define_intrinsic(
    ctx: &mut Context,
    ident: impl ToString,
    f: impl Fn(&[Expression], &mut Context) -> Expression + 'static,
) {
    ctx.insert(
        ident.to_string(),
        Expression::Callable(Callable::Intrinsic(Rc::new(f))),
    );
}

fn define_macro(
    ctx: &mut Context,
    ident: impl ToString,
    f: impl Fn(ConsList<Expression>, &mut Context) -> Expression + 'static,
) {
    ctx.insert(
        ident.to_string(),
        Expression::Callable(Callable::Macro(Rc::new(f))),
    );
}

macro_rules! define_macros {
    {
        context: $ctx:expr,
        $($name:expr => $func:expr),*,
    } => ({
        $(
            define_macro($ctx, $name, $func);
        )*
    });
    {
        context: $ctx:expr,
        $($name:expr => $func:expr),*
    } => ({
        $(
            define_macro($ctx, $name, $func);
        )*
    });
}

fn load_macros(ctx: &mut Context) {
    use self::macros::*;
    define_macros! {
        context: ctx,
        "define" => define,
        "lambda" => lambda,
        "λ" => lambda,
        "env" => env,
        "if" => if_expr,
        "cond" => cond,
        "let" => let_expr,
        "try" => try_expr,
        "define-struct" => define_struct,
        "begin" => begin,
    }
}

macro_rules! define_intrinsics {
    {
        context: $ctx:expr,
        $($name:expr => $func:expr),*,
    } => ({
        $(
            define_intrinsic($ctx, $name, $func);
        )*
    });
    {
        context: $ctx:expr,
        $($name:expr => $func:expr),*
    } => ({
        $(
            define_intrinsic($ctx, $name, $func);
        )*
    });
}

fn load_functions(ctx: &mut Context) {
    use self::functions::*;

    define_intrinsics! {
        context: ctx,

        // Operators
        "+" => add,
        "-" => sub,
        "*" => mul,
        "/" => div,
        "%" => rem,
        "rem" => rem,
        "eq?" => eq,
        "=" => eq,
        ">" => gt,
        ">=" => gte,
        "<" => lt,
        "<=" => lte,

        // Mathematical functions
        "sin" => sin,
        "cos" => cos,
        "tan" => tan,
        "csc" => csc,
        "sec" => sec,
        "cot" => cot,
        "asin" => asin,
        "acos" => acos,
        "atan" => atan,
        "sqrt" => sqrt,

        // Boolean logic
        "and" => and,
        "&&" => and,
        "or" => or,
        "||" => or,
        "not" => not,

        "set!" => set,

        // Lists
        "cons" => cons,
        ":" => cons,
        "head" => head,
        "tail" => tail,

        "exit" => exit,
        "display" => display,
        "display-debug" => display_debug,
        "display-pretty" => display_pretty,
        "newline" => newline,
        "readline" => readline,

        "++" => append,
        "append" => append,
        "empty?" => empty,
        "eval" => eval,
        "import" => import,
        "readfile" => readfile,
        "parse" => parse,
        "type-of" => type_of,
        "format" => format,

        "quat" => quaternion,
        "exp" => exp,
        "ln" => ln,
        "env-var" => env_var,
    }

    // Boolean logic

    // Lists
}
