use context::Context;
use expression::{Callable, Expression};
use im::ConsList;
use std::rc::Rc;

pub mod functions;
pub mod macros;

pub fn load_intrinsics(ctx: &mut Context) {
    load_macros(ctx);
    load_functions(ctx);
}

pub fn init_context() -> Context {
    let mut ctx = Context::new();
    load_intrinsics(&mut ctx);
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

fn load_macros(ctx: &mut Context) {
    use self::macros::*;
    define_macro(ctx, "define", _define);
    define_macro(ctx, "lambda", _lambda);
    define_macro(ctx, "λ", _lambda);
    define_macro(ctx, "env", _env);
    define_macro(ctx, "if", _if);
    define_macro(ctx, "cond", _cond);
    define_macro(ctx, "let", _let);
    define_macro(ctx, "try", _try);
    define_macro(ctx, "define-struct", _define_struct);
    // define_macro(ctx, "set!", _set);
    define_macro(ctx, "begin", _begin);
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

    // Mathematical operators
    define_intrinsic(ctx, "+", functions::_add);
    define_intrinsic(ctx, "-", functions::_sub);
    define_intrinsic(ctx, "*", functions::_mul);
    define_intrinsic(ctx, "/", functions::_div);
    define_intrinsic(ctx, "%", functions::_rem);
    define_intrinsic(ctx, "eq?", functions::_eq);
    define_intrinsic(ctx, "=", functions::_eq);
    define_intrinsic(ctx, ">", functions::_gt);
    define_intrinsic(ctx, ">=", functions::_gte);
    define_intrinsic(ctx, "<", functions::_lt);
    define_intrinsic(ctx, "<=", functions::_lte);

    // Mathematical functions
    define_intrinsics! {
        context: ctx,

        // Mathematical functions
        "sin" => _sin,
        "cos" => _cos,
        "tan" => _tan,
        "csc" => _csc,
        "sec" => _sec,
        "cot" => _cot,
        "asin" => _asin,
        "acos" => _acos,
        "atan" => _atan,
        "sqrt" => _sqrt,

        // Boolean logic
        "and" => _and,
        "&&" => _and,
        "or" => _or,
        "||" => _or,
        "not" => _not,

        "set!" => _set,

        // Lists
        "cons" => _cons,
        ":" => _cons,
        "head" => _head,
        "tail" => _tail,

        "exit" => _exit,
        "display" => _display,
        "display-debug" => _display_debug,
        "display-pretty" => _display_pretty,
        "newline" => _newline,
        "readline" => _readline,

        "++" => _append,
        "append" => _append,
        "empty?" => _empty,
        "eval" => _eval,
        "import" => _import,
        "readfile" => _readfile,
        "parse" => _parse,
        "type-of" => _type_of,
        "format" => _format,
    }

    // Boolean logic

    // Lists
}
